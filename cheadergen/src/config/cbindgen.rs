use std::path::Path;

use serde::Deserialize;

use super::{ConfigError, DocumentationLength, DocumentationStyle, RawCSection, RawConfig, RawCxxSection, RawFnSection, RawStaticSection, SortKey, Style};

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
    enumeration: Option<toml::Value>,
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

/// Translate a cbindgen config file into cheadergen format.
///
/// Reads the cbindgen TOML at `input`, translates supported fields,
/// emits warnings on stderr for unsupported fields, and writes the
/// cheadergen TOML to `output`.
pub fn translate(input: &Path, output: &Path) -> Result<(), ConfigError> {
    let contents = fs_err::read_to_string(input)
        .map_err(|e| ConfigError { message: format!("failed to read cbindgen config: {e}") })?;
    let cbindgen: CbindgenConfig = toml::from_str(&contents)
        .map_err(|e| ConfigError { message: format!("failed to parse cbindgen config: {e}") })?;

    let (raw, skipped) = translate_config(&cbindgen);

    let mut toml_str = toml::to_string_pretty(&raw)
        .map_err(|e| ConfigError { message: format!("failed to serialize cheadergen config: {e}") })?;

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

    fs_err::write(output, toml_str)
        .map_err(|e| ConfigError { message: format!("failed to write cheadergen config: {e}") })?;

    Ok(())
}

fn translate_config(cb: &CbindgenConfig) -> (RawConfig, Vec<&str>) {
    let mut skipped = collect_unsupported_fields(cb);
    emit_unsupported_warnings(cb);

    // Determine target language from cbindgen's `language` field.
    // cbindgen defaults to C when unset.
    let is_cxx = match cb.language.as_deref() {
        Some("Cxx") | Some("C++") => true,
        Some("Cython") => {
            eprintln!("warning: Cython language is not supported by cheadergen, defaulting to C");
            skipped.push("language = \"Cython\"");
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
            skipped.push("style");
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
    let documentation_style = cb
        .documentation_style
        .as_deref()
        .and_then(|s| match s {
            "auto" | "Auto" => Some(DocumentationStyle::Auto),
            "c" | "C" => Some(DocumentationStyle::C),
            "c99" | "C99" => Some(DocumentationStyle::C99),
            "doxy" | "Doxy" => Some(DocumentationStyle::Doxy),
            "cxx" | "Cxx" => Some(DocumentationStyle::Cxx),
            other => {
                eprintln!(
                    "warning: ignoring unrecognized cbindgen documentation_style `{other}`"
                );
                skipped.push("documentation_style");
                None
            }
        });
    let documentation_length = cb
        .documentation_length
        .as_deref()
        .and_then(|s| match s {
            "full" | "Full" => Some(DocumentationLength::Full),
            "short" | "Short" => Some(DocumentationLength::Short),
            other => {
                eprintln!(
                    "warning: ignoring unrecognized cbindgen documentation_length `{other}`"
                );
                skipped.push("documentation_length");
                None
            }
        });

    let mut config = RawConfig {
        header: cb.header.clone(),
        trailer: cb.trailer.clone(),
        include_guard: cb.include_guard.clone(),
        pragma_once,
        no_includes,
        after_includes: cb.after_includes.clone(),
        includes: cb.includes.clone().unwrap_or_default(),
        sys_includes: cb.sys_includes.clone().unwrap_or_default(),
        autogen_warning: cb.autogen_warning.clone(),
        documentation,
        documentation_style,
        documentation_length,
        sort_by: top_sort_by,
        fn_: fn_section,
        static_: static_section,
        c: None,
        cxx: None,
    };

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

type UnsupportedField = (&'static str, fn(&CbindgenConfig) -> bool);

const UNSUPPORTED_FIELDS: &[UnsupportedField] = &[
    ("include_version", |cb| cb.include_version.is_some()),
    ("package_version", |cb| cb.package_version.is_some()),
    ("namespace", |cb| cb.namespace.is_some()),
    ("namespaces", |cb| cb.namespaces.is_some()),
    ("using_namespaces", |cb| cb.using_namespaces.is_some()),
    ("braces", |cb| cb.braces.is_some()),
    ("line_length", |cb| cb.line_length.is_some()),
    ("tab_width", |cb| cb.tab_width.is_some()),
    ("line_endings", |cb| cb.line_endings.is_some()),
    ("usize_is_size_t", |cb| cb.usize_is_size_t.is_some()),
    (
        "only_target_dependencies",
        |cb| cb.only_target_dependencies.is_some(),
    ),
    ("[parse]", |cb| cb.parse.is_some()),
    ("[export]", |cb| cb.export.is_some()),
    ("[fn]", |cb| {
        cb.function
            .as_ref()
            .is_some_and(|s| !s.other.is_empty())
    }),
    ("[struct]", |cb| cb.structure.is_some()),
    ("[enum]", |cb| cb.enumeration.is_some()),
    ("[const]", |cb| {
        cb.constant
            .as_ref()
            .is_some_and(|s| !s.other.is_empty())
    }),
    ("[layout]", |cb| cb.layout.is_some()),
    ("[macro_expansion]", |cb| cb.macro_expansion.is_some()),
    ("[ptr]", |cb| cb.pointer.is_some()),
    ("[cython]", |cb| cb.cython.is_some()),
    ("[defines]", |cb| cb.defines.is_some()),
];

/// Collect names of unsupported cbindgen fields that are present.
fn collect_unsupported_fields<'a>(cb: &CbindgenConfig) -> Vec<&'a str> {
    UNSUPPORTED_FIELDS
        .iter()
        .filter(|(_, is_present)| is_present(cb))
        .map(|(name, _)| *name)
        .collect()
}

/// Emit warnings for all unsupported cbindgen fields that are present.
fn emit_unsupported_warnings(cb: &CbindgenConfig) {
    for (name, is_present) in UNSUPPORTED_FIELDS {
        if is_present(cb) {
            eprintln!("warning: ignoring unsupported cbindgen option `{name}`");
        }
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
        let output = translate_to_toml(r##"
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
"##);
        insta::assert_snapshot!(output, @r##"
        header = "/* License */"
        trailer = "/* End */"
        autogen_warning = "// DO NOT EDIT"
        include_guard = "MY_H"
        pragma_once = true
        sys_includes = ["stdint.h"]
        includes = ["foo.h"]
        no_includes = true
        after_includes = "#define FOO 1"

        [c]
        style = "Tag"
        cpp_compat = true
        "##);
    }

    #[test]
    fn default_values_suppressed() {
        let output = translate_to_toml(r#"
pragma_once = false
no_includes = false
cpp_compat = false
style = "Type"
"#);
        insta::assert_snapshot!(output, @"");
    }

    #[test]
    fn cxx_language_creates_cxx_section() {
        let output = translate_to_toml(r#"
language = "Cxx"
header = "/* C++ */"
"#);
        insta::assert_snapshot!(output, @r#"
        header = "/* C++ */"

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
        let output = translate_to_toml(r#"
language = "C"
style = "Both"
"#);
        insta::assert_snapshot!(output, @r#"
        [c]
        style = "Both"
        "#);
    }

    #[test]
    fn roundtrip_serialization() {
        let output = translate_to_toml(r##"
header = "/* License */"
include_guard = "MY_H"
includes = ["foo.h"]
sys_includes = ["stdint.h"]
style = "Tag"
cpp_compat = true
"##);

        // Verify the output parses back as valid cheadergen config.
        let _: RawConfig = toml::from_str(&output).unwrap();

        insta::assert_snapshot!(output, @r#"
        header = "/* License */"
        include_guard = "MY_H"
        sys_includes = ["stdint.h"]
        includes = ["foo.h"]

        [c]
        style = "Tag"
        cpp_compat = true
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
        # `[export]` was skipped: not supported by cheadergen
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
        header = "/* License */"

        # `braces` was skipped: not supported by cheadergen
        # `line_length` was skipped: not supported by cheadergen
        # `[export]` was skipped: not supported by cheadergen
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
        include_guard = "TEST_H"
        after_includes = "#define VERSION 1"
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
        include_guard = "TEST_H"

        # `braces` was skipped: not supported by cheadergen
        # `[export]` was skipped: not supported by cheadergen
        "#);
    }
}
