pub mod cbindgen;

use std::path::Path;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// The target language for the generated header file.
///
/// Used by the CLI `--lang` flag to select which language output to produce.
#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum Language {
    /// Generate a C-compatible header.
    #[value(name = "c", alias = "C")]
    C,
    /// Generate a C++ header.
    #[value(name = "c++", alias = "C++", alias = "cpp")]
    Cxx,
    /// Reserved for future Cython support. Currently rejected at validation time
    /// by [`RawConfig::into_config`].
    #[value(name = "cython", alias = "Cython")]
    Cython,
}

/// The declaration style for C struct and enum definitions.
///
/// This only applies when the target language is [`Language::C`].
/// C++ does not use typedef-style declarations, so this option is ignored
/// (and rejected) for [`Language::Cxx`].
#[derive(Debug, Clone, ValueEnum, Deserialize, Serialize)]
pub enum Style {
    /// Emit both a tag definition and a typedef:
    /// `typedef struct MyType { ... } MyType;`
    #[value(name = "both", alias = "Both")]
    #[serde(alias = "both")]
    Both,
    /// Emit only a tag definition: `struct MyType { ... };`
    #[value(name = "tag", alias = "Tag")]
    #[serde(alias = "tag")]
    Tag,
    /// Emit only a typedef: `typedef struct { ... } MyType;`
    #[value(name = "type", alias = "Type")]
    #[serde(alias = "type")]
    Type,
}

/// The raw configuration as deserialized from a TOML file.
///
/// Common options live at the top level and are inherited by all language
/// outputs. Language-specific sections (`[c]`, `[cxx]`) can override any
/// common option and add language-specific settings.
///
/// Defaults are resolved and language-specific constraints are validated
/// when converting into a [`Config`] via [`RawConfig::into_config`].
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawConfig {
    /// Verbatim text prepended to the generated file (e.g. a license block).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    /// Verbatim text appended to the end of the generated file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<String>,
    /// Warning text emitted between major sections to discourage manual edits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autogen_warning: Option<String>,
    /// Custom `#ifndef`/`#define` include guard name.
    /// When omitted, no include guard is emitted (see also [`pragma_once`](RawConfig::pragma_once)).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_guard: Option<String>,
    /// Emit `#pragma once` instead of (or in addition to) an include guard.
    /// Defaults to `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pragma_once: Option<bool>,
    /// System headers to emit as `#include <…>`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sys_includes: Vec<String>,
    /// User headers to emit as `#include "…"`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub includes: Vec<String>,
    /// Suppress the default language-specific includes (e.g. `<stdint.h>` for C).
    /// Defaults to `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_includes: Option<bool>,
    /// Verbatim text inserted immediately after the include block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_includes: Option<String>,

    /// C-specific configuration section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c: Option<RawCSection>,
    /// C++-specific configuration section.
    #[serde(alias = "c++", alias = "cpp", skip_serializing_if = "Option::is_none")]
    pub cxx: Option<RawCxxSection>,
}

/// C-specific options inside the `[c]` TOML section.
///
/// Any common option specified here overrides the top-level default
/// for C output only.
///
/// Common fields are duplicated from [`RawConfig`] (rather than extracted
/// into a shared struct with `#[serde(flatten)]`) so that we can keep
/// `#[serde(deny_unknown_fields)]` on every struct — giving clear error
/// messages when a config file contains a typo.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawCSection {
    /// C declaration style for structs and enums. Defaults to [`Style::Both`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<Style>,
    /// Wrap C output in an `extern "C"` block for C++ compatibility.
    /// Defaults to `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpp_compat: Option<bool>,
    // Common option overrides (see struct-level doc for why these are duplicated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autogen_warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_guard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pragma_once: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sys_includes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub includes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_includes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_includes: Option<String>,
}

/// C++-specific options inside the `[cxx]` TOML section.
///
/// Any common option specified here overrides the top-level default
/// for C++ output only.
///
/// See [`RawCSection`] for why common fields are duplicated here.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawCxxSection {
    // Common option overrides (see RawCSection doc for why these are duplicated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autogen_warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_guard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pragma_once: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sys_includes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub includes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_includes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_includes: Option<String>,
}

/// Validated, language-specific configuration.
///
/// Produced by [`RawConfig::into_config`] after resolving defaults and
/// enforcing language-specific constraints.
#[derive(Debug, Clone)]
pub enum Config {
    /// Configuration for [`Language::C`] output.
    C(CConfig),
    /// Configuration for [`Language::Cxx`] output.
    #[allow(dead_code)]
    Cxx(CxxConfig),
}

/// Validated options shared across all target languages.
///
/// Fields mirror [`RawConfig`] but with defaults resolved
/// (e.g. `Option<bool>` becomes `bool`).
#[derive(Debug, Clone)]
pub struct CommonConfig {
    /// See [`RawConfig::header`].
    pub header: Option<String>,
    /// See [`RawConfig::trailer`].
    pub trailer: Option<String>,
    /// See [`RawConfig::autogen_warning`].
    pub autogen_warning: Option<String>,
    /// See [`RawConfig::include_guard`].
    pub include_guard: Option<String>,
    /// See [`RawConfig::pragma_once`]. Defaults to `false`.
    pub pragma_once: bool,
    /// See [`RawConfig::sys_includes`].
    pub sys_includes: Vec<String>,
    /// See [`RawConfig::includes`].
    pub includes: Vec<String>,
    /// See [`RawConfig::no_includes`]. Defaults to `false`.
    pub no_includes: bool,
    /// See [`RawConfig::after_includes`].
    pub after_includes: Option<String>,
}

/// C-specific configuration, including options that are only meaningful for
/// [`Language::C`] output (e.g. [`Style`] and `cpp_compat`).
#[derive(Debug, Clone)]
pub struct CConfig {
    /// Options shared with all languages.
    pub common: CommonConfig,
    /// See [`RawCSection::style`]. Defaults to [`Style::Both`].
    #[allow(dead_code)]
    pub style: Style,
    /// See [`RawCSection::cpp_compat`]. Defaults to `false`.
    pub cpp_compat: bool,
}

/// C++-specific configuration for [`Language::Cxx`] output.
///
/// Currently only holds the [`CommonConfig`] shared options, since C++ does not
/// use typedef-style declarations or `cpp_compat`.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CxxConfig {
    /// Options shared with all languages.
    pub common: CommonConfig,
}

/// Error returned when config parsing or validation fails.
///
/// This covers both TOML deserialization errors (from [`RawConfig::from_toml_file`])
/// and semantic validation errors (from [`RawConfig::into_config`]).
#[derive(Debug)]
pub struct ConfigError {
    pub message: String,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ConfigError {}

/// Top-level common fields extracted from [`RawConfig`] for merging with
/// language-section overrides.
struct RawCommonFields {
    header: Option<String>,
    trailer: Option<String>,
    autogen_warning: Option<String>,
    include_guard: Option<String>,
    pragma_once: Option<bool>,
    sys_includes: Vec<String>,
    includes: Vec<String>,
    no_includes: Option<bool>,
    after_includes: Option<String>,
}

/// Optional overrides from a language section that can replace top-level
/// common field values.
struct RawCommonOverrides {
    header: Option<String>,
    trailer: Option<String>,
    autogen_warning: Option<String>,
    include_guard: Option<String>,
    pragma_once: Option<bool>,
    sys_includes: Option<Vec<String>>,
    includes: Option<Vec<String>>,
    no_includes: Option<bool>,
    after_includes: Option<String>,
}

impl RawCommonFields {
    /// Merge section overrides onto these base fields, producing a validated
    /// [`CommonConfig`]. Section values win when present; otherwise the
    /// top-level value is used.
    fn resolve(self, overrides: RawCommonOverrides) -> CommonConfig {
        CommonConfig {
            header: overrides.header.or(self.header),
            trailer: overrides.trailer.or(self.trailer),
            autogen_warning: overrides.autogen_warning.or(self.autogen_warning),
            include_guard: overrides.include_guard.or(self.include_guard),
            pragma_once: overrides
                .pragma_once
                .or(self.pragma_once)
                .unwrap_or(false),
            sys_includes: overrides.sys_includes.unwrap_or(self.sys_includes),
            includes: overrides.includes.unwrap_or(self.includes),
            no_includes: overrides
                .no_includes
                .or(self.no_includes)
                .unwrap_or(false),
            after_includes: overrides.after_includes.or(self.after_includes),
        }
    }
}

/// CLI overrides that can be applied to the config before validation.
#[derive(Debug, Default)]
pub struct CliOverrides {
    /// Override the C declaration style.
    pub style: Option<Style>,
    /// Force `cpp_compat` on.
    pub cpp_compat: bool,
}

impl RawConfig {
    /// Read and deserialize a [`RawConfig`] from a TOML file at `path`.
    pub fn from_toml_file(path: &Path) -> Result<Self, ConfigError> {
        let contents = fs_err::read_to_string(path)
            .map_err(|e| ConfigError { message: format!("failed to read config file: {e}") })?;
        toml::from_str(&contents)
            .map_err(|e| ConfigError { message: format!("failed to parse config file: {e}") })
    }

    /// Validate and convert into a language-specific [`Config`].
    ///
    /// `language` selects which language section to use.
    /// If the matching section is absent, top-level common options are used with
    /// language-specific defaults.
    ///
    /// `overrides` allows CLI flags to override config values.
    pub fn into_config(
        self,
        language: &Language,
        overrides: &CliOverrides,
    ) -> Result<Config, ConfigError> {
        match language {
            Language::Cython => {
                return Err(ConfigError {
                    message: "Cython output is not yet supported".to_string(),
                });
            }
            Language::Cxx => {
                if overrides.cpp_compat {
                    return Err(ConfigError {
                        message: "`--cpp-compat` is not supported for C++ output \
                                  (it is only meaningful for C headers)"
                            .to_string(),
                    });
                }
                if overrides.style.is_some() {
                    return Err(ConfigError {
                        message: "`--style` is not supported for C++ output \
                                  (C++ does not use typedef-style declarations)"
                            .to_string(),
                    });
                }
            }
            Language::C => {}
        }

        // Build the base CommonConfig from top-level fields.
        let base = RawCommonFields {
            header: self.header,
            trailer: self.trailer,
            autogen_warning: self.autogen_warning,
            include_guard: self.include_guard,
            pragma_once: self.pragma_once,
            sys_includes: self.sys_includes,
            includes: self.includes,
            no_includes: self.no_includes,
            after_includes: self.after_includes,
        };

        match language {
            Language::C => {
                let section = self.c.unwrap_or_default();
                let section_overrides = RawCommonOverrides {
                    header: section.header,
                    trailer: section.trailer,
                    autogen_warning: section.autogen_warning,
                    include_guard: section.include_guard,
                    pragma_once: section.pragma_once,
                    sys_includes: section.sys_includes,
                    includes: section.includes,
                    no_includes: section.no_includes,
                    after_includes: section.after_includes,
                };
                let common = base.resolve(section_overrides);

                let style = overrides
                    .style
                    .clone()
                    .or(section.style)
                    .unwrap_or(Style::Both);
                let cpp_compat = if overrides.cpp_compat {
                    true
                } else {
                    section.cpp_compat.unwrap_or(false)
                };

                Ok(Config::C(CConfig {
                    common,
                    style,
                    cpp_compat,
                }))
            }
            Language::Cxx => {
                let section = self.cxx.unwrap_or_default();
                let section_overrides = RawCommonOverrides {
                    header: section.header,
                    trailer: section.trailer,
                    autogen_warning: section.autogen_warning,
                    include_guard: section.include_guard,
                    pragma_once: section.pragma_once,
                    sys_includes: section.sys_includes,
                    includes: section.includes,
                    no_includes: section.no_includes,
                    after_includes: section.after_includes,
                };
                let common = base.resolve(section_overrides);
                Ok(Config::Cxx(CxxConfig { common }))
            }
            Language::Cython => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config() {
        let raw: RawConfig = toml::from_str("").unwrap();
        let config = raw.into_config(&Language::C, &CliOverrides::default()).unwrap();
        assert!(matches!(config, Config::C(_)));
    }

    #[test]
    fn full_c_config() {
        let toml_str = r#"
header = "/* License */"
trailer = "/* End */"
autogen_warning = "// Auto-generated"
include_guard = "MY_LIB_H"
pragma_once = false
sys_includes = ["stdint.h", "stdbool.h"]
includes = ["my_types.h"]
no_includes = false
after_includes = "/* after includes */"

[c]
style = "Tag"
cpp_compat = true
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config = raw.into_config(&Language::C, &CliOverrides::default()).unwrap();
        match config {
            Config::C(c) => {
                assert!(matches!(c.style, Style::Tag));
                assert!(c.cpp_compat);
                assert_eq!(c.common.header.as_deref(), Some("/* License */"));
                assert_eq!(c.common.trailer.as_deref(), Some("/* End */"));
                assert_eq!(
                    c.common.autogen_warning.as_deref(),
                    Some("// Auto-generated")
                );
                assert_eq!(c.common.include_guard.as_deref(), Some("MY_LIB_H"));
                assert!(!c.common.pragma_once);
                assert_eq!(c.common.sys_includes, vec!["stdint.h", "stdbool.h"]);
                assert_eq!(c.common.includes, vec!["my_types.h"]);
                assert!(!c.common.no_includes);
                assert_eq!(c.common.after_includes.as_deref(), Some("/* after includes */"));
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn full_cxx_config() {
        let toml_str = r#"
header = "/* C++ License */"
include_guard = "MY_CXX_H"
pragma_once = true

[cxx]
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config = raw
            .into_config(&Language::Cxx, &CliOverrides::default())
            .unwrap();
        match config {
            Config::Cxx(cxx) => {
                assert_eq!(cxx.common.header.as_deref(), Some("/* C++ License */"));
                assert_eq!(cxx.common.include_guard.as_deref(), Some("MY_CXX_H"));
                assert!(cxx.common.pragma_once);
            }
            _ => panic!("expected Config::Cxx"),
        }
    }

    #[test]
    fn section_overrides_common() {
        let toml_str = r#"
header = "/* Shared */"
include_guard = "SHARED_H"

[c]
include_guard = "C_ONLY_H"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config = raw.into_config(&Language::C, &CliOverrides::default()).unwrap();
        match config {
            Config::C(c) => {
                assert_eq!(c.common.header.as_deref(), Some("/* Shared */"));
                assert_eq!(c.common.include_guard.as_deref(), Some("C_ONLY_H"));
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn multi_language_config() {
        let toml_str = r#"
header = "/* Shared License */"

[c]
style = "Tag"
cpp_compat = true
include_guard = "MY_C_H"

[cxx]
include_guard = "MY_CXX_H"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();

        // Select C
        let c_config = raw
            .clone()
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();
        match &c_config {
            Config::C(c) => {
                assert!(matches!(c.style, Style::Tag));
                assert!(c.cpp_compat);
                assert_eq!(c.common.header.as_deref(), Some("/* Shared License */"));
                assert_eq!(c.common.include_guard.as_deref(), Some("MY_C_H"));
            }
            _ => panic!("expected Config::C"),
        }

        // Select C++
        let cxx_config = raw
            .into_config(&Language::Cxx, &CliOverrides::default())
            .unwrap();
        match &cxx_config {
            Config::Cxx(cxx) => {
                assert_eq!(cxx.common.header.as_deref(), Some("/* Shared License */"));
                assert_eq!(cxx.common.include_guard.as_deref(), Some("MY_CXX_H"));
            }
            _ => panic!("expected Config::Cxx"),
        }
    }

    #[test]
    fn cxx_without_section_uses_common_defaults() {
        let toml_str = r#"
header = "/* License */"
include_guard = "MY_H"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config = raw
            .into_config(&Language::Cxx, &CliOverrides::default())
            .unwrap();
        match config {
            Config::Cxx(cxx) => {
                assert_eq!(cxx.common.header.as_deref(), Some("/* License */"));
                assert_eq!(cxx.common.include_guard.as_deref(), Some("MY_H"));
            }
            _ => panic!("expected Config::Cxx"),
        }
    }

    #[test]
    fn cli_overrides_style() {
        let toml_str = r#"
[c]
style = "Tag"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let overrides = CliOverrides {
            style: Some(Style::Type),
            cpp_compat: false,
        };
        let config = raw.into_config(&Language::C, &overrides).unwrap();
        match config {
            Config::C(c) => {
                assert!(matches!(c.style, Style::Type));
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn cli_overrides_cpp_compat() {
        let raw: RawConfig = toml::from_str("").unwrap();
        let overrides = CliOverrides {
            style: None,
            cpp_compat: true,
        };
        let config = raw.into_config(&Language::C, &overrides).unwrap();
        match config {
            Config::C(c) => {
                assert!(c.cpp_compat);
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn cxx_rejects_cpp_compat_cli_override() {
        let raw: RawConfig = toml::from_str("").unwrap();
        let overrides = CliOverrides {
            style: None,
            cpp_compat: true,
        };
        let err = raw.into_config(&Language::Cxx, &overrides).unwrap_err();
        assert!(err.message.contains("cpp-compat"));
    }

    #[test]
    fn cxx_rejects_style_cli_override() {
        let raw: RawConfig = toml::from_str("").unwrap();
        let overrides = CliOverrides {
            style: Some(Style::Tag),
            cpp_compat: false,
        };
        let err = raw.into_config(&Language::Cxx, &overrides).unwrap_err();
        assert!(err.message.contains("style"));
    }

    #[test]
    fn cxx_alias_parses() {
        let toml_str = r#"
[cxx]
header = "/* C++ */"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        assert!(raw.cxx.is_some());
    }
}
