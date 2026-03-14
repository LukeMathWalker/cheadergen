use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write;
use std::path::Path;

use crate::analysis::{
    CEnumRepr, CFieldlessEnumDef, CIdentifier, CStructDef, CTaggedUnionDef, CTypedefDef,
    CTypeDefinition, CTypeKind, CUnionDef, c_type_name,
};
use crate::config::{CConfig, CommonConfig, DocumentationLength, DocumentationStyle, Style};
use crate::constant_item::ConstantItem;
use crate::static_item::StaticItem;
use rustdoc_ir::{FreeFunction, FunctionPointer, ScalarPrimitive, Type};
use rustdoc_processor::indexing::CrateIndexer;
use rustdoc_processor::{CrateCollection, GlobalItemId};

/// What C type tag to use when referring to a user-defined type by name.
enum CTypeTag {
    Struct,
    Union,
    Enum,
    /// No tag prefix — the type name is typedef'd to an integer type.
    IntTypedef,
}

/// Build a lookup from type name → C type tag.
fn build_type_tag_map(type_defs: &[CTypeDefinition]) -> HashMap<String, CTypeTag> {
    let mut map = HashMap::new();
    for def in type_defs {
        let tag = match &def.kind {
            CTypeKind::OpaqueStruct | CTypeKind::Struct(_) => CTypeTag::Struct,
            CTypeKind::OpaqueUnion | CTypeKind::Union(_) => CTypeTag::Union,
            CTypeKind::FieldlessEnum(e) => match &e.repr {
                CEnumRepr::C => CTypeTag::Enum,
                CEnumRepr::Int { .. } => CTypeTag::IntTypedef,
            },
            CTypeKind::Typedef(_) => CTypeTag::IntTypedef,
            CTypeKind::TaggedUnion(t) => {
                if t.repr.is_repr_c() {
                    CTypeTag::Struct
                } else {
                    CTypeTag::Union
                }
            }
        };
        map.insert(def.name.clone(), tag);
    }
    map
}

/// Generate a C header from the resolved functions, writing to `out`.
///
/// The output order follows cbindgen conventions:
/// header, include guard/pragma once, autogen warning, includes,
/// after_includes, cpp_compat open, declarations, cpp_compat close,
/// include guard close, trailer.
#[allow(clippy::too_many_arguments)]
pub fn generate_c_header<I: CrateIndexer>(
    config: &CConfig,
    type_defs: &[CTypeDefinition],
    constants: &[ConstantItem],
    assoc_constants: &[(String, Vec<ConstantItem>)],
    functions: &[FreeFunction],
    statics: &[StaticItem],
    collection: &CrateCollection<I>,
    out: &mut String,
) {
    let common = &config.common;

    // Header (verbatim text at top of file).
    if let Some(ref header) = common.header {
        out.push_str(header);
        out.push('\n');
    }

    // Include guard or pragma once.
    if let Some(ref guard) = common.include_guard {
        writeln!(out, "#ifndef {guard}").unwrap();
        writeln!(out, "#define {guard}").unwrap();
        out.push('\n');
    } else if common.pragma_once {
        out.push_str("#pragma once\n\n");
    }

    // Autogen warning.
    if let Some(ref warning) = common.autogen_warning {
        out.push_str(warning);
        out.push_str("\n\n");
    }

    // Standard includes.
    if !common.no_includes {
        out.push_str("#include <stdarg.h>\n");
        out.push_str("#include <stdbool.h>\n");
        out.push_str("#include <stdint.h>\n");
        out.push_str("#include <stdlib.h>\n");
    }

    // Additional system includes.
    for inc in &common.sys_includes {
        writeln!(out, "#include <{inc}>").unwrap();
    }

    // Additional user includes.
    for inc in &common.includes {
        writeln!(out, "#include \"{inc}\"").unwrap();
    }

    // After includes (verbatim text).
    if let Some(ref after) = common.after_includes {
        out.push_str(after);
        out.push('\n');
    }

    // Build type tag map for correct type references in declarations.
    let type_tags = build_type_tag_map(type_defs);

    // Type forward declarations.
    if !type_defs.is_empty() {
        out.push('\n');
        write_c_type_definitions(
            type_defs,
            assoc_constants,
            &config.style,
            config.cpp_compat,
            &type_tags,
            common,
            collection,
            out,
        );
    }

    // Constants as #define macros (after types, before extern "C" block).
    for c in constants {
        out.push('\n');
        let docs = lookup_docs(Some(&c.rustdoc_id), collection);
        write_doc_comment(docs.as_deref(), common, out);
        writeln!(out, "#define {} {}", c.name, c.value).unwrap();
    }

    let has_declarations = !functions.is_empty() || !statics.is_empty();

    // cpp_compat open (only if there are declarations to wrap).
    if config.cpp_compat && has_declarations {
        out.push('\n');
        out.push_str("#ifdef __cplusplus\n");
        out.push_str("extern \"C\" {\n");
        out.push_str("#endif // __cplusplus\n");
    }

    // Static declarations (before functions, matching cbindgen order).
    for s in statics.iter() {
        out.push('\n');
        let docs = lookup_docs(Some(&s.rustdoc_id), collection);
        write_doc_comment(docs.as_deref(), common, out);
        write_c_static_decl(s, &config.style, &type_tags, out);
        out.push('\n');
    }

    // Function declarations.
    for func in functions.iter() {
        out.push('\n');
        let docs = lookup_docs(func.source_coordinates.as_ref(), collection);
        write_doc_comment(docs.as_deref(), common, out);
        write_c_function_decl(func, &config.style, &type_tags, out);
        out.push('\n');
    }

    // cpp_compat close.
    if config.cpp_compat && has_declarations {
        out.push_str("\n#ifdef __cplusplus\n");
        out.push_str("}  // extern \"C\"\n");
        out.push_str("#endif  // __cplusplus\n");
    }

    // Include guard close.
    if let Some(ref guard) = common.include_guard {
        out.push('\n');
        writeln!(out, "#endif  /* {guard} */").unwrap();
    }

    // Trailer (verbatim text at bottom of file).
    if let Some(ref trailer) = common.trailer {
        out.push_str(trailer);
        out.push('\n');
    }
}

fn lookup_docs<I: CrateIndexer>(
    id: Option<&GlobalItemId>,
    collection: &CrateCollection<I>,
) -> Option<String> {
    let id = id?;
    let item = collection.get_item_by_global_type_id(id);
    item.docs.clone()
}

fn write_doc_comment(docs: Option<&str>, config: &CommonConfig, out: &mut String) {
    if !config.documentation {
        return;
    }
    let Some(docs) = docs else {
        return;
    };
    if docs.is_empty() {
        return;
    }

    let text = match config.documentation_length {
        DocumentationLength::Full => docs,
        DocumentationLength::Short => {
            // First line only (cbindgen's "short" behaviour).
            docs.lines().next().unwrap_or(docs)
        }
    };

    // Trim trailing whitespace-only lines from doc text.
    let text = text.trim_end();

    // Detect whether the original text had a trailing blank line:
    // ends with '\n' and the char before it is not whitespace (i.e. content\n, not ws\n).
    let has_trailing_blank = docs.len() >= 2 && {
        let bytes = docs.as_bytes();
        bytes[bytes.len() - 1] == b'\n' && !bytes[bytes.len() - 2].is_ascii_whitespace()
    };

    let use_line_comments = matches!(
        config.documentation_style,
        DocumentationStyle::C99 | DocumentationStyle::Cxx
    );

    if use_line_comments {
        for line in text.lines() {
            if line.is_empty() {
                out.push_str("//\n");
            } else {
                writeln!(out, "// {line}").unwrap();
            }
        }
        if has_trailing_blank {
            out.push_str("//\n");
        }
    } else {
        // C-style block comment: /** ... */
        out.push_str("/**\n");
        for line in text.lines() {
            if line.is_empty() {
                out.push_str(" *\n");
            } else {
                writeln!(out, " * {line}").unwrap();
            }
        }
        if has_trailing_blank {
            out.push_str(" *\n");
        }
        out.push_str(" */\n");
    }
}

fn exported_static_name(s: &StaticItem) -> &str {
    s.symbol_name.as_deref().unwrap_or(&s.name)
}

fn write_c_static_decl(
    s: &StaticItem,
    style: &Style,
    type_tags: &HashMap<String, CTypeTag>,
    out: &mut String,
) {
    out.push_str("extern ");
    if !s.is_mutable && !is_const_pointer(&s.type_) {
        out.push_str("const ");
    }
    write_c_decl(&s.type_, exported_static_name(s), style, type_tags, out);
    out.push(';');
}

/// Write a C declaration in cdecl style: handles arrays, function pointers,
/// and pointer-to-function-pointers by placing the name inside the declarator.
fn write_c_decl(
    ty: &Type,
    name: &str,
    style: &Style,
    type_tags: &HashMap<String, CTypeTag>,
    out: &mut String,
) {
    if let Type::Array(a) = ty {
        write_c_type(&a.element_type, style, type_tags, out);
        write!(out, " {name}[{}]", a.len).unwrap();
    } else if let Some((fp, depth)) = fn_ptr_through_pointers(ty) {
        let declarator = format!("{}{name}", "*".repeat(depth));
        write_fn_ptr_decl(fp, &declarator, style, type_tags, out);
    } else {
        let mut type_buf = String::new();
        write_c_type(ty, style, type_tags, &mut type_buf);
        if type_buf.ends_with('*') {
            write!(out, "{type_buf}{name}").unwrap();
        } else {
            write!(out, "{type_buf} {name}").unwrap();
        }
    }
}

/// Returns `true` if the type is a `*const T` raw pointer.
fn is_const_pointer(ty: &Type) -> bool {
    matches!(ty, Type::RawPointer(p) if !p.is_mutable)
}

fn write_c_function_decl(
    func: &FreeFunction,
    style: &Style,
    type_tags: &HashMap<String, CTypeTag>,
    out: &mut String,
) {
    let name = func
        .header
        .symbol_name
        .as_deref()
        .unwrap_or(&func.path.function_name);

    // Return type.
    let mut ret_buf = String::new();
    match &func.header.output {
        None => ret_buf.push_str("void"),
        Some(ty) if is_void(ty) => ret_buf.push_str("void"),
        Some(ty) => write_c_type(ty, style, type_tags, &mut ret_buf),
    }
    if ret_buf.ends_with('*') {
        write!(out, "{ret_buf}{name}(").unwrap();
    } else {
        write!(out, "{ret_buf} {name}(").unwrap();
    }

    // Parameters.
    if func.header.inputs.is_empty() && !func.header.is_c_variadic {
        out.push_str("void");
    } else {
        for (i, input) in func.header.inputs.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            write_c_param(&input.type_, input.name.as_str(), style, type_tags, out);
        }
        if func.header.is_c_variadic {
            if !func.header.inputs.is_empty() {
                out.push_str(", ");
            }
            out.push_str("...");
        }
    }

    out.push_str(");");
}

fn write_c_param(
    ty: &Type,
    name: &str,
    style: &Style,
    type_tags: &HashMap<String, CTypeTag>,
    out: &mut String,
) {
    if let Some((fp, depth)) = fn_ptr_through_pointers(ty) {
        let declarator = format!("{}{name}", "*".repeat(depth));
        write_fn_ptr_decl(fp, &declarator, style, type_tags, out);
    } else {
        let mut type_buf = String::new();
        write_c_type(ty, style, type_tags, &mut type_buf);
        if type_buf.ends_with('*') {
            write!(out, "{type_buf}{name}").unwrap();
        } else {
            write!(out, "{type_buf} {name}").unwrap();
        }
    }
}

fn write_c_type(ty: &Type, style: &Style, type_tags: &HashMap<String, CTypeTag>, out: &mut String) {
    match ty {
        Type::ScalarPrimitive(p) => out.push_str(scalar_to_c(p)),
        Type::RawPointer(ptr) => {
            if let Some((fp, depth)) = fn_ptr_through_pointers(&ptr.inner) {
                let declarator = "*".repeat(depth + 1);
                write_fn_ptr_decl(fp, &declarator, style, type_tags, out);
            } else if ptr.is_mutable {
                write_c_type(&ptr.inner, style, type_tags, out);
                out.push_str(" *");
            } else {
                out.push_str("const ");
                write_c_type(&ptr.inner, style, type_tags, out);
                out.push_str(" *");
            }
        }
        Type::Tuple(t) if t.elements.is_empty() => out.push_str("void"),
        Type::Array(a) => {
            write_c_type(&a.element_type, style, type_tags, out);
            write!(out, "[{}]", a.len).unwrap();
        }
        Type::Reference(r) => {
            if r.is_mutable {
                write_c_type(&r.inner, style, type_tags, out);
                out.push_str(" *");
            } else {
                out.push_str("const ");
                write_c_type(&r.inner, style, type_tags, out);
                out.push_str(" *");
            }
        }
        Type::FunctionPointer(fp) => {
            write_fn_ptr_decl(fp, "", style, type_tags, out);
        }
        Type::Path(_) | Type::TypeAlias(_) => {
            let name = c_type_name(ty);
            match style {
                Style::Tag | Style::Both => {
                    let tag = type_tags.get(&name);
                    match tag {
                        Some(CTypeTag::Enum) => write!(out, "enum {name}").unwrap(),
                        Some(CTypeTag::Union) => write!(out, "union {name}").unwrap(),
                        Some(CTypeTag::IntTypedef) => out.push_str(&name),
                        Some(CTypeTag::Struct) | None => write!(out, "struct {name}").unwrap(),
                    }
                }
                Style::Type => out.push_str(&name),
            }
        }
        Type::Tuple(_) | Type::Slice(_) | Type::Generic(_) => {
            unreachable!("unsupported type in C codegen: {ty:?}")
        }
    }
}

/// Check if a type is a function pointer possibly wrapped in raw pointer layers.
/// Returns the inner function pointer and the pointer depth (0 for bare fn ptr).
fn fn_ptr_through_pointers(ty: &Type) -> Option<(&FunctionPointer, usize)> {
    match ty {
        Type::FunctionPointer(fp) => Some((fp, 0)),
        Type::RawPointer(p) => {
            let (fp, depth) = fn_ptr_through_pointers(&p.inner)?;
            Some((fp, depth + 1))
        }
        _ => None,
    }
}

/// Write a C function pointer declaration.
///
/// `declarator` goes between `(*` and `)` — it's the name for named declarations,
/// extra `*`s for pointer-to-fn-ptr, or empty for unnamed.
fn write_fn_ptr_decl(
    fp: &FunctionPointer,
    declarator: &str,
    style: &Style,
    type_tags: &HashMap<String, CTypeTag>,
    out: &mut String,
) {
    // Return type.
    let mut ret_buf = String::new();
    match &fp.output {
        None => ret_buf.push_str("void"),
        Some(ty) if is_void(ty) => ret_buf.push_str("void"),
        Some(ty) => write_c_type(ty, style, type_tags, &mut ret_buf),
    }

    if ret_buf.ends_with('*') {
        write!(out, "{ret_buf}(*{declarator})(").unwrap();
    } else {
        write!(out, "{ret_buf} (*{declarator})(").unwrap();
    }

    // Parameters.
    if fp.inputs.is_empty() {
        out.push_str("void");
    } else {
        for (i, input) in fp.inputs.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            write_c_type(input, style, type_tags, out);
        }
    }
    out.push(')');
}

#[allow(clippy::too_many_arguments)]
fn write_c_type_definitions<I: CrateIndexer>(
    type_defs: &[CTypeDefinition],
    assoc_constants: &[(String, Vec<ConstantItem>)],
    style: &Style,
    cpp_compat: bool,
    type_tags: &HashMap<String, CTypeTag>,
    config: &CommonConfig,
    collection: &CrateCollection<I>,
    out: &mut String,
) {
    // Partition into categories for ordering: fieldless enums, opaques, then structs + tagged unions.
    let fieldless_enum_defs: Vec<_> = type_defs
        .iter()
        .filter(|d| matches!(d.kind, CTypeKind::FieldlessEnum(_)))
        .collect();
    let opaque_defs: Vec<_> = type_defs
        .iter()
        .filter(|d| matches!(d.kind, CTypeKind::OpaqueStruct | CTypeKind::OpaqueUnion))
        .collect();
    let compound_defs: Vec<_> = type_defs
        .iter()
        .filter(|d| {
            matches!(
                d.kind,
                CTypeKind::Struct(_)
                    | CTypeKind::Union(_)
                    | CTypeKind::TaggedUnion(_)
                    | CTypeKind::Typedef(_)
            )
        })
        .collect();

    // Build a lookup from type name → associated constants.
    let assoc_map: HashMap<&str, &Vec<ConstantItem>> = assoc_constants
        .iter()
        .map(|(name, consts)| (name.as_str(), consts))
        .collect();

    // Fieldless enums first.
    for (i, def) in fieldless_enum_defs.iter().enumerate() {
        let CTypeKind::FieldlessEnum(ref enum_def) = def.kind else {
            unreachable!();
        };
        let docs = lookup_docs(def.rustdoc_id.as_ref(), collection);
        write_doc_comment(docs.as_deref(), config, out);
        write_c_fieldless_enum(&def.name, enum_def, style, cpp_compat, out);
        write_assoc_constants_for_type(&def.name, &assoc_map, collection, config, out);
        if i + 1 < fieldless_enum_defs.len() {
            out.push('\n');
        }
    }

    // Blank line between sections.
    if !fieldless_enum_defs.is_empty() && (!opaque_defs.is_empty() || !compound_defs.is_empty()) {
        out.push('\n');
    }

    // Opaques.
    for (i, def) in opaque_defs.iter().enumerate() {
        let name = &def.name;
        let docs = lookup_docs(def.rustdoc_id.as_ref(), collection);
        write_doc_comment(docs.as_deref(), config, out);
        let tag = if matches!(def.kind, CTypeKind::OpaqueUnion) {
            "union"
        } else {
            "struct"
        };
        match style {
            Style::Tag => writeln!(out, "{tag} {name};").unwrap(),
            Style::Type | Style::Both => writeln!(out, "typedef {tag} {name} {name};").unwrap(),
        }
        write_assoc_constants_for_type(name, &assoc_map, collection, config, out);
        if i + 1 < opaque_defs.len() {
            out.push('\n');
        }
    }

    // Blank line between opaques and compounds.
    if !opaque_defs.is_empty() && !compound_defs.is_empty() {
        out.push('\n');
    }

    // For Plain (Type) style, emit forward declarations only for compounds
    // that are referenced by pointer before their definition (self-referential
    // types, corecursive pointer back-references, etc.).
    if matches!(style, Style::Type) && !compound_defs.is_empty() {
        let forward_decls = compute_needed_forward_decls(&compound_defs);
        if !forward_decls.is_empty() {
            for def in &compound_defs {
                if forward_decls.contains(def.name.as_str()) {
                    let name = &def.name;
                    match &def.kind {
                        CTypeKind::Union(_)
                        | CTypeKind::TaggedUnion(CTaggedUnionDef {
                            repr: CEnumRepr::Int { .. },
                            ..
                        }) => {
                            writeln!(out, "typedef union {name} {name};").unwrap();
                        }
                        _ => {
                            writeln!(out, "typedef struct {name} {name};").unwrap();
                        }
                    }
                }
            }
            out.push('\n');
        }
    }

    // Structs and tagged unions.
    let forward_declared: HashSet<&str> = if matches!(style, Style::Type) {
        compute_needed_forward_decls(&compound_defs)
    } else {
        HashSet::new()
    };
    for (i, def) in compound_defs.iter().enumerate() {
        let docs = lookup_docs(def.rustdoc_id.as_ref(), collection);
        write_doc_comment(docs.as_deref(), config, out);
        let has_fwd_decl = forward_declared.contains(def.name.as_str());
        match &def.kind {
            CTypeKind::Struct(struct_def) => {
                write_c_struct_definition(
                    &def.name,
                    struct_def,
                    style,
                    has_fwd_decl,
                    type_tags,
                    out,
                );
            }
            CTypeKind::Union(union_def) => {
                write_c_union_definition(&def.name, union_def, style, has_fwd_decl, type_tags, out);
            }
            CTypeKind::TaggedUnion(tagged_def) => {
                write_c_tagged_union(
                    &def.name,
                    tagged_def,
                    style,
                    has_fwd_decl,
                    cpp_compat,
                    type_tags,
                    out,
                );
            }
            CTypeKind::Typedef(typedef_def) => {
                write_c_typedef(&def.name, typedef_def, style, type_tags, out);
            }
            _ => unreachable!(),
        }
        write_assoc_constants_for_type(&def.name, &assoc_map, collection, config, out);
        if i + 1 < compound_defs.len() {
            out.push('\n');
        }
    }
}

/// Emit associated constants for a type, if any exist.
fn write_assoc_constants_for_type<I: CrateIndexer>(
    type_name: &str,
    assoc_map: &HashMap<&str, &Vec<ConstantItem>>,
    collection: &CrateCollection<I>,
    config: &CommonConfig,
    out: &mut String,
) {
    if let Some(constants) = assoc_map.get(type_name) {
        for c in *constants {
            let docs = lookup_docs(Some(&c.rustdoc_id), collection);
            write_doc_comment(docs.as_deref(), config, out);
            writeln!(out, "#define {} {}", c.name, c.value).unwrap();
        }
    }
}

/// Emit a full C struct definition with fields.
fn write_c_struct_definition(
    name: &str,
    def: &CStructDef,
    style: &Style,
    has_fwd_decl: bool,
    type_tags: &HashMap<String, CTypeTag>,
    out: &mut String,
) {
    match style {
        Style::Type if has_fwd_decl => {
            // typedef already emitted as a forward declaration.
            writeln!(out, "struct {name} {{").unwrap();
            if def.fields.is_empty() {
                writeln!(out).unwrap();
            } else {
                for field in &def.fields {
                    out.push_str("  ");
                    write_c_decl(&field.type_, field.name.as_str(), style, type_tags, out);
                    writeln!(out, ";").unwrap();
                }
            }
            writeln!(out, "}};").unwrap();
        }
        Style::Type => {
            writeln!(out, "typedef struct {{").unwrap();
            if def.fields.is_empty() {
                writeln!(out).unwrap();
            } else {
                for field in &def.fields {
                    out.push_str("  ");
                    write_c_decl(&field.type_, field.name.as_str(), style, type_tags, out);
                    writeln!(out, ";").unwrap();
                }
            }
            writeln!(out, "}} {name};").unwrap();
        }
        Style::Tag => {
            writeln!(out, "struct {name} {{").unwrap();
            if def.fields.is_empty() {
                writeln!(out).unwrap();
            } else {
                for field in &def.fields {
                    out.push_str("  ");
                    write_c_decl(
                        &field.type_,
                        field.name.as_str(),
                        &Style::Tag,
                        type_tags,
                        out,
                    );
                    writeln!(out, ";").unwrap();
                }
            }
            writeln!(out, "}};").unwrap();
        }
        Style::Both => {
            writeln!(out, "typedef struct {name} {{").unwrap();
            if def.fields.is_empty() {
                writeln!(out).unwrap();
            } else {
                for field in &def.fields {
                    out.push_str("  ");
                    write_c_decl(&field.type_, field.name.as_str(), style, type_tags, out);
                    writeln!(out, ";").unwrap();
                }
            }
            writeln!(out, "}} {name};").unwrap();
        }
    }
}

/// Emit a full C union definition with fields.
fn write_c_union_definition(
    name: &str,
    def: &CUnionDef,
    style: &Style,
    has_fwd_decl: bool,
    type_tags: &HashMap<String, CTypeTag>,
    out: &mut String,
) {
    match style {
        Style::Type if has_fwd_decl => {
            writeln!(out, "union {name} {{").unwrap();
            if def.fields.is_empty() {
                writeln!(out).unwrap();
            } else {
                for field in &def.fields {
                    out.push_str("  ");
                    write_c_decl(&field.type_, field.name.as_str(), style, type_tags, out);
                    writeln!(out, ";").unwrap();
                }
            }
            writeln!(out, "}};").unwrap();
        }
        Style::Type => {
            writeln!(out, "typedef union {{").unwrap();
            if def.fields.is_empty() {
                writeln!(out).unwrap();
            } else {
                for field in &def.fields {
                    out.push_str("  ");
                    write_c_decl(&field.type_, field.name.as_str(), style, type_tags, out);
                    writeln!(out, ";").unwrap();
                }
            }
            writeln!(out, "}} {name};").unwrap();
        }
        Style::Tag => {
            writeln!(out, "union {name} {{").unwrap();
            if def.fields.is_empty() {
                writeln!(out).unwrap();
            } else {
                for field in &def.fields {
                    out.push_str("  ");
                    write_c_decl(&field.type_, field.name.as_str(), &Style::Tag, type_tags, out);
                    writeln!(out, ";").unwrap();
                }
            }
            writeln!(out, "}};").unwrap();
        }
        Style::Both => {
            writeln!(out, "typedef union {name} {{").unwrap();
            if def.fields.is_empty() {
                writeln!(out).unwrap();
            } else {
                for field in &def.fields {
                    out.push_str("  ");
                    write_c_decl(&field.type_, field.name.as_str(), style, type_tags, out);
                    writeln!(out, ";").unwrap();
                }
            }
            writeln!(out, "}} {name};").unwrap();
        }
    }
}

/// Emit a fieldless C enum.
fn write_c_fieldless_enum(
    name: &str,
    def: &CFieldlessEnumDef,
    style: &Style,
    cpp_compat: bool,
    out: &mut String,
) {
    match &def.repr {
        CEnumRepr::Int { int_type, .. } => {
            let c_int = scalar_to_c(&int_type.to_scalar_primitive());
            if cpp_compat {
                writeln!(out, "enum {name}").unwrap();
                writeln!(out, "#ifdef __cplusplus").unwrap();
                writeln!(out, "  : {c_int}").unwrap();
                writeln!(out, "#endif // __cplusplus").unwrap();
                writeln!(out, " {{").unwrap();
                write_enum_variant_list(&def.variants, out);
                writeln!(out, "}};").unwrap();
                writeln!(out, "#ifndef __cplusplus").unwrap();
                writeln!(out, "typedef {c_int} {name};").unwrap();
                writeln!(out, "#endif // __cplusplus").unwrap();
            } else {
                writeln!(out, "enum {name} {{").unwrap();
                write_enum_variant_list(&def.variants, out);
                writeln!(out, "}};").unwrap();
                writeln!(out, "typedef {c_int} {name};").unwrap();
            }
        }
        CEnumRepr::C => match style {
            Style::Type => {
                writeln!(out, "typedef enum {{").unwrap();
                write_enum_variant_list(&def.variants, out);
                writeln!(out, "}} {name};").unwrap();
            }
            Style::Tag => {
                writeln!(out, "enum {name} {{").unwrap();
                write_enum_variant_list(&def.variants, out);
                writeln!(out, "}};").unwrap();
            }
            Style::Both => {
                writeln!(out, "typedef enum {name} {{").unwrap();
                write_enum_variant_list(&def.variants, out);
                writeln!(out, "}} {name};").unwrap();
            }
        },
    }
}

/// Write the inner variant list of an enum (indented, with trailing commas).
fn write_enum_variant_list(variants: &[crate::analysis::CEnumVariant], out: &mut String) {
    for (i, variant) in variants.iter().enumerate() {
        out.push_str("  ");
        out.push_str(variant.name.as_str());
        if let Some(ref disc) = variant.discriminant {
            write!(out, " = {disc}").unwrap();
        }
        out.push(',');
        if i + 1 < variants.len() {
            out.push('\n');
        }
    }
    out.push('\n');
}

/// Emit a tagged union (enum with data variants).
fn write_c_tagged_union(
    name: &str,
    def: &CTaggedUnionDef,
    style: &Style,
    has_fwd_decl: bool,
    cpp_compat: bool,
    type_tags: &HashMap<String, CTypeTag>,
    out: &mut String,
) {
    let tag_name = format!("{name}_Tag");

    // 1. Emit the tag enum.
    let tag_variants: Vec<crate::analysis::CEnumVariant> = def
        .variants
        .iter()
        .map(|v| {
            let variant_name = if def.prefix_with_name {
                format!("{}_{}", name, v.name)
            } else {
                v.name.clone()
            };
            crate::analysis::CEnumVariant {
                name: CIdentifier::new(variant_name),
                discriminant: None,
            }
        })
        .collect();

    let tag_repr = match &def.repr {
        CEnumRepr::C => CEnumRepr::C,
        CEnumRepr::Int { int_type, .. } => CEnumRepr::Int {
            is_repr_c: false,
            int_type: *int_type,
        },
    };

    let tag_enum_def = CFieldlessEnumDef {
        repr: tag_repr,
        variants: tag_variants,
    };
    write_c_fieldless_enum(&tag_name, &tag_enum_def, style, cpp_compat, out);
    out.push('\n');

    // Determine the tag type string for use in fields.
    let tag_type_str = match &def.repr {
        CEnumRepr::Int { .. } => tag_name.clone(),
        CEnumRepr::C => match style {
            Style::Tag => format!("enum {tag_name}"),
            Style::Type | Style::Both => tag_name.clone(),
        },
    };

    // 2. Emit body structs for multi-field variants.
    for variant in &def.variants {
        let Some(ref body) = variant.body else {
            continue;
        };
        if body.fields.len() < 2 {
            continue;
        }
        let body_name = if def.prefix_with_name {
            format!("{name}_{}_Body", variant.name)
        } else {
            format!("{}_Body", variant.name)
        };

        // For repr(uN) body structs, the tag is the first field.
        let include_tag_field = !def.repr.is_repr_c();

        match style {
            Style::Type => {
                writeln!(out, "typedef struct {{").unwrap();
                if include_tag_field {
                    writeln!(out, "  {tag_type_str} tag;").unwrap();
                }
                for field in &body.fields {
                    out.push_str("  ");
                    write_c_decl(&field.type_, field.name.as_str(), style, type_tags, out);
                    writeln!(out, ";").unwrap();
                }
                writeln!(out, "}} {body_name};").unwrap();
            }
            Style::Tag => {
                writeln!(out, "struct {body_name} {{").unwrap();
                if include_tag_field {
                    writeln!(out, "  {tag_type_str} tag;").unwrap();
                }
                for field in &body.fields {
                    out.push_str("  ");
                    write_c_decl(
                        &field.type_,
                        field.name.as_str(),
                        &Style::Tag,
                        type_tags,
                        out,
                    );
                    writeln!(out, ";").unwrap();
                }
                writeln!(out, "}};").unwrap();
            }
            Style::Both => {
                writeln!(out, "typedef struct {body_name} {{").unwrap();
                if include_tag_field {
                    writeln!(out, "  {tag_type_str} tag;").unwrap();
                }
                for field in &body.fields {
                    out.push_str("  ");
                    write_c_decl(&field.type_, field.name.as_str(), style, type_tags, out);
                    writeln!(out, ";").unwrap();
                }
                writeln!(out, "}} {body_name};").unwrap();
            }
        }
        out.push('\n');
    }

    // 3. Emit the outer container.
    if def.repr.is_repr_c() {
        // repr(C) or repr(C, uN) → struct with tag + anonymous union
        write_tagged_union_repr_c(
            name,
            def,
            style,
            has_fwd_decl,
            type_tags,
            &tag_type_str,
            out,
        );
    } else {
        // repr(uN) → union with tag + anonymous struct members
        write_tagged_union_repr_int(
            name,
            def,
            style,
            has_fwd_decl,
            type_tags,
            &tag_type_str,
            out,
        );
    }
}

/// Emit the outer container for a repr(C) tagged union.
fn write_tagged_union_repr_c(
    name: &str,
    def: &CTaggedUnionDef,
    style: &Style,
    has_fwd_decl: bool,
    type_tags: &HashMap<String, CTypeTag>,
    tag_type_str: &str,
    out: &mut String,
) {
    let body_prefix = if def.prefix_with_name {
        format!("{name}_")
    } else {
        String::new()
    };

    // Check if there are any non-unit variants that need a union.
    let has_data_variants = def.variants.iter().any(|v| v.body.is_some());

    match (style, has_fwd_decl) {
        (Style::Type, true) => writeln!(out, "struct {name} {{").unwrap(),
        (Style::Type, false) => out.push_str("typedef struct {\n"),
        (Style::Tag, _) => writeln!(out, "struct {name} {{").unwrap(),
        (Style::Both, _) => writeln!(out, "typedef struct {name} {{").unwrap(),
    }
    writeln!(out, "  {tag_type_str} tag;").unwrap();

    if has_data_variants {
        writeln!(out, "  union {{").unwrap();
        for variant in &def.variants {
            let Some(ref body) = variant.body else {
                continue;
            };
            let field_name = CIdentifier::new(variant.name.to_lowercase());
            if body.fields.len() == 1 {
                // Single-field: anonymous struct with lowercased variant name
                writeln!(out, "    struct {{").unwrap();
                let field = &body.fields[0];
                out.push_str("      ");
                write_c_decl(&field.type_, field_name.as_str(), style, type_tags, out);
                writeln!(out, ";").unwrap();
                writeln!(out, "    }};").unwrap();
            } else {
                // Multi-field: reference to _Body struct
                let body_name = format!("{body_prefix}{}_Body", variant.name);
                match style {
                    Style::Tag | Style::Both => {
                        writeln!(out, "    struct {body_name} {field_name};").unwrap()
                    }
                    _ => writeln!(out, "    {body_name} {field_name};").unwrap(),
                }
            }
        }
        writeln!(out, "  }};").unwrap();
    }

    match (style, has_fwd_decl) {
        (Style::Type, true) | (Style::Tag, _) => writeln!(out, "}};").unwrap(),
        _ => writeln!(out, "}} {name};").unwrap(),
    }
}

/// Emit the outer container for a repr(uN) tagged union.
fn write_tagged_union_repr_int(
    name: &str,
    def: &CTaggedUnionDef,
    style: &Style,
    has_fwd_decl: bool,
    type_tags: &HashMap<String, CTypeTag>,
    tag_type_str: &str,
    out: &mut String,
) {
    match (style, has_fwd_decl) {
        (Style::Type, true) => writeln!(out, "union {name} {{").unwrap(),
        (Style::Type, false) => out.push_str("typedef union {\n"),
        (Style::Tag, _) => writeln!(out, "union {name} {{").unwrap(),
        (Style::Both, _) => writeln!(out, "typedef union {name} {{").unwrap(),
    }
    writeln!(out, "  {tag_type_str} tag;").unwrap();

    let body_prefix = if def.prefix_with_name {
        format!("{name}_")
    } else {
        String::new()
    };

    for variant in &def.variants {
        let Some(ref body) = variant.body else {
            continue;
        };
        let field_name = CIdentifier::new(variant.name.to_lowercase());
        if body.fields.len() == 1 {
            // Single-field tuple variant: anonymous struct with tag + field
            let field = &body.fields[0];
            writeln!(out, "  struct {{").unwrap();
            writeln!(out, "    {tag_type_str} {field_name}_tag;").unwrap();
            out.push_str("    ");
            write_c_decl(&field.type_, field_name.as_str(), style, type_tags, out);
            writeln!(out, ";").unwrap();
            writeln!(out, "  }};").unwrap();
        } else {
            // Multi-field: reference to _Body struct
            let body_name = format!("{body_prefix}{}_Body", variant.name);
            match style {
                Style::Tag | Style::Both => {
                    writeln!(out, "  struct {body_name} {field_name};").unwrap()
                }
                _ => writeln!(out, "  {body_name} {field_name};").unwrap(),
            }
        }
    }

    match (style, has_fwd_decl) {
        (Style::Type, true) | (Style::Tag, _) => writeln!(out, "}};").unwrap(),
        _ => writeln!(out, "}} {name};").unwrap(),
    }
}

/// Determine which compound types need a forward `typedef` declaration
/// in Plain (Type) style.
///
/// A forward declaration is needed when a compound type is referenced via
/// pointer from a compound that appears earlier in the emission order (the
/// bare typedef name isn't available yet). This covers self-referential types
/// and corecursive pointer back-references.
fn compute_needed_forward_decls<'a>(compound_defs: &'a [&CTypeDefinition]) -> HashSet<&'a str> {
    let all_compound_names: HashSet<&str> = compound_defs.iter().map(|d| d.name.as_str()).collect();

    // Track which types have been "defined" as we walk the list in order.
    let mut defined: HashSet<&str> = HashSet::new();
    let mut need_fwd: HashSet<&str> = HashSet::new();

    for def in compound_defs {
        // Collect all pointer-target type names from this compound's fields.
        let ptr_refs = pointer_referenced_types(def);
        for name in &ptr_refs {
            // If this name is a compound that hasn't been defined yet
            // (or is the current type itself), it needs a forward declaration.
            if all_compound_names.contains(name.as_str()) && !defined.contains(name.as_str()) {
                need_fwd.insert(
                    compound_defs
                        .iter()
                        .find(|d| d.name == *name)
                        .unwrap()
                        .name
                        .as_str(),
                );
            }
        }
        defined.insert(def.name.as_str());
    }

    need_fwd
}

/// Collect all type names that appear behind a pointer/reference in a compound's fields.
fn pointer_referenced_types(def: &CTypeDefinition) -> Vec<String> {
    let mut refs = Vec::new();
    match &def.kind {
        CTypeKind::Struct(s) => {
            for field in &s.fields {
                collect_pointer_targets(&field.type_, &mut refs);
            }
        }
        CTypeKind::Union(u) => {
            for field in &u.fields {
                collect_pointer_targets(&field.type_, &mut refs);
            }
        }
        CTypeKind::TaggedUnion(t) => {
            for variant in &t.variants {
                if let Some(ref body) = variant.body {
                    for field in &body.fields {
                        collect_pointer_targets(&field.type_, &mut refs);
                    }
                }
            }
        }
        CTypeKind::Typedef(_) => {}
        _ => {}
    }
    refs
}

/// Walk a type tree and collect the C names of types found behind pointers/references.
fn collect_pointer_targets(ty: &Type, refs: &mut Vec<String>) {
    match ty {
        Type::RawPointer(p) => collect_by_value_names(&p.inner, refs),
        Type::Reference(r) => collect_by_value_names(&r.inner, refs),
        Type::Array(a) => collect_pointer_targets(&a.element_type, refs),
        // By-value Path types are not pointer targets.
        _ => {}
    }
}

/// Collect the C name of any Path type found by value (recursing into arrays).
fn collect_by_value_names(ty: &Type, refs: &mut Vec<String>) {
    match ty {
        Type::Path(_) | Type::TypeAlias(_) => refs.push(c_type_name(ty)),
        Type::Array(a) => collect_by_value_names(&a.element_type, refs),
        Type::RawPointer(p) => collect_by_value_names(&p.inner, refs),
        Type::Reference(r) => collect_by_value_names(&r.inner, refs),
        _ => {}
    }
}

/// Emit a typedef: `typedef <inner> <name>;`.
fn write_c_typedef(
    name: &str,
    def: &CTypedefDef,
    style: &Style,
    type_tags: &HashMap<String, CTypeTag>,
    out: &mut String,
) {
    out.push_str("typedef ");
    write_c_decl(&def.inner, name, style, type_tags, out);
    writeln!(out, ";").unwrap();
}

fn scalar_to_c(p: &ScalarPrimitive) -> &'static str {
    match p {
        ScalarPrimitive::U8 => "uint8_t",
        ScalarPrimitive::U16 => "uint16_t",
        ScalarPrimitive::U32 => "uint32_t",
        ScalarPrimitive::U64 => "uint64_t",
        ScalarPrimitive::U128 => "__uint128_t",
        ScalarPrimitive::Usize => "uintptr_t",
        ScalarPrimitive::I8 => "int8_t",
        ScalarPrimitive::I16 => "int16_t",
        ScalarPrimitive::I32 => "int32_t",
        ScalarPrimitive::I64 => "int64_t",
        ScalarPrimitive::I128 => "__int128_t",
        ScalarPrimitive::Isize => "intptr_t",
        ScalarPrimitive::F32 => "float",
        ScalarPrimitive::F64 => "double",
        ScalarPrimitive::Bool => "bool",
        ScalarPrimitive::Char => "uint32_t",
        ScalarPrimitive::Str => "const char",
    }
}

fn is_void(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(t) if t.elements.is_empty())
}

/// Write a symbol file listing exported dynamic symbols in `{ sym; ... };` format.
pub fn write_symbol_file(symbols: &BTreeSet<String>, path: &Path) -> anyhow::Result<()> {
    let mut out = String::from("{\n");
    for sym in symbols {
        out.push_str(sym);
        out.push_str(";\n");
    }
    out.push_str("};\n");
    fs_err::write(path, &out)?;
    Ok(())
}
