use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use serde::Deserialize;

use super::{
    ConfigError, DocumentationLength, DocumentationStyle, RawCSection, RawConfig, RawCxxSection,
    RawEnumSection, RawFnSection, RawStaticSection, SortKey, Style,
};

/// Permissive deserialization of a cbindgen config file.
///
/// Translatable fields are deserialized into their proper types.
/// Unsupported fields use `Option<toml::Value>` so we can detect their
/// presence (for warnings) without needing full type definitions.
#[derive(Deserialize)]
pub(crate) struct CbindgenConfig {
    // -- Translatable fields --
    header: Option<String>,
    trailer: Option<String>,
    include_guard: Option<String>,
    pragma_once: Option<bool>,
    no_includes: Option<bool>,
    after_includes: Option<String>,
    includes: Option<Vec<String>>,
    sys_includes: Option<Vec<String>>,
    autogen_warning: Option<String>,
    language: Option<String>,
    style: Option<String>,
    cpp_compat: Option<bool>,

    // -- Unsupported fields (detected for warnings) --
    include_version: Option<toml::Value>,
    package_version: Option<toml::Value>,
    namespace: Option<toml::Value>,
    namespaces: Option<toml::Value>,
    using_namespaces: Option<toml::Value>,
    braces: Option<toml::Value>,
    line_length: Option<toml::Value>,
    tab_width: Option<toml::Value>,
    line_endings: Option<toml::Value>,
    sort_by: Option<String>,
    usize_is_size_t: Option<toml::Value>,
    documentation: Option<bool>,
    documentation_style: Option<String>,
    documentation_length: Option<String>,
    only_target_dependencies: Option<toml::Value>,
    parse: Option<toml::Value>,
    export: Option<toml::Value>,
    #[serde(rename = "fn")]
    function: Option<CbindgenFnSection>,
    #[serde(rename = "struct")]
    structure: Option<toml::Value>,
    #[serde(rename = "enum")]
    enumeration: Option<CbindgenEnumSection>,
    #[serde(rename = "const")]
    constant: Option<CbindgenConstSection>,
    layout: Option<toml::Value>,
    macro_expansion: Option<toml::Value>,
    #[serde(rename = "ptr")]
    pointer: Option<toml::Value>,
    cython: Option<toml::Value>,
    defines: Option<toml::Value>,
}

/// Cbindgen `[fn]` section. Currently only `sort_by` is translatable.
#[derive(Deserialize)]
struct CbindgenFnSection {
    sort_by: Option<String>,
    #[serde(flatten)]
    other: std::collections::HashMap<String, toml::Value>,
}

/// Cbindgen `[const]` section. Currently only `sort_by` is translatable.
#[derive(Deserialize)]
struct CbindgenConstSection {
    sort_by: Option<String>,
    #[serde(flatten)]
    other: std::collections::HashMap<String, toml::Value>,
}

/// Cbindgen `[enum]` section. Currently only `prefix_with_name` is translatable.
#[derive(Deserialize)]
struct CbindgenEnumSection {
    prefix_with_name: Option<bool>,
    #[serde(flatten)]
    other: std::collections::HashMap<String, toml::Value>,
}

/// Translate a cbindgen config file into cheadergen format.
///
/// Reads the cbindgen TOML at `input`, translates supported fields,
/// emits warnings on stderr for unsupported fields, and writes the
/// cheadergen TOML to `output`.
pub fn translate(input: &Path, output: &Path) -> Result<(), ConfigError> {
    let contents = fs_err::read_to_string(input).map_err(|e| ConfigError {
        message: format!("failed to read cbindgen config: {e}"),
    })?;
    let cbindgen: CbindgenConfig = toml::from_str(&contents).map_err(|e| ConfigError {
        message: format!("failed to parse cbindgen config: {e}"),
    })?;

    let (raw, skipped) = translate_config(&cbindgen);

    let mut toml_str = toml::to_string_pretty(&raw).map_err(|e| ConfigError {
        message: format!("failed to serialize cheadergen config: {e}"),
    })?;

    if !skipped.is_empty() {
        if !toml_str.is_empty() {
            toml_str.push('\n');
        }
        for field in &skipped {
            toml_str.push_str(&format!(
                "# `{field}` was skipped: not supported by cheadergen\n"
            ));
        }
    }

    fs_err::write(output, toml_str).map_err(|e| ConfigError {
        message: format!("failed to write cheadergen config: {e}"),
    })?;

    Ok(())
}

fn translate_config(cb: &CbindgenConfig) -> (RawConfig, Vec<String>) {
    let mut skipped = collect_unsupported_fields(cb);
    emit_unsupported_warnings(cb);

    // Determine target language from cbindgen's `language` field.
    // cbindgen defaults to C when unset.
    let is_cxx = match cb.language.as_deref() {
        Some("Cxx") | Some("C++") => true,
        Some("Cython") => {
            eprintln!("warning: Cython language is not supported by cheadergen, defaulting to C");
            skipped.push("language = \"Cython\"".into());
            false
        }
        _ => false,
    };

    // Translate style string to our Style enum, suppressing the default ("Type" in cbindgen).
    let style = cb.style.as_deref().and_then(|s| match s {
        "Both" | "both" => Some(Style::Both),
        "Tag" | "tag" => Some(Style::Tag),
        // "Type" is cbindgen's default — suppress it.
        "Type" | "type" => None,
        other => {
            eprintln!("warning: ignoring unrecognized cbindgen style `{other}`");
            skipped.push("style".into());
            None
        }
    });

    // Suppress default values.
    let pragma_once = cb.pragma_once.filter(|&v| v);
    let no_includes = cb.no_includes.filter(|&v| v);
    let cpp_compat = cb.cpp_compat.filter(|&v| v);

    // Translate top-level sort_by.
    let top_sort_by = cb.sort_by.as_deref().and_then(translate_sort_by);

    // Translate [fn] section.
    let fn_section = cb.function.as_ref().and_then(|section| {
        let sort_by = section.sort_by.as_deref().and_then(translate_sort_by);
        if !section.other.is_empty() {
            let other_keys: Vec<_> = section.other.keys().map(|k| k.as_str()).collect();
            eprintln!(
                "warning: ignoring unsupported cbindgen [fn] option(s): {}",
                other_keys.join(", ")
            );
        }
        sort_by.map(|s| RawFnSection { sort_by: Some(s) })
    });

    // Translate [const] section → cheadergen [static] section.
    let static_section = cb.constant.as_ref().and_then(|section| {
        let sort_by = section.sort_by.as_deref().and_then(translate_sort_by);
        if !section.other.is_empty() {
            let other_keys: Vec<_> = section.other.keys().map(|k| k.as_str()).collect();
            eprintln!(
                "warning: ignoring unsupported cbindgen [const] option(s): {}",
                other_keys.join(", ")
            );
        }
        sort_by.map(|s| RawStaticSection { sort_by: Some(s) })
    });

    // Translate documentation fields.
    // cbindgen defaults documentation to true, so we suppress that default.
    let documentation = cb.documentation.filter(|&v| !v);
    let documentation_style = cb.documentation_style.as_deref().and_then(|s| match s {
        "auto" | "Auto" => Some(DocumentationStyle::Auto),
        "c" | "C" => Some(DocumentationStyle::C),
        "c99" | "C99" => Some(DocumentationStyle::C99),
        "doxy" | "Doxy" => Some(DocumentationStyle::Doxy),
        "cxx" | "Cxx" => Some(DocumentationStyle::Cxx),
        other => {
            eprintln!("warning: ignoring unrecognized cbindgen documentation_style `{other}`");
            skipped.push("documentation_style".into());
            None
        }
    });
    let documentation_length = cb.documentation_length.as_deref().and_then(|s| match s {
        "full" | "Full" => Some(DocumentationLength::Full),
        "short" | "Short" => Some(DocumentationLength::Short),
        other => {
            eprintln!("warning: ignoring unrecognized cbindgen documentation_length `{other}`");
            skipped.push("documentation_length".into());
            None
        }
    });

    // Translate [enum] section.
    let enum_section = cb.enumeration.as_ref().and_then(|section| {
        if !section.other.is_empty() {
            let other_keys: Vec<_> = section.other.keys().map(|k| k.as_str()).collect();
            eprintln!(
                "warning: ignoring unsupported cbindgen [enum] option(s): {}",
                other_keys.join(", ")
            );
        }
        // Only emit if prefix_with_name is true (false is our default).
        let prefix = section.prefix_with_name.filter(|&v| v);
        prefix.map(|p| RawEnumSection {
            prefix_with_name: Some(p),
        })
    });

    // cbindgen keeps `includes` and `sys_includes` as separate fields; cheadergen
    // exposes a single `includes` list where system headers are written as
    // `<foo.h>`. Merge the two, keeping system includes first to preserve the
    // historical emission order.
    let mut includes: Vec<String> = cb
        .sys_includes
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|h| format!("<{h}>"))
        .collect();
    includes.extend(cb.includes.clone().unwrap_or_default());

    let mut config = RawConfig {
        preamble: cb.header.clone(),
        trailer: cb.trailer.clone(),
        pragma_once,
        no_includes,
        after_includes: cb.after_includes.clone(),
        includes,
        autogen_warning: cb.autogen_warning.clone(),
        documentation,
        documentation_style,
        documentation_length,
        sort_by: top_sort_by,
        fn_: fn_section,
        static_: static_section,
        constant_: None,
        enum_: enum_section,
        bundle: None,
        package: BTreeMap::new(),
        header: HashMap::new(),
        c: None,
        cxx: None,
    };

    // include_guard is now per-header only; the translator cannot place it
    // without knowing the crate name, so skip it with a comment.
    if cb.include_guard.is_some() {
        skipped.push("include_guard".into());
    }

    if is_cxx {
        // For C++ language, create a [cxx] section (no style/cpp_compat).
        config.cxx = Some(RawCxxSection::default());
    } else {
        // For C language, only create a [c] section if there are C-specific fields.
        if style.is_some() || cpp_compat.is_some() {
            config.c = Some(RawCSection {
                style,
                cpp_compat,
                ..Default::default()
            });
        }
    }

    (config, skipped)
}

/// Translate a cbindgen `sort_by` string to a cheadergen [`SortKey`].
///
/// Returns `None` for `"None"` (cbindgen's default, equivalent to cheadergen's `SourceOrder`).
fn translate_sort_by(value: &str) -> Option<SortKey> {
    match value {
        "Name" | "name" => Some(SortKey::Name),
        "None" | "none" => None,
        other => {
            eprintln!("warning: ignoring unrecognized cbindgen sort_by `{other}`");
            None
        }
    }
}

type UnsupportedFieldCollector = (&'static str, fn(&CbindgenConfig, &mut Vec<String>));

const UNSUPPORTED_FIELD_COLLECTORS: &[UnsupportedFieldCollector] = &[
    ("include_version", |cb, out| {
        if cb.include_version.is_some() {
            out.push("include_version".into());
        }
    }),
    ("package_version", |cb, out| {
        if cb.package_version.is_some() {
            out.push("package_version".into());
        }
    }),
    ("namespace", |cb, out| {
        if cb.namespace.is_some() {
            out.push("namespace".into());
        }
    }),
    ("namespaces", |cb, out| {
        if cb.namespaces.is_some() {
            out.push("namespaces".into());
        }
    }),
    ("using_namespaces", |cb, out| {
        if cb.using_namespaces.is_some() {
            out.push("using_namespaces".into());
        }
    }),
    ("braces", |cb, out| {
        if cb.braces.is_some() {
            out.push("braces".into());
        }
    }),
    ("line_length", |cb, out| {
        if cb.line_length.is_some() {
            out.push("line_length".into());
        }
    }),
    ("tab_width", |cb, out| {
        if cb.tab_width.is_some() {
            out.push("tab_width".into());
        }
    }),
    ("line_endings", |cb, out| {
        if cb.line_endings.is_some() {
            out.push("line_endings".into());
        }
    }),
    ("usize_is_size_t", |cb, out| {
        if cb.usize_is_size_t.is_some() {
            out.push("usize_is_size_t".into());
        }
    }),
    ("only_target_dependencies", |cb, out| {
        if cb.only_target_dependencies.is_some() {
            out.push("only_target_dependencies".into());
        }
    }),
    ("parse", |cb, out| {
        if let Some(v) = &cb.parse {
            collect_table_keys("parse", v, out);
        }
    }),
    ("export", |cb, out| {
        if let Some(v) = &cb.export {
            collect_table_keys("export", v, out);
        }
    }),
    ("fn", |cb, out| {
        if let Some(s) = &cb.function {
            for key in s.other.keys() {
                out.push(format!("fn.{key}"));
            }
        }
    }),
    ("struct", |cb, out| {
        if let Some(v) = &cb.structure {
            collect_table_keys("struct", v, out);
        }
    }),
    ("enum", |cb, out| {
        if let Some(s) = &cb.enumeration {
            for key in s.other.keys() {
                out.push(format!("enum.{key}"));
            }
        }
    }),
    ("const", |cb, out| {
        if let Some(s) = &cb.constant {
            for key in s.other.keys() {
                out.push(format!("const.{key}"));
            }
        }
    }),
    ("layout", |cb, out| {
        if let Some(v) = &cb.layout {
            collect_table_keys("layout", v, out);
        }
    }),
    ("macro_expansion", |cb, out| {
        if let Some(v) = &cb.macro_expansion {
            collect_table_keys("macro_expansion", v, out);
        }
    }),
    ("ptr", |cb, out| {
        if let Some(v) = &cb.pointer {
            collect_table_keys("ptr", v, out);
        }
    }),
    ("cython", |cb, out| {
        if let Some(v) = &cb.cython {
            collect_table_keys("cython", v, out);
        }
    }),
    ("defines", |cb, out| {
        if let Some(v) = &cb.defines {
            collect_table_keys("defines", v, out);
        }
    }),
];

/// Push `section.key` for each key in a TOML table, or just `section` if it's not a table.
fn collect_table_keys(section: &str, value: &toml::Value, out: &mut Vec<String>) {
    if let Some(table) = value.as_table() {
        for key in table.keys() {
            out.push(format!("{section}.{key}"));
        }
    } else {
        out.push(section.into());
    }
}

/// Collect names of unsupported cbindgen fields that are present.
fn collect_unsupported_fields(cb: &CbindgenConfig) -> Vec<String> {
    let mut out = Vec::new();
    for (_, collector) in UNSUPPORTED_FIELD_COLLECTORS {
        collector(cb, &mut out);
    }
    out
}

/// Emit warnings for all unsupported cbindgen fields that are present.
fn emit_unsupported_warnings(cb: &CbindgenConfig) {
    let fields = collect_unsupported_fields(cb);
    for name in &fields {
        eprintln!("warning: ignoring unsupported cbindgen option `{name}`");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn translate_to_toml(input: &str) -> String {
        let cb: CbindgenConfig = toml::from_str(input).unwrap();
        let (config, skipped) = translate_config(&cb);
        let mut output = toml::to_string_pretty(&config).unwrap();
        if !skipped.is_empty() {
            if !output.is_empty() {
                output.push('\n');
            }
            for field in &skipped {
                output.push_str(&format!(
                    "# `{field}` was skipped: not supported by cheadergen\n"
                ));
            }
        }
        output
    }

    #[test]
    fn empty_config_produces_empty_output() {
        let output = translate_to_toml("");
        insta::assert_snapshot!(output, @"");
    }

    #[test]
    fn all_supported_fields() {
        let output = translate_to_toml(
            r##"
header = "/* License */"
trailer = "/* End */"
include_guard = "MY_H"
pragma_once = true
no_includes = true
after_includes = "#define FOO 1"
includes = ["foo.h"]
sys_includes = ["stdint.h"]
autogen_warning = "// DO NOT EDIT"
style = "Tag"
cpp_compat = true
"##,
        );
        insta::assert_snapshot!(output, @r##"
        preamble = "/* License */"
        trailer = "/* End */"
        autogen_warning = "// DO NOT EDIT"
        pragma_once = true
        includes = [
            "<stdint.h>",
            "foo.h",
        ]
        no_includes = true
        after_includes = "#define FOO 1"

        [c]
        style = "Tag"
        cpp_compat = true

        # `include_guard` was skipped: not supported by cheadergen
        "##);
    }

    #[test]
    fn default_values_suppressed() {
        let output = translate_to_toml(
            r#"
pragma_once = false
no_includes = false
cpp_compat = false
style = "Type"
"#,
        );
        insta::assert_snapshot!(output, @"");
    }

    #[test]
    fn cxx_language_creates_cxx_section() {
        let output = translate_to_toml(
            r#"
language = "Cxx"
header = "/* C++ */"
"#,
        );
        insta::assert_snapshot!(output, @r#"
        preamble = "/* C++ */"

        [cxx]
        "#);
    }

    #[test]
    fn cpp_language_alias() {
        let output = translate_to_toml(r#"language = "C++""#);
        insta::assert_snapshot!(output, @r"
        [cxx]
        ");
    }

    #[test]
    fn c_language_explicit() {
        let output = translate_to_toml(
            r#"
language = "C"
style = "Both"
"#,
        );
        insta::assert_snapshot!(output, @r#"
        [c]
        style = "Both"
        "#);
    }

    #[test]
    fn roundtrip_serialization() {
        let output = translate_to_toml(
            r##"
header = "/* License */"
include_guard = "MY_H"
includes = ["foo.h"]
sys_includes = ["stdint.h"]
style = "Tag"
cpp_compat = true
"##,
        );

        // Verify the output parses back as valid cheadergen config.
        let _: RawConfig = toml::from_str(&output).unwrap();

        insta::assert_snapshot!(output, @r#"
        preamble = "/* License */"
        includes = [
            "<stdint.h>",
            "foo.h",
        ]

        [c]
        style = "Tag"
        cpp_compat = true

        # `include_guard` was skipped: not supported by cheadergen
        "#);
    }

    #[test]
    fn unsupported_fields_produce_comments() {
        let output = translate_to_toml(
            r#"
braces = "SameLine"

[export]
include = ["Foo"]
"#,
        );
        insta::assert_snapshot!(output, @r#"
        # `braces` was skipped: not supported by cheadergen
        # `export.include` was skipped: not supported by cheadergen
        "#);
    }

    #[test]
    fn unsupported_fields_with_supported_fields() {
        let output = translate_to_toml(
            r#"
header = "/* License */"
braces = "SameLine"
line_length = 100

[export]
include = ["Foo"]
"#,
        );
        insta::assert_snapshot!(output, @r#"
        preamble = "/* License */"

        # `braces` was skipped: not supported by cheadergen
        # `line_length` was skipped: not supported by cheadergen
        # `export.include` was skipped: not supported by cheadergen
        "#);
    }

    #[test]
    fn cython_language_skipped() {
        let output = translate_to_toml(r#"language = "Cython""#);
        insta::assert_snapshot!(output, @r#"
        # `language = "Cython"` was skipped: not supported by cheadergen
        "#);
    }

    #[test]
    fn unrecognized_style_skipped() {
        let output = translate_to_toml(r#"style = "Unknown""#);
        insta::assert_snapshot!(output, @r"
        # `style` was skipped: not supported by cheadergen
        ");
    }

    #[test]
    fn translate_file_end_to_end() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("cbindgen.toml");
        let output_path = dir.path().join("cheadergen.toml");

        fs_err::write(
            &input_path,
            r##"
include_guard = "TEST_H"
after_includes = "#define VERSION 1"
"##,
        )
        .unwrap();

        translate(&input_path, &output_path).unwrap();

        let output = fs_err::read_to_string(&output_path).unwrap();
        insta::assert_snapshot!(output, @r##"
        after_includes = "#define VERSION 1"

        # `include_guard` was skipped: not supported by cheadergen
        "##);
    }

    #[test]
    fn translate_file_end_to_end_with_unsupported_fields() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("cbindgen.toml");
        let output_path = dir.path().join("cheadergen.toml");

        fs_err::write(
            &input_path,
            r#"
include_guard = "TEST_H"
braces = "SameLine"

[export]
include = ["Foo"]
"#,
        )
        .unwrap();

        translate(&input_path, &output_path).unwrap();

        let output = fs_err::read_to_string(&output_path).unwrap();
        insta::assert_snapshot!(output, @r#"
        # `braces` was skipped: not supported by cheadergen
        # `export.include` was skipped: not supported by cheadergen
        # `include_guard` was skipped: not supported by cheadergen
        "#);
    }
}
