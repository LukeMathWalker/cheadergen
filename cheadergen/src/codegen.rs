use std::collections::BTreeSet;
use std::fmt::Write;
use std::path::Path;

use crate::config::CConfig;
use rustdoc_ir::{FreeFunction, ScalarPrimitive, Type};

/// Generate a C header from the resolved functions, writing to `out`.
///
/// The output order follows cbindgen conventions:
/// header, include guard/pragma once, autogen warning, includes,
/// after_includes, cpp_compat open, declarations, cpp_compat close,
/// include guard close, trailer.
pub fn generate_c_header(config: &CConfig, functions: &mut [FreeFunction], out: &mut String) {
    // Sort by exported name for deterministic output.
    functions.sort_by(|a, b| exported_name(a).cmp(exported_name(b)));

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

    let has_functions = !functions.is_empty();

    // cpp_compat open (only if there are declarations to wrap).
    if config.cpp_compat && has_functions {
        out.push('\n');
        out.push_str("#ifdef __cplusplus\n");
        out.push_str("extern \"C\" {\n");
        out.push_str("#endif // __cplusplus\n");
    }

    // Function declarations.
    for func in functions.iter() {
        out.push('\n');
        write_c_function_decl(func, out);
        out.push('\n');
    }

    // cpp_compat close.
    if config.cpp_compat && has_functions {
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

fn exported_name(func: &FreeFunction) -> &str {
    func.header
        .symbol_name
        .as_deref()
        .unwrap_or(&func.path.function_name)
}

fn write_c_function_decl(func: &FreeFunction, out: &mut String) {
    let name = func
        .header
        .symbol_name
        .as_deref()
        .unwrap_or(&func.path.function_name);

    // Return type.
    match &func.header.output {
        None => out.push_str("void"),
        Some(ty) if is_void(ty) => out.push_str("void"),
        Some(ty) => write_c_type(ty, out),
    }

    write!(out, " {name}(").unwrap();

    // Parameters.
    if func.header.inputs.is_empty() && !func.header.is_c_variadic {
        out.push_str("void");
    } else {
        for (i, input) in func.header.inputs.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            write_c_param(&input.type_, input.name.as_str(), out);
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

fn write_c_param(ty: &Type, name: &str, out: &mut String) {
    // For pointer types, the name goes after the base type and asterisks.
    // For non-pointer types, it's just `type name`.
    let mut type_buf = String::new();
    write_c_type(ty, &mut type_buf);
    write!(out, "{type_buf} {name}").unwrap();
}

fn write_c_type(ty: &Type, out: &mut String) {
    match ty {
        Type::ScalarPrimitive(p) => out.push_str(scalar_to_c(p)),
        Type::RawPointer(ptr) => {
            if ptr.is_mutable {
                write_c_type(&ptr.inner, out);
                out.push('*');
            } else {
                out.push_str("const ");
                write_c_type(&ptr.inner, out);
                out.push('*');
            }
        }
        Type::Tuple(t) if t.elements.is_empty() => out.push_str("void"),
        Type::Array(a) => {
            // Arrays in parameter position decay to pointers in C, but for type
            // representation we write `type[N]`. This will need refinement for
            // parameters vs typedefs.
            write_c_type(&a.element_type, out);
            write!(out, "[{}]", a.len).unwrap();
        }
        // These should have been rejected earlier.
        Type::Path(_)
        | Type::Reference(_)
        | Type::Tuple(_)
        | Type::Slice(_)
        | Type::Generic(_) => {
            unreachable!("unsupported type in C codegen: {ty:?}")
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
