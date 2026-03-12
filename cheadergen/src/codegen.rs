use std::collections::BTreeSet;
use std::fmt::Write;
use std::path::Path;

use crate::analysis::{CTypeDefinition, CTypeKind, c_type_name};
use crate::config::{CConfig, CommonConfig, DocumentationLength, DocumentationStyle, Style};
use crate::static_item::StaticItem;
use rustdoc_ir::{FreeFunction, ScalarPrimitive, Type};
use rustdoc_processor::crate_data::CrateItemIndex;
use rustdoc_types::Id;

/// Generate a C header from the resolved functions, writing to `out`.
///
/// The output order follows cbindgen conventions:
/// header, include guard/pragma once, autogen warning, includes,
/// after_includes, cpp_compat open, declarations, cpp_compat close,
/// include guard close, trailer.
pub fn generate_c_header(
    config: &CConfig,
    type_defs: &[CTypeDefinition],
    functions: &[FreeFunction],
    statics: &[StaticItem],
    index: &CrateItemIndex,
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

    // Type forward declarations.
    if !type_defs.is_empty() {
        out.push('\n');
        write_c_type_definitions(type_defs, &config.style, common, index, out);
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
        let docs = lookup_docs(Some(&s.rustdoc_id), index);
        write_doc_comment(docs.as_deref(), common, out);
        write_c_static_decl(s, &config.style, out);
        out.push('\n');
    }

    // Function declarations.
    for func in functions.iter() {
        out.push('\n');
        let docs = lookup_docs(
            func.source_coordinates
                .as_ref()
                .map(|c| &c.rustdoc_item_id),
            index,
        );
        write_doc_comment(docs.as_deref(), common, out);
        write_c_function_decl(func, &config.style, out);
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

fn lookup_docs(id: Option<&Id>, index: &CrateItemIndex) -> Option<String> {
    let id = id?;
    let item = index.get(id)?;
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
        out.push_str(" */\n");
    }
}

fn exported_static_name(s: &StaticItem) -> &str {
    s.symbol_name.as_deref().unwrap_or(&s.name)
}

fn write_c_static_decl(s: &StaticItem, style: &Style, out: &mut String) {
    // In "Both" style, declarations use the tag form (same as Tag).
    let decl_style = match style {
        Style::Both => Style::Tag,
        other => other.clone(),
    };

    out.push_str("extern ");
    if !s.is_mutable && !is_const_pointer(&s.type_) {
        out.push_str("const ");
    }
    write_c_decl(&s.type_, exported_static_name(s), &decl_style, out);
    out.push(';');
}

/// Write a C declaration in cdecl style: handles arrays by placing the name
/// between the element type and `[N]`, e.g. `char NAME[128]`.
fn write_c_decl(ty: &Type, name: &str, style: &Style, out: &mut String) {
    match ty {
        Type::Array(a) => {
            write_c_type(&a.element_type, style, out);
            write!(out, " {name}[{}]", a.len).unwrap();
        }
        _ => {
            write_c_type(ty, style, out);
            write!(out, " {name}").unwrap();
        }
    }
}

/// Returns `true` if the type is a `*const T` raw pointer.
fn is_const_pointer(ty: &Type) -> bool {
    matches!(ty, Type::RawPointer(p) if !p.is_mutable)
}

fn write_c_function_decl(func: &FreeFunction, style: &Style, out: &mut String) {
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
        Some(ty) => write_c_type(ty, style, &mut ret_buf),
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
            write_c_param(&input.type_, input.name.as_str(), style, out);
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

fn write_c_param(ty: &Type, name: &str, style: &Style, out: &mut String) {
    let mut type_buf = String::new();
    write_c_type(ty, style, &mut type_buf);
    if type_buf.ends_with('*') {
        write!(out, "{type_buf}{name}").unwrap();
    } else {
        write!(out, "{type_buf} {name}").unwrap();
    }
}

fn write_c_type(ty: &Type, style: &Style, out: &mut String) {
    match ty {
        Type::ScalarPrimitive(p) => out.push_str(scalar_to_c(p)),
        Type::RawPointer(ptr) => {
            if ptr.is_mutable {
                write_c_type(&ptr.inner, style, out);
                out.push_str(" *");
            } else {
                out.push_str("const ");
                write_c_type(&ptr.inner, style, out);
                out.push_str(" *");
            }
        }
        Type::Tuple(t) if t.elements.is_empty() => out.push_str("void"),
        Type::Array(a) => {
            // Arrays in parameter position decay to pointers in C, but for type
            // representation we write `type[N]`. This will need refinement for
            // parameters vs typedefs.
            write_c_type(&a.element_type, style, out);
            write!(out, "[{}]", a.len).unwrap();
        }
        Type::Reference(r) => {
            if r.is_mutable {
                write_c_type(&r.inner, style, out);
                out.push_str(" *");
            } else {
                out.push_str("const ");
                write_c_type(&r.inner, style, out);
                out.push_str(" *");
            }
        }
        Type::Path(_) => {
            let name = c_type_name(ty);
            match style {
                Style::Tag => write!(out, "struct {name}").unwrap(),
                Style::Type | Style::Both => out.push_str(&name),
            }
        }
        // These should have been rejected earlier.
        Type::Tuple(_)
        | Type::Slice(_)
        | Type::Generic(_) => {
            unreachable!("unsupported type in C codegen: {ty:?}")
        }
    }
}

fn write_c_type_definitions(
    type_defs: &[CTypeDefinition],
    style: &Style,
    config: &CommonConfig,
    index: &CrateItemIndex,
    out: &mut String,
) {
    // Emit forward declarations first (always valid ordering in C).
    let opaque_defs: Vec<_> = type_defs
        .iter()
        .filter(|d| matches!(d.kind, CTypeKind::Opaque))
        .collect();
    let struct_defs: Vec<_> = type_defs
        .iter()
        .filter(|d| matches!(d.kind, CTypeKind::Struct(_)))
        .collect();

    for (i, def) in opaque_defs.iter().enumerate() {
        let name = &def.name;
        let docs = lookup_docs(def.rustdoc_id.as_ref(), index);
        write_doc_comment(docs.as_deref(), config, out);
        match style {
            Style::Tag => writeln!(out, "struct {name};").unwrap(),
            Style::Type | Style::Both => writeln!(out, "typedef struct {name} {name};").unwrap(),
        }
        if i + 1 < opaque_defs.len() {
            out.push('\n');
        }
    }

    // Blank line between opaque and struct sections.
    if !opaque_defs.is_empty() && !struct_defs.is_empty() {
        out.push('\n');
    }

    for (i, def) in struct_defs.iter().enumerate() {
        let name = &def.name;
        let CTypeKind::Struct(ref struct_def) = def.kind else {
            unreachable!();
        };
        let docs = lookup_docs(def.rustdoc_id.as_ref(), index);
        write_doc_comment(docs.as_deref(), config, out);
        write_c_struct_definition(name, struct_def, style, out);
        if i + 1 < struct_defs.len() {
            out.push('\n');
        }
    }
}

/// Emit a full C struct definition with fields.
fn write_c_struct_definition(
    name: &str,
    def: &crate::analysis::CStructDef,
    style: &Style,
    out: &mut String,
) {
    // In "Both" style, field types use Tag form for inner references.
    let field_style = match style {
        Style::Both => Style::Tag,
        other => other.clone(),
    };

    match style {
        Style::Type => {
            writeln!(out, "typedef struct {{").unwrap();
            for field in &def.fields {
                out.push_str("  ");
                write_c_decl(&field.type_, &field.name, &field_style, out);
                writeln!(out, ";").unwrap();
            }
            writeln!(out, "}} {name};").unwrap();
        }
        Style::Tag => {
            writeln!(out, "struct {name} {{").unwrap();
            for field in &def.fields {
                out.push_str("  ");
                write_c_decl(&field.type_, &field.name, &Style::Tag, out);
                writeln!(out, ";").unwrap();
            }
            writeln!(out, "}};").unwrap();
        }
        Style::Both => {
            writeln!(out, "typedef struct {name} {{").unwrap();
            for field in &def.fields {
                out.push_str("  ");
                write_c_decl(&field.type_, &field.name, &field_style, out);
                writeln!(out, ";").unwrap();
            }
            writeln!(out, "}} {name};").unwrap();
        }
    }
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
