use std::path::Path;

use serde::Deserialize;

use super::{ConfigError, RawCSection, RawConfig, RawCxxSection, Style};

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
    sort_by: Option<toml::Value>,
    usize_is_size_t: Option<toml::Value>,
    documentation: Option<toml::Value>,
    documentation_style: Option<toml::Value>,
    documentation_length: Option<toml::Value>,
    only_target_dependencies: Option<toml::Value>,
    parse: Option<toml::Value>,
    export: Option<toml::Value>,
    #[serde(rename = "fn")]
    function: Option<toml::Value>,
    #[serde(rename = "struct")]
    structure: Option<toml::Value>,
    #[serde(rename = "enum")]
    enumeration: Option<toml::Value>,
    #[serde(rename = "const")]
    constant: Option<toml::Value>,
    layout: Option<toml::Value>,
    macro_expansion: Option<toml::Value>,
    #[serde(rename = "ptr")]
    pointer: Option<toml::Value>,
    cython: Option<toml::Value>,
    defines: Option<toml::Value>,
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

    let raw = translate_config(&cbindgen);

    let toml_str = toml::to_string_pretty(&raw)
        .map_err(|e| ConfigError { message: format!("failed to serialize cheadergen config: {e}") })?;
    fs_err::write(output, toml_str)
        .map_err(|e| ConfigError { message: format!("failed to write cheadergen config: {e}") })?;

    Ok(())
}

fn translate_config(cb: &CbindgenConfig) -> RawConfig {
    emit_unsupported_warnings(cb);

    // Determine target language from cbindgen's `language` field.
    // cbindgen defaults to C when unset.
    let is_cxx = match cb.language.as_deref() {
        Some("Cxx") | Some("C++") => true,
        Some("Cython") => {
            eprintln!("warning: Cython language is not supported by cheadergen, defaulting to C");
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
            None
        }
    });

    // Suppress default values.
    let pragma_once = cb.pragma_once.filter(|&v| v);
    let no_includes = cb.no_includes.filter(|&v| v);
    let cpp_compat = cb.cpp_compat.filter(|&v| v);

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

    config
}

/// Emit warnings for all unsupported cbindgen fields that are present.
fn emit_unsupported_warnings(cb: &CbindgenConfig) {
    let fields: &[(&str, bool)] = &[
        ("include_version", cb.include_version.is_some()),
        ("package_version", cb.package_version.is_some()),
        ("namespace", cb.namespace.is_some()),
        ("namespaces", cb.namespaces.is_some()),
        ("using_namespaces", cb.using_namespaces.is_some()),
        ("braces", cb.braces.is_some()),
        ("line_length", cb.line_length.is_some()),
        ("tab_width", cb.tab_width.is_some()),
        ("line_endings", cb.line_endings.is_some()),
        ("sort_by", cb.sort_by.is_some()),
        ("usize_is_size_t", cb.usize_is_size_t.is_some()),
        ("documentation", cb.documentation.is_some()),
        ("documentation_style", cb.documentation_style.is_some()),
        ("documentation_length", cb.documentation_length.is_some()),
        ("only_target_dependencies", cb.only_target_dependencies.is_some()),
        ("[parse]", cb.parse.is_some()),
        ("[export]", cb.export.is_some()),
        ("[fn]", cb.function.is_some()),
        ("[struct]", cb.structure.is_some()),
        ("[enum]", cb.enumeration.is_some()),
        ("[const]", cb.constant.is_some()),
        ("[layout]", cb.layout.is_some()),
        ("[macro_expansion]", cb.macro_expansion.is_some()),
        ("[ptr]", cb.pointer.is_some()),
        ("[cython]", cb.cython.is_some()),
        ("[defines]", cb.defines.is_some()),
    ];

    for &(name, present) in fields {
        if present {
            eprintln!("warning: ignoring unsupported cbindgen option `{name}`");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn translate_to_toml(input: &str) -> String {
        let cb: CbindgenConfig = toml::from_str(input).unwrap();
        let config = translate_config(&cb);
        toml::to_string_pretty(&config).unwrap()
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
}
