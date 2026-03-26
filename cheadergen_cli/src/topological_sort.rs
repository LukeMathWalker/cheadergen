use std::collections::{BTreeMap, HashMap, HashSet};

use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use rustdoc_ir::Type;

use crate::Collection;
use crate::analysis::{CTypeDefinition, CTypeKind, c_type_name, span_sort_key_global};
use crate::diagnostic::DiagnosticSink;

/// Sort type definitions in dependency order: types used **by value** in
/// another type's fields are emitted first.
///
/// Within the same topological level, items are ordered by source position
/// (line, column) as a stable tiebreaker.
///
/// If a cycle exists in by-value dependencies (impossible in valid
/// `#[repr(C)]` Rust), the remaining types are appended in source order
/// with a warning.
pub fn topological_sort(
    type_defs: &mut Vec<CTypeDefinition>,
    collection: &Collection,
    diagnostics: &mut DiagnosticSink,
) {
    // Only compound types (structs, tagged unions) participate in ordering.
    // Fieldless enums and opaques have no by-value dependencies on other compounds.
    // We sort only the compounds and leave the rest untouched — codegen partitions
    // by kind anyway.

    // Separate compounds from others, preserving relative order of non-compounds.
    let mut compounds: Vec<CTypeDefinition> = Vec::new();
    let mut non_compounds: Vec<CTypeDefinition> = Vec::new();
    let mut layout: Vec<bool> = Vec::new(); // true = compound slot

    for def in type_defs.drain(..) {
        let is_compound = matches!(
            def.kind,
            CTypeKind::Struct(_)
                | CTypeKind::Union(_)
                | CTypeKind::TaggedUnion(_)
                | CTypeKind::Typedef(_)
        );
        layout.push(is_compound);
        if is_compound {
            compounds.push(def);
        } else {
            non_compounds.push(def);
        }
    }

    if compounds.len() <= 1 {
        // Nothing to sort; put everything back.
        let mut nc_iter = non_compounds.into_iter();
        let mut c_iter = compounds.into_iter();
        for is_compound in &layout {
            if *is_compound {
                type_defs.push(c_iter.next().unwrap());
            } else {
                type_defs.push(nc_iter.next().unwrap());
            }
        }
        return;
    }

    // Build name→node map.
    let name_to_idx: HashMap<&str, usize> = compounds
        .iter()
        .enumerate()
        .map(|(i, d)| (d.name.as_str(), i))
        .collect();

    let n = compounds.len();

    // Build a dependency graph.
    // Edge direction: dep → dependent (dep must be emitted before dependent).
    let mut graph = DiGraph::<usize, ()>::with_capacity(n, 0);
    let nodes: Vec<NodeIndex> = (0..n).map(|i| graph.add_node(i)).collect();

    for (i, def) in compounds.iter().enumerate() {
        for dep_name in by_value_dependencies(def) {
            if let Some(&dep_idx) = name_to_idx.get(dep_name.as_str())
                && dep_idx != i
            {
                graph.add_edge(nodes[dep_idx], nodes[i], ());
            }
        }
    }

    // Kahn's algorithm with source-order tiebreaker.
    let sort_key = |idx: usize| -> (usize, usize, &str) {
        let def = &compounds[idx];
        let (line, col) = span_sort_key_for_def(def, collection);
        (line, col, def.name.as_str())
    };

    let mut in_degree: Vec<usize> = (0..n)
        .map(|i| graph.neighbors_directed(nodes[i], Direction::Incoming).count())
        .collect();

    // BTreeMap keyed by (line, col, name) → idx for deterministic ordering.
    let mut queue: BTreeMap<(usize, usize, String), usize> = BTreeMap::new();
    for (i, &deg) in in_degree.iter().enumerate() {
        if deg == 0 {
            let (l, c, name) = sort_key(i);
            queue.insert((l, c, name.to_owned()), i);
        }
    }

    let mut sorted_indices: Vec<usize> = Vec::with_capacity(n);

    while let Some(entry) = queue.first_key_value().map(|(k, v)| (k.clone(), *v)) {
        let (key, idx) = entry;
        queue.remove(&key);
        sorted_indices.push(idx);

        for dependent in graph.neighbors_directed(nodes[idx], Direction::Outgoing) {
            let dep_idx = graph[dependent];
            in_degree[dep_idx] -= 1;
            if in_degree[dep_idx] == 0 {
                let (l, c, name) = sort_key(dep_idx);
                queue.insert((l, c, name.to_owned()), dep_idx);
            }
        }
    }

    // Handle cycles.
    if sorted_indices.len() < n {
        let in_sorted: HashSet<usize> = sorted_indices.iter().copied().collect();

        // Find SCCs — each non-trivial SCC contains at least one cycle.
        let sccs = petgraph::algo::tarjan_scc(&graph);
        let mut cycle_found = false;
        for scc in &sccs {
            if scc.len() < 2 {
                continue;
            }
            cycle_found = true;
            let cycle = extract_cycle_from_scc(scc, &graph, &compounds);
            diagnostics
                .warning(format!("cycle detected in by-value type dependencies: {cycle}"))
                .with_help("appending remaining types in source order")
                .emit();
        }
        if !cycle_found {
            diagnostics
                .warning(
                    "topological sort did not converge, but no cycle was found; \
                     this is a cheadergen bug",
                )
                .with_help("appending remaining types in source order")
                .emit();
        }

        let mut remaining: Vec<usize> = (0..n).filter(|i| !in_sorted.contains(i)).collect();
        remaining.sort_by_key(|&i| {
            let (l, c, name) = sort_key(i);
            (l, c, name.to_owned())
        });
        sorted_indices.extend(remaining);
    }

    // Reorder compounds according to sorted_indices.
    let mut compounds_opt: Vec<Option<CTypeDefinition>> = compounds.into_iter().map(Some).collect();
    let sorted_compounds: Vec<CTypeDefinition> = sorted_indices
        .iter()
        .map(|&i| compounds_opt[i].take().unwrap())
        .collect();

    // Reassemble type_defs: non-compounds keep their relative positions,
    // compounds are placed in their slots in the new topological order.
    let mut nc_iter = non_compounds.into_iter();
    let mut c_iter = sorted_compounds.into_iter();
    for is_compound in &layout {
        if *is_compound {
            type_defs.push(c_iter.next().unwrap());
        } else {
            type_defs.push(nc_iter.next().unwrap());
        }
    }
}

/// Walk the dependency edges within an SCC to extract a cycle in dependency order.
///
/// Edges in the graph point dep → dependent. We follow `Incoming` edges
/// (i.e. "depends on") to trace the cycle in the order a developer would
/// read: `A -> B -> C -> A` means A depends on B, B depends on C, C depends on A.
fn extract_cycle_from_scc(
    scc: &[NodeIndex],
    graph: &DiGraph<usize, ()>,
    compounds: &[CTypeDefinition],
) -> String {
    let scc_set: HashSet<NodeIndex> = scc.iter().copied().collect();

    // Start from the node with the smallest name for deterministic output.
    let start = *scc
        .iter()
        .min_by_key(|&&n| &compounds[graph[n]].name)
        .unwrap();

    let mut path = vec![start];
    let mut visited = HashSet::new();
    visited.insert(start);
    let mut current = start;

    // Follow "depends on" (Incoming) edges within the SCC until we revisit the start.
    loop {
        let next = graph
            .neighbors_directed(current, Direction::Incoming)
            .find(|n| scc_set.contains(n) && !visited.contains(n))
            .or_else(|| {
                // All neighbors visited — close the cycle back to start.
                graph
                    .neighbors_directed(current, Direction::Incoming)
                    .find(|n| *n == start)
            });
        match next {
            Some(n) if n == start => break,
            Some(n) => {
                path.push(n);
                visited.insert(n);
                current = n;
            }
            None => break,
        }
    }

    // Format: "Foo -> Bar -> Baz -> Foo"
    let names: Vec<&str> = path
        .iter()
        .map(|&n| compounds[graph[n]].name.as_str())
        .collect();
    let start_name = compounds[graph[start]].name.as_str();
    format!("{} -> {}", names.join(" -> "), start_name)
}

fn span_sort_key_for_def(
    def: &CTypeDefinition,
    collection: &Collection,
) -> (usize, usize) {
    let Some(gid) = &def.rustdoc_id else {
        return (usize::MAX, usize::MAX);
    };
    span_sort_key_global(gid, collection)
}

/// Extract the C type names that `def` depends on **by value** (not behind a pointer).
fn by_value_dependencies(def: &CTypeDefinition) -> Vec<String> {
    let mut deps = Vec::new();
    match &def.kind {
        CTypeKind::Struct(s) => {
            for field in &s.fields {
                collect_by_value_type_deps(&field.type_, &mut deps);
            }
        }
        CTypeKind::Union(u) => {
            for field in &u.fields {
                collect_by_value_type_deps(&field.type_, &mut deps);
            }
        }
        CTypeKind::TaggedUnion(t) => {
            for variant in &t.variants {
                if let Some(ref body) = variant.body {
                    for field in &body.fields {
                        collect_by_value_type_deps(&field.type_, &mut deps);
                    }
                }
            }
        }
        CTypeKind::Typedef(t) => {
            collect_by_value_type_deps(&t.inner, &mut deps);
        }
        CTypeKind::OpaqueStruct | CTypeKind::OpaqueUnion | CTypeKind::FieldlessEnum(_) => {}
    }
    deps
}

/// Recursively collect by-value type dependencies from a field type.
fn collect_by_value_type_deps(ty: &Type, deps: &mut Vec<String>) {
    match ty {
        Type::Path(_) | Type::TypeAlias(_) => {
            deps.push(c_type_name(ty));
        }
        Type::Array(a) => {
            collect_by_value_type_deps(&a.element_type, deps);
        }
        // Pointers and references are NOT by-value dependencies.
        Type::RawPointer(_) | Type::Reference(_) => {}
        // Primitives, tuples, etc. have no compound dependencies.
        _ => {}
    }
}
