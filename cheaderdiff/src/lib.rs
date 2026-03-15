use std::collections::BTreeMap;
use std::fmt;

/// A top-level item that exists in both headers but with different content.
pub struct ChangedItem {
    /// The extracted name of the item (e.g. function name, typedef name).
    pub name: String,
    /// The full source text from the left header.
    pub left: String,
    /// The full source text from the right header.
    pub right: String,
}

/// The result of comparing two headers.
pub enum HeaderDiff {
    /// Both headers are semantically equivalent.
    Equivalent,
    /// The headers differ.
    Different {
        /// Items present in the left header but not in the right.
        left_only: Vec<String>,
        /// Items present in the right header but not in the left.
        right_only: Vec<String>,
        /// Items present in both headers (matched by name) but with different content.
        changed: Vec<ChangedItem>,
    },
}

impl fmt::Display for ChangedItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[{}]", self.name)?;
        writeln!(f, "  left:  {}", self.left)?;
        write!(f, "  right: {}", self.right)
    }
}

impl fmt::Display for HeaderDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let HeaderDiff::Different {
            left_only,
            right_only,
            changed,
        } = self
        else {
            return write!(f, "Equivalent");
        };

        let mut need_blank = false;

        if !left_only.is_empty() {
            writeln!(f, "Left only:")?;
            for item in left_only {
                writeln!(f, "  {item}")?;
            }
            need_blank = true;
        }

        if !right_only.is_empty() {
            if need_blank {
                writeln!(f)?;
            }
            writeln!(f, "Right only:")?;
            for item in right_only {
                writeln!(f, "  {item}")?;
            }
            need_blank = true;
        }

        if !changed.is_empty() {
            if need_blank {
                writeln!(f)?;
            }
            writeln!(f, "Changed:")?;
            for (i, item) in changed.iter().enumerate() {
                if i > 0 {
                    writeln!(f)?;
                }
                let rendered = item.to_string();
                for (j, line) in rendered.lines().enumerate() {
                    if j == 0 {
                        writeln!(f, "  {line}")?;
                    } else if line.is_empty() {
                        writeln!(f)?;
                    } else {
                        writeln!(f, "  {line}")?;
                    }
                }
            }
        }

        Ok(())
    }
}

/// Compare two C header files for equivalence modulo top-level item reordering.
///
/// Parses both headers with tree-sitter-c, extracts top-level items (ignoring
/// comments), and returns a structured diff of items that appear in one but
/// not the other, as well as items that share a name but differ in content.
pub fn diff_headers(left: &str, right: &str) -> anyhow::Result<HeaderDiff> {
    let left_items = extract_top_level_items(left)?;
    let right_items = extract_top_level_items(right)?;

    let (left_named, left_anon) = partition_by_name(&left_items);
    let (right_named, right_anon) = partition_by_name(&right_items);

    let mut left_only = Vec::new();
    let mut right_only = Vec::new();
    let mut changed = Vec::new();

    // Diff named items by name.
    let all_names: BTreeMap<&str, ()> = left_named
        .keys()
        .chain(right_named.keys())
        .map(|k| (k.as_str(), ()))
        .collect();

    for name in all_names.keys() {
        let l_texts = left_named.get(*name);
        let r_texts = right_named.get(*name);

        match (l_texts, r_texts) {
            (Some(l), Some(r)) => {
                let l_counts = count_items(l);
                let r_counts = count_items(r);

                let all_canonical: BTreeMap<&str, ()> = l_counts
                    .keys()
                    .chain(r_counts.keys())
                    .map(|k| (k.as_str(), ()))
                    .collect();

                for canonical in all_canonical.keys() {
                    let (lc, l_text) = l_counts
                        .get(*canonical)
                        .map(|(c, t)| (*c, t.as_str()))
                        .unwrap_or((0, ""));
                    let (rc, r_text) = r_counts
                        .get(*canonical)
                        .map(|(c, t)| (*c, t.as_str()))
                        .unwrap_or((0, ""));
                    // Use whichever side's text is available (they're
                    // canonically equivalent, so pick the left if present).
                    let text = if !l_text.is_empty() { l_text } else { r_text };
                    if lc > rc {
                        for _ in 0..(lc - rc) {
                            left_only.push(text.to_string());
                        }
                    } else if rc > lc {
                        for _ in 0..(rc - lc) {
                            right_only.push(text.to_string());
                        }
                    }
                }

                // If there are unmatched items on both sides with the same name,
                // pair them as changed items.
                let matched_left: Vec<_> = l
                    .iter()
                    .filter(|(_, t)| left_only.contains(t))
                    .cloned()
                    .collect();
                let matched_right: Vec<_> = r
                    .iter()
                    .filter(|(_, t)| right_only.contains(t))
                    .cloned()
                    .collect();

                let pair_count = matched_left.len().min(matched_right.len());
                for i in 0..pair_count {
                    let (_, ref l_text) = matched_left[i];
                    let (_, ref r_text) = matched_right[i];
                    // Remove from left_only and right_only, add to changed.
                    if let Some(pos) = left_only.iter().position(|x| x == l_text) {
                        left_only.remove(pos);
                    }
                    if let Some(pos) = right_only.iter().position(|x| x == r_text) {
                        right_only.remove(pos);
                    }
                    changed.push(ChangedItem {
                        name: name.to_string(),
                        left: l_text.clone(),
                        right: r_text.clone(),
                    });
                }
            }
            (Some(l), None) => {
                left_only.extend(l.iter().map(|(_, t)| t.clone()));
            }
            (None, Some(r)) => {
                right_only.extend(r.iter().map(|(_, t)| t.clone()));
            }
            (None, None) => unreachable!(),
        }
    }

    // Diff anonymous items by canonical form counting.
    let left_anon_counts = count_items(&left_anon);
    let right_anon_counts = count_items(&right_anon);

    let all_anon: BTreeMap<&str, ()> = left_anon_counts
        .keys()
        .chain(right_anon_counts.keys())
        .map(|k| (k.as_str(), ()))
        .collect();

    for canonical in all_anon.keys() {
        let (lc, l_text) = left_anon_counts
            .get(*canonical)
            .map(|(c, t)| (*c, t.as_str()))
            .unwrap_or((0, ""));
        let (rc, r_text) = right_anon_counts
            .get(*canonical)
            .map(|(c, t)| (*c, t.as_str()))
            .unwrap_or((0, ""));
        let text = if !l_text.is_empty() { l_text } else { r_text };
        if lc > rc {
            for _ in 0..(lc - rc) {
                left_only.push(text.to_string());
            }
        } else if rc > lc {
            for _ in 0..(rc - lc) {
                right_only.push(text.to_string());
            }
        }
    }

    if left_only.is_empty() && right_only.is_empty() && changed.is_empty() {
        return Ok(HeaderDiff::Equivalent);
    }

    left_only.sort();
    right_only.sort();
    changed.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(HeaderDiff::Different {
        left_only,
        right_only,
        changed,
    })
}

/// Extract the identity name from a top-level tree-sitter node.
fn extract_item_name(node: tree_sitter::Node, source: &[u8]) -> Option<String> {
    match node.kind() {
        "declaration" | "function_definition" | "type_definition" => {
            let declarator = node.child_by_field_name("declarator")?;
            extract_name_from_declarator(declarator, source)
        }
        "struct_specifier" | "union_specifier" | "enum_specifier" => {
            let name_node = node.child_by_field_name("name")?;
            name_node.utf8_text(source).ok().map(str::to_string)
        }
        "preproc_def" | "preproc_function_def" => {
            let name_node = node.child_by_field_name("name")?;
            name_node.utf8_text(source).ok().map(str::to_string)
        }
        _ => None,
    }
}

/// Recursively follow the `declarator` field through wrapper nodes until
/// reaching an `identifier` or `type_identifier` leaf.
fn extract_name_from_declarator(node: tree_sitter::Node, source: &[u8]) -> Option<String> {
    match node.kind() {
        "identifier" | "type_identifier" => node.utf8_text(source).ok().map(str::to_string),
        "pointer_declarator"
        | "array_declarator"
        | "function_declarator"
        | "parenthesized_declarator"
        | "attributed_declarator" => {
            let inner = node.child_by_field_name("declarator")?;
            extract_name_from_declarator(inner, source)
        }
        _ => {
            // Try the declarator field as a fallback.
            let inner = node.child_by_field_name("declarator")?;
            extract_name_from_declarator(inner, source)
        }
    }
}

struct NamedItem {
    name: Option<String>,
    text: String,
    /// Whitespace-normalized form used for comparison.
    canonical: String,
}

fn extract_top_level_items(source: &str) -> anyhow::Result<Vec<NamedItem>> {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_c::LANGUAGE;
    parser
        .set_language(&language.into())
        .map_err(|e| anyhow::anyhow!("failed to set C language: {e}"))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("failed to parse C header"))?;

    let root = tree.root_node();
    let mut items = Vec::new();

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "comment" {
            continue;
        }
        let text = child
            .utf8_text(source.as_bytes())
            .map_err(|e| anyhow::anyhow!("invalid UTF-8 in source: {e}"))?;
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            let name = extract_item_name(child, source.as_bytes());
            let canonical = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
            items.push(NamedItem {
                name,
                text: trimmed.to_string(),
                canonical,
            });
        }
    }

    Ok(items)
}

/// A pair of `(canonical, text)` where `canonical` is used for comparison
/// and `text` is the original source for display.
type ItemPair = (String, String);

/// Partition items into named (grouped by name) and anonymous.
fn partition_by_name(items: &[NamedItem]) -> (BTreeMap<String, Vec<ItemPair>>, Vec<ItemPair>) {
    let mut named: BTreeMap<String, Vec<ItemPair>> = BTreeMap::new();
    let mut anon = Vec::new();

    for item in items {
        let pair = (item.canonical.clone(), item.text.clone());
        match &item.name {
            Some(name) => named.entry(name.clone()).or_default().push(pair),
            None => anon.push(pair),
        }
    }

    (named, anon)
}

/// Count items by their canonical form. Returns a map from canonical form to
/// `(count, text)` where `text` is the original source of the first occurrence.
fn count_items(items: &[ItemPair]) -> BTreeMap<String, (usize, String)> {
    let mut counts: BTreeMap<String, (usize, String)> = BTreeMap::new();
    for (canonical, text) in items {
        counts
            .entry(canonical.clone())
            .and_modify(|(c, _)| *c += 1)
            .or_insert((1, text.clone()));
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn identical_headers() {
        let header = "typedef int Foo;\nvoid bar(void);\n";
        let diff = diff_headers(header, header).unwrap();
        assert_snapshot!(diff, @"Equivalent");
    }

    #[test]
    fn same_items_different_order() {
        let left = "typedef int Foo;\nvoid bar(void);\n";
        let right = "void bar(void);\ntypedef int Foo;\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @"Equivalent");
    }

    #[test]
    fn missing_item_in_right() {
        let left = "typedef int Foo;\nvoid bar(void);\n";
        let right = "typedef int Foo;\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @r"
        Left only:
          void bar(void);
        ");
    }

    #[test]
    fn extra_item_in_right() {
        let left = "typedef int Foo;\n";
        let right = "typedef int Foo;\nvoid bar(void);\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @r"
        Right only:
          void bar(void);
        ");
    }

    #[test]
    fn different_function_signature() {
        let left = "void foo(int x);\n";
        let right = "void foo(double x);\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @r"
        Changed:
          [foo]
            left:  void foo(int x);
            right: void foo(double x);
        ");
    }

    #[test]
    fn different_struct_fields() {
        let left = "struct Point { int x; int y; };\n";
        let right = "struct Point { double x; double y; };\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @r"
        Changed:
          [Point]
            left:  struct Point { int x; int y; }
            right: struct Point { double x; double y; }
        ");
    }

    #[test]
    fn same_name_same_content_is_equivalent() {
        let left = "void foo(int x);\n";
        let right = "void foo(int x);\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @"Equivalent");
    }

    #[test]
    fn anonymous_items_compared_by_text() {
        // Semicolons at top level are anonymous items with no extractable name.
        let left = ";\n;\n";
        let right = ";\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @r"
        Left only:
          ;
        ");
    }

    #[test]
    fn comments_ignored() {
        let left = "/* hello */\ntypedef int Foo;\n";
        let right = "// different comment\ntypedef int Foo;\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @"Equivalent");
    }

    #[test]
    fn typedef_changed() {
        let left = "typedef int Foo;\n";
        let right = "typedef double Foo;\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @r"
        Changed:
          [Foo]
            left:  typedef int Foo;
            right: typedef double Foo;
        ");
    }

    #[test]
    fn preproc_def_changed() {
        let left = "#define FOO 1\n";
        let right = "#define FOO 2\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @r"
        Changed:
          [FOO]
            left:  #define FOO 1
            right: #define FOO 2
        ");
    }

    #[test]
    fn duplicate_declarations_same_name() {
        // Two declarations with the same name on the left, one on the right.
        let left = "void foo(int x);\nvoid foo(double x);\n";
        let right = "void foo(int x);\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @r"
        Left only:
          void foo(double x);
        ");
    }

    #[test]
    fn duplicate_declarations_both_sides() {
        // Two declarations with the same name on each side, but different signatures.
        let left = "void foo(int x);\nvoid foo(char y);\n";
        let right = "void foo(double x);\nvoid foo(float y);\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @r"
        Changed:
          [foo]
            left:  void foo(int x);
            right: void foo(double x);

          [foo]
            left:  void foo(char y);
            right: void foo(float y);
        ");
    }

    #[test]
    fn whitespace_in_struct_body() {
        // Same struct but with different internal whitespace/blank lines.
        let left = "struct S {\n    int x;\n    int y;\n};\n";
        let right = "struct S {\n    int x;\n\n    int y;\n};\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @"Equivalent");
    }

    #[test]
    fn mixed_left_right_and_changed() {
        let left = "void a(void);\nvoid b(int x);\nvoid c(void);\n";
        let right = "void b(double x);\nvoid d(void);\n";
        let diff = diff_headers(left, right).unwrap();
        assert_snapshot!(diff, @r"
        Left only:
          void a(void);
          void c(void);

        Right only:
          void d(void);

        Changed:
          [b]
            left:  void b(int x);
            right: void b(double x);
        ");
    }
}
