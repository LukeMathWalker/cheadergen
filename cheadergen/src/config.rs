use std::path::Path;

use clap::ValueEnum;
use serde::Deserialize;

/// The target language for the generated header file.
#[derive(Debug, Clone, ValueEnum, Deserialize)]
pub enum Language {
    /// Generate a C-compatible header.
    #[value(name = "c", alias = "C")]
    #[serde(alias = "c")]
    C,
    /// Generate a C++ header.
    #[value(name = "c++", alias = "C++", alias = "cpp")]
    #[serde(alias = "c++", alias = "C++", alias = "cpp", alias = "Cxx")]
    Cxx,
    /// Reserved for future Cython support. Currently rejected at validation time
    /// by [`RawConfig::into_config`].
    #[value(name = "cython", alias = "Cython")]
    #[serde(alias = "cython")]
    Cython,
}

/// The declaration style for C struct and enum definitions.
///
/// This only applies when the target language is [`Language::C`].
/// C++ does not use typedef-style declarations, so this option is ignored
/// (and rejected) for [`Language::Cxx`].
#[derive(Debug, Clone, ValueEnum, Deserialize)]
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
/// All fields are optional at this stage. Defaults are resolved and
/// language-specific constraints are validated when converting into a
/// [`Config`] via [`RawConfig::into_config`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawConfig {
    /// Target language for the generated header.
    /// Can also be set via the `--lang` CLI flag, which takes precedence.
    pub language: Option<Language>,
    /// C declaration style for structs and enums. Defaults to [`Style::Both`].
    /// Only applicable to [`Language::C`].
    pub style: Option<Style>,
    /// Wrap C output in an `extern "C"` block for C++ compatibility.
    /// Defaults to `false`. Only applicable to [`Language::C`].
    pub cpp_compat: Option<bool>,
    /// Verbatim text prepended to the generated file (e.g. a license block).
    pub header: Option<String>,
    /// Verbatim text appended to the end of the generated file.
    pub trailer: Option<String>,
    /// Warning text emitted between major sections to discourage manual edits.
    pub autogen_warning: Option<String>,
    /// Custom `#ifndef`/`#define` include guard name.
    /// When omitted, no include guard is emitted (see also [`pragma_once`](RawConfig::pragma_once)).
    pub include_guard: Option<String>,
    /// Emit `#pragma once` instead of (or in addition to) an include guard.
    /// Defaults to `false`.
    pub pragma_once: Option<bool>,
    /// System headers to emit as `#include <…>`.
    #[serde(default)]
    pub sys_includes: Vec<String>,
    /// User headers to emit as `#include "…"`.
    #[serde(default)]
    pub includes: Vec<String>,
    /// Suppress the default language-specific includes (e.g. `<stdint.h>` for C).
    /// Defaults to `false`.
    pub no_includes: Option<bool>,
    /// Verbatim text inserted immediately after the include block.
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
    /// See [`RawConfig::style`]. Defaults to [`Style::Both`].
    #[allow(dead_code)]
    pub style: Style,
    /// See [`RawConfig::cpp_compat`]. Defaults to `false`.
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
    /// `language` is required — it may come from the config file itself,
    /// from a CLI flag, or from a default.
    pub fn into_config(self, language: &Language) -> Result<Config, ConfigError> {
        match language {
            Language::Cython => {
                return Err(ConfigError {
                    message: "Cython output is not yet supported".to_string(),
                });
            }
            Language::Cxx => {
                if self.cpp_compat.is_some() {
                    return Err(ConfigError {
                        message: "`cpp_compat` is not supported for C++ output \
                                  (it is only meaningful for C headers)"
                            .to_string(),
                    });
                }
                if self.style.is_some() {
                    return Err(ConfigError {
                        message: "`style` is not supported for C++ output \
                                  (C++ does not use typedef-style declarations)"
                            .to_string(),
                    });
                }
            }
            Language::C => {}
        }

        let common = CommonConfig {
            header: self.header,
            trailer: self.trailer,
            autogen_warning: self.autogen_warning,
            include_guard: self.include_guard,
            pragma_once: self.pragma_once.unwrap_or(false),
            sys_includes: self.sys_includes,
            includes: self.includes,
            no_includes: self.no_includes.unwrap_or(false),
            after_includes: self.after_includes,
        };

        match language {
            Language::C => Ok(Config::C(CConfig {
                common,
                style: self.style.unwrap_or(Style::Both),
                cpp_compat: self.cpp_compat.unwrap_or(false),
            })),
            Language::Cxx => Ok(Config::Cxx(CxxConfig { common })),
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
        let config = raw.into_config(&Language::C).unwrap();
        assert!(matches!(config, Config::C(_)));
    }

    #[test]
    fn full_c_config() {
        let toml_str = r#"
language = "C"
style = "Tag"
cpp_compat = true
header = "/* License */"
trailer = "/* End */"
autogen_warning = "// Auto-generated"
include_guard = "MY_LIB_H"
pragma_once = false
sys_includes = ["stdint.h", "stdbool.h"]
includes = ["my_types.h"]
no_includes = false
after_includes = "/* after includes */"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        assert!(matches!(raw.language, Some(Language::C)));

        let config = raw.into_config(&Language::C).unwrap();
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
language = "C++"
header = "/* C++ License */"
include_guard = "MY_CXX_H"
pragma_once = true
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config = raw.into_config(&Language::Cxx).unwrap();
        match config {
            Config::Cxx(cxx) => {
                assert_eq!(cxx.common.header.as_deref(), Some("/* C++ License */"));
                assert_eq!(cxx.common.include_guard.as_deref(), Some("MY_CXX_H"));
                assert!(cxx.common.pragma_once);
            }
            _ => panic!("expected Config::Cxx"),
        }
    }

}
