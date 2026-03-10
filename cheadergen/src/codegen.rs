use std::fmt::Write;

use rustdoc_ir::{FreeFunction, ScalarPrimitive, Type};

/// Generate a C header from the resolved functions, writing to `out`.
pub fn generate_c_header(functions: &mut [FreeFunction], out: &mut String) {
    // Sort by exported name for deterministic output.
    functions.sort_by(|a, b| exported_name(a).cmp(exported_name(b)));

    // Standard includes.
    out.push_str("#include <stdarg.h>\n");
    out.push_str("#include <stdbool.h>\n");
    out.push_str("#include <stdint.h>\n");
    out.push_str("#include <stdlib.h>\n");

    for func in functions.iter() {
        out.push('\n');
        write_c_function_decl(func, out);
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
