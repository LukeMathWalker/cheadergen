pub mod cbindgen;

use std::collections::{BTreeMap, HashMap};
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

impl Language {
    pub fn extension(&self) -> &'static str {
        match self {
            Language::C => "h",
            Language::Cxx => "hpp",
            Language::Cython => "pyx",
        }
    }
}

/// Controls how items of a given kind are sorted in the generated header.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SortKey {
    /// Emit items in the order they appear in the Rust source file.
    #[default]
    SourceOrder,
    /// Sort items alphabetically by name.
    Name,
}

/// The comment style used when emitting Rust doc comments in the header.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentationStyle {
    /// Use C-style block comments (`/** ... */`) for C output,
    /// C++ line comments (`///`) for C++ output.
    #[default]
    Auto,
    /// Always use C-style block comments: `/** ... */`.
    C,
    /// Always use C99/C++ line comments: `// ...`.
    C99,
    /// Use Doxygen-style block comments: `/** ... */` (same as `C` currently).
    Doxy,
    /// Use C++ triple-slash comments: `/// ...`.
    Cxx,
}

/// Controls how much of the Rust doc comment is included.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentationLength {
    /// Include the entire doc comment.
    #[default]
    Full,
    /// Include only the first paragraph (up to the first blank line).
    Short,
}

/// Function-specific configuration inside the `[fn]` TOML section.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawFnSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SortKey>,
}

/// Static-specific configuration inside the `[static]` TOML section.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawStaticSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SortKey>,
}

/// Constant-specific configuration inside the `[constant]` TOML section.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawConstantSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SortKey>,
}

/// Enum-specific configuration inside the `[enum]` TOML section.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawEnumSection {
    /// When true, prefix tagged-union variant names with the enum name
    /// (e.g. `Foo_A` instead of `A`). Defaults to `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix_with_name: Option<bool>,
}

/// Per-header configuration inside a `[header.<name>]` TOML section.
///
/// Each header section can override any common option for a specific
/// crate's generated header. The `include_guard` field is only available
/// here (not at the top level or in language sections).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawHeaderSection {
    /// Custom `#ifndef`/`#define` include guard name for this header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_guard: Option<String>,

    // Common option overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preamble: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autogen_warning: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_style: Option<DocumentationStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_length: Option<DocumentationLength>,

    /// Default sort order override for this header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SortKey>,

    /// Function-specific configuration.
    #[serde(rename = "fn", skip_serializing_if = "Option::is_none")]
    pub fn_: Option<RawFnSection>,
    /// Static-specific configuration.
    #[serde(rename = "static", skip_serializing_if = "Option::is_none")]
    pub static_: Option<RawStaticSection>,
    /// Constant-specific configuration.
    #[serde(rename = "constant", skip_serializing_if = "Option::is_none")]
    pub constant_: Option<RawConstantSection>,
    /// Enum-specific configuration.
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_: Option<RawEnumSection>,

    /// C-specific configuration section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c: Option<RawCSection>,
    /// C++-specific configuration section.
    #[serde(alias = "c++", alias = "cpp", skip_serializing_if = "Option::is_none")]
    pub cxx: Option<RawCxxSection>,
}

/// Controls how types from a specific dependency package are emitted
/// in the generated header.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageTypeMode {
    /// Emit only forward declarations for all types from this package.
    Opaque,
    /// Do not emit anything for types from this package.
    /// The consumer is expected to provide definitions via included headers.
    Skip,
}

/// Per-dependency-package configuration inside a `[package.<name>]` TOML section.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawPackageConfig {
    /// How types from this package should be emitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<PackageTypeMode>,
    /// Override the on-disk base name of the generated header for this package
    /// in partitioned mode. Must be a bare filename (no path separators, no
    /// extension). Rejected in bundle mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_name: Option<String>,
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
    pub preamble: Option<String>,
    /// Verbatim text appended to the end of the generated file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<String>,
    /// Warning text emitted between major sections to discourage manual edits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autogen_warning: Option<String>,
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

    /// Whether to emit Rust doc comments in the generated header.
    /// Defaults to `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<bool>,
    /// Comment style for doc comments. Defaults to [`DocumentationStyle::Auto`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_style: Option<DocumentationStyle>,
    /// How much of the doc comment to include. Defaults to [`DocumentationLength::Full`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_length: Option<DocumentationLength>,

    /// Default sort order for all item kinds.
    /// Can be overridden per-kind via `[fn]` or `[static]` sections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SortKey>,

    /// Function-specific configuration.
    #[serde(rename = "fn", skip_serializing_if = "Option::is_none")]
    pub fn_: Option<RawFnSection>,
    /// Static-specific configuration.
    #[serde(rename = "static", skip_serializing_if = "Option::is_none")]
    pub static_: Option<RawStaticSection>,
    /// Constant-specific configuration.
    #[serde(rename = "constant", skip_serializing_if = "Option::is_none")]
    pub constant_: Option<RawConstantSection>,
    /// Enum-specific configuration.
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_: Option<RawEnumSection>,

    /// Produce a single combined header per target, inlining all dependency
    /// types instead of emitting per-crate headers with `#include` directives.
    /// Defaults to `false` (partitioned mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle: Option<bool>,

    /// Per-dependency-package configuration.
    ///
    /// Keys are crate names (e.g. `my-dep`) or Cargo-style `name@version`
    /// specifiers for disambiguation (e.g. `"foo@1.0"`). Stored as a
    /// `BTreeMap` so iteration order is deterministic (alphabetical).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub package: BTreeMap<String, RawPackageConfig>,

    /// Per-header configuration sections.
    ///
    /// Keys are crate names. Each section can override common options
    /// for that crate's generated header.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub header: HashMap<String, RawHeaderSection>,

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
    pub preamble: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autogen_warning: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_style: Option<DocumentationStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_length: Option<DocumentationLength>,
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
    pub preamble: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autogen_warning: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_style: Option<DocumentationStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_length: Option<DocumentationLength>,
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

/// A set of validated configs: one default plus optional per-header overrides.
///
/// Produced by [`RawConfig::into_config`]. Use [`ConfigSet::for_header`] to
/// look up the config for a specific generated header (falls back to the default).
#[derive(Debug, Clone)]
pub struct ConfigSet {
    /// Config for headers without a `[header.<name>]` section.
    pub default: Config,
    /// Per-header configs, keyed by the final on-disk base name of the header
    /// (i.e. the name produced by `HeaderFilenames::base_name`).
    pub per_header: HashMap<String, Config>,
    /// Whether to produce a single combined header per target (bundle mode).
    pub bundle: bool,
    /// Header rename overrides, keyed by the `[package.<name>]` section key
    /// (crate name, or `name@version` for disambiguation). Values are the
    /// final on-disk base name (no extension, no path separators).
    pub header_renames: HashMap<String, String>,
}

impl ConfigSet {
    /// Look up the config for a header by its final base name.
    ///
    /// Returns the per-header config if one exists, otherwise the default.
    pub fn for_header(&self, base_name: &str) -> &Config {
        self.per_header.get(base_name).unwrap_or(&self.default)
    }

    /// Returns the names of all `[header.<name>]` sections.
    pub fn header_names(&self) -> impl Iterator<Item = &str> {
        self.per_header.keys().map(|s| s.as_str())
    }
}

/// Validated options shared across all target languages.
///
/// Fields mirror [`RawConfig`] but with defaults resolved
/// (e.g. `Option<bool>` becomes `bool`).
#[derive(Debug, Clone)]
pub struct CommonConfig {
    /// See [`RawConfig::preamble`].
    pub preamble: Option<String>,
    /// See [`RawConfig::trailer`].
    pub trailer: Option<String>,
    /// See [`RawConfig::autogen_warning`].
    pub autogen_warning: Option<String>,
    /// Custom `#ifndef`/`#define` include guard name.
    /// Only available in per-header `[header.<name>]` sections.
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
    /// Resolved sort order for functions: `[fn].sort_by` → top-level `sort_by` → `SourceOrder`.
    pub fn_sort_by: SortKey,
    /// Resolved sort order for statics: `[static].sort_by` → top-level `sort_by` → `SourceOrder`.
    pub static_sort_by: SortKey,
    /// Resolved sort order for constants: `[constant].sort_by` → top-level `sort_by` → `SourceOrder`.
    pub constant_sort_by: SortKey,
    /// Whether to emit Rust doc comments. Defaults to `true`.
    pub documentation: bool,
    /// Comment style for doc comments.
    pub documentation_style: DocumentationStyle,
    /// How much of the doc comment to include.
    pub documentation_length: DocumentationLength,
    /// Per-dependency-package configuration, keyed by the raw config key
    /// (crate name or `name@version`).
    pub package_configs: HashMap<String, PackageConfig>,
}

/// Validated per-dependency-package configuration.
#[derive(Debug, Clone)]
pub struct PackageConfig {
    /// How types from this package should be emitted.
    pub types: PackageTypeMode,
}

/// C-specific configuration, including options that are only meaningful for
/// [`Language::C`] output (e.g. [`Style`] and `cpp_compat`).
#[derive(Debug, Clone)]
pub struct CConfig {
    /// Options shared with all languages.
    pub common: CommonConfig,
    /// See [`RawCSection::style`]. Defaults to [`Style::Both`].
    pub style: Style,
    /// See [`RawCSection::cpp_compat`]. Defaults to `false`.
    pub cpp_compat: bool,
    /// Whether to prefix tagged-union variant names with the enum name.
    /// Defaults to `false`.
    pub enum_prefix_with_name: bool,
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
#[derive(Clone)]
struct RawCommonFields {
    preamble: Option<String>,
    trailer: Option<String>,
    autogen_warning: Option<String>,
    pragma_once: Option<bool>,
    sys_includes: Vec<String>,
    includes: Vec<String>,
    no_includes: Option<bool>,
    after_includes: Option<String>,
    sort_by: Option<SortKey>,
    fn_sort_by: Option<SortKey>,
    static_sort_by: Option<SortKey>,
    constant_sort_by: Option<SortKey>,
    documentation: Option<bool>,
    documentation_style: Option<DocumentationStyle>,
    documentation_length: Option<DocumentationLength>,
    package_configs: HashMap<String, PackageConfig>,
}

/// Optional overrides from a language section that can replace top-level
/// common field values.
struct RawCommonOverrides {
    preamble: Option<String>,
    trailer: Option<String>,
    autogen_warning: Option<String>,
    pragma_once: Option<bool>,
    sys_includes: Option<Vec<String>>,
    includes: Option<Vec<String>>,
    no_includes: Option<bool>,
    after_includes: Option<String>,
    documentation: Option<bool>,
    documentation_style: Option<DocumentationStyle>,
    documentation_length: Option<DocumentationLength>,
}

impl RawCommonOverrides {
    /// Merge `other` on top of `self`. Values in `other` win when present.
    fn merge(self, other: RawCommonOverrides) -> RawCommonOverrides {
        RawCommonOverrides {
            preamble: other.preamble.or(self.preamble),
            trailer: other.trailer.or(self.trailer),
            autogen_warning: other.autogen_warning.or(self.autogen_warning),
            pragma_once: other.pragma_once.or(self.pragma_once),
            sys_includes: other.sys_includes.or(self.sys_includes),
            includes: other.includes.or(self.includes),
            no_includes: other.no_includes.or(self.no_includes),
            after_includes: other.after_includes.or(self.after_includes),
            documentation: other.documentation.or(self.documentation),
            documentation_style: other.documentation_style.or(self.documentation_style),
            documentation_length: other.documentation_length.or(self.documentation_length),
        }
    }
}

impl RawCommonFields {
    /// Merge section overrides onto these base fields, producing a validated
    /// [`CommonConfig`]. Section values win when present; otherwise the
    /// top-level value is used.
    ///
    /// `include_guard` is passed separately because it is only available
    /// in per-header sections, not at the global or language-section level.
    fn resolve(self, overrides: RawCommonOverrides, include_guard: Option<String>) -> CommonConfig {
        CommonConfig {
            preamble: overrides.preamble.or(self.preamble),
            trailer: overrides.trailer.or(self.trailer),
            autogen_warning: overrides.autogen_warning.or(self.autogen_warning),
            include_guard,
            pragma_once: overrides.pragma_once.or(self.pragma_once).unwrap_or(false),
            sys_includes: overrides.sys_includes.unwrap_or(self.sys_includes),
            includes: overrides.includes.unwrap_or(self.includes),
            no_includes: overrides.no_includes.or(self.no_includes).unwrap_or(false),
            after_includes: overrides.after_includes.or(self.after_includes),
            fn_sort_by: self.fn_sort_by.or(self.sort_by).unwrap_or_default(),
            static_sort_by: self.static_sort_by.or(self.sort_by).unwrap_or_default(),
            constant_sort_by: self.constant_sort_by.or(self.sort_by).unwrap_or_default(),
            documentation: overrides
                .documentation
                .or(self.documentation)
                .unwrap_or(true),
            documentation_style: overrides
                .documentation_style
                .or(self.documentation_style)
                .unwrap_or_default(),
            documentation_length: overrides
                .documentation_length
                .or(self.documentation_length)
                .unwrap_or_default(),
            package_configs: self.package_configs,
        }
    }
}

/// Trait for language-specific sections that can provide common option overrides.
trait IntoCommonOverrides {
    fn into_common_overrides(self) -> RawCommonOverrides;
}

impl IntoCommonOverrides for RawCSection {
    fn into_common_overrides(self) -> RawCommonOverrides {
        RawCommonOverrides {
            preamble: self.preamble,
            trailer: self.trailer,
            autogen_warning: self.autogen_warning,
            pragma_once: self.pragma_once,
            sys_includes: self.sys_includes,
            includes: self.includes,
            no_includes: self.no_includes,
            after_includes: self.after_includes,
            documentation: self.documentation,
            documentation_style: self.documentation_style,
            documentation_length: self.documentation_length,
        }
    }
}

impl IntoCommonOverrides for RawCxxSection {
    fn into_common_overrides(self) -> RawCommonOverrides {
        RawCommonOverrides {
            preamble: self.preamble,
            trailer: self.trailer,
            autogen_warning: self.autogen_warning,
            pragma_once: self.pragma_once,
            sys_includes: self.sys_includes,
            includes: self.includes,
            no_includes: self.no_includes,
            after_includes: self.after_includes,
            documentation: self.documentation,
            documentation_style: self.documentation_style,
            documentation_length: self.documentation_length,
        }
    }
}

impl IntoCommonOverrides for RawHeaderSection {
    fn into_common_overrides(self) -> RawCommonOverrides {
        RawCommonOverrides {
            preamble: self.preamble,
            trailer: self.trailer,
            autogen_warning: self.autogen_warning,
            pragma_once: self.pragma_once,
            sys_includes: self.sys_includes,
            includes: self.includes,
            no_includes: self.no_includes,
            after_includes: self.after_includes,
            documentation: self.documentation,
            documentation_style: self.documentation_style,
            documentation_length: self.documentation_length,
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
        let contents = fs_err::read_to_string(path).map_err(|e| ConfigError {
            message: format!("failed to read config file: {e}"),
        })?;
        toml::from_str(&contents).map_err(|e| ConfigError {
            message: format!("failed to parse config file: {e}"),
        })
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
    ) -> Result<ConfigSet, ConfigError> {
        match language {
            Language::Cython => {
                return Err(ConfigError {
                    message: "Cython output is not yet supported".to_string(),
                });
            }
            Language::Cxx => {
                return Err(ConfigError {
                    message: "C++ output is not yet supported".to_string(),
                });
            }
            Language::C => {}
        }

        let bundle = self.bundle.unwrap_or(false);

        // Extract header renames and per-package type configs from [package.<name>]
        // sections. Iteration is over a BTreeMap, so error messages naming
        // multiple packages stay deterministic.
        let mut package_configs: HashMap<String, PackageConfig> = HashMap::new();
        let mut header_renames: HashMap<String, String> = HashMap::new();
        let mut rename_targets: HashMap<String, String> = HashMap::new();
        for (key, raw) in self.package {
            if let Some(types) = raw.types {
                package_configs.insert(key.clone(), PackageConfig { types });
            }
            if let Some(header_name) = raw.header_name {
                if bundle {
                    return Err(ConfigError {
                        message: format!(
                            "`header_name` is not supported in bundle mode \
                             (set on `[package.\"{key}\"]`)"
                        ),
                    });
                }
                if header_name.is_empty() {
                    return Err(ConfigError {
                        message: format!(
                            "`header_name` on `[package.\"{key}\"]` must not be empty"
                        ),
                    });
                }
                if header_name.contains(['/', '\\']) {
                    return Err(ConfigError {
                        message: format!(
                            "`header_name` on `[package.\"{key}\"]` must not contain \
                             path separators: `{header_name}`"
                        ),
                    });
                }
                if header_name.contains('.') {
                    return Err(ConfigError {
                        message: format!(
                            "`header_name` on `[package.\"{key}\"]` must not include \
                             a file extension (got `{header_name}`); the language extension \
                             is appended automatically"
                        ),
                    });
                }
                if let Some(prev_key) = rename_targets.insert(header_name.clone(), key.clone()) {
                    return Err(ConfigError {
                        message: format!(
                            "`header_name = \"{header_name}\"` is set on both \
                             `[package.\"{prev_key}\"]` and `[package.\"{key}\"]`"
                        ),
                    });
                }
                header_renames.insert(key, header_name);
            }
        }

        // Build the base CommonConfig from top-level fields.
        let base = RawCommonFields {
            preamble: self.preamble,
            trailer: self.trailer,
            autogen_warning: self.autogen_warning,
            pragma_once: self.pragma_once,
            sys_includes: self.sys_includes,
            includes: self.includes,
            no_includes: self.no_includes,
            after_includes: self.after_includes,
            sort_by: self.sort_by,
            fn_sort_by: self.fn_.and_then(|s| s.sort_by),
            static_sort_by: self.static_.and_then(|s| s.sort_by),
            constant_sort_by: self.constant_.and_then(|s| s.sort_by),
            documentation: self.documentation,
            documentation_style: self.documentation_style,
            documentation_length: self.documentation_length,
            package_configs,
        };

        // Build the default config (no include_guard, no per-header overrides).
        let default = Self::build_config(
            language,
            overrides,
            &base,
            &self.c,
            &self.cxx,
            self.enum_.as_ref(),
            None, // no per-header overrides
            None, // no per-header C section
            None, // no per-header C++ section
            None, // no include_guard
        )?;

        // Build per-header configs.
        let mut per_header = HashMap::new();
        for (name, mut header_section) in self.header {
            let include_guard = header_section.include_guard.take();

            // Merge header-level item-kind overrides with global defaults.
            let header_sort_by = header_section.sort_by;
            let header_fn_sort_by = header_section.fn_.as_ref().and_then(|s| s.sort_by);
            let header_static_sort_by = header_section.static_.as_ref().and_then(|s| s.sort_by);
            let header_constant_sort_by = header_section.constant_.as_ref().and_then(|s| s.sort_by);

            // Build a modified base with per-header sort overrides.
            let mut header_base = base.clone();
            if let Some(sort_by) = header_sort_by {
                header_base.sort_by = Some(sort_by);
            }
            if let Some(fn_sort_by) = header_fn_sort_by {
                header_base.fn_sort_by = Some(fn_sort_by);
            }
            if let Some(static_sort_by) = header_static_sort_by {
                header_base.static_sort_by = Some(static_sort_by);
            }
            if let Some(constant_sort_by) = header_constant_sort_by {
                header_base.constant_sort_by = Some(constant_sort_by);
            }

            // Extract per-header language and enum sections before consuming header_section.
            let header_c = header_section.c.take();
            let header_cxx = header_section.cxx.take();
            let header_enum = header_section.enum_.take();
            let effective_enum = header_enum.as_ref().or(self.enum_.as_ref());

            let header_config = Self::build_config(
                language,
                overrides,
                &header_base,
                &self.c,
                &self.cxx,
                effective_enum,
                Some(header_section),
                header_c,
                header_cxx,
                include_guard,
            )?;

            per_header.insert(name, header_config);
        }

        Ok(ConfigSet {
            default,
            per_header,
            bundle,
            header_renames,
        })
    }

    /// Build a single [`Config`] for a given language, merging overrides.
    ///
    /// When `header_section` is `Some`, its common overrides are merged
    /// on top of the language-section overrides.
    /// Build a single [`Config`] for a given language, merging overrides.
    ///
    /// Override priority (first wins):
    /// 1. CLI flags
    /// 2. `header_c_section` / `header_cxx_section` (per-header language)
    /// 3. `header_section` (per-header common)
    /// 4. `c_section` / `cxx_section` (global language)
    /// 5. `base` (top-level defaults)
    #[allow(clippy::too_many_arguments)]
    fn build_config(
        language: &Language,
        cli_overrides: &CliOverrides,
        base: &RawCommonFields,
        c_section: &Option<RawCSection>,
        cxx_section: &Option<RawCxxSection>,
        enum_section: Option<&RawEnumSection>,
        header_section: Option<RawHeaderSection>,
        header_c_section: Option<RawCSection>,
        header_cxx_section: Option<RawCxxSection>,
        include_guard: Option<String>,
    ) -> Result<Config, ConfigError> {
        match language {
            Language::C => {
                let global_section = c_section.clone().unwrap_or_default();
                // For style/cpp_compat: header-level [c] wins over global [c],
                // CLI wins over both.
                let style = cli_overrides
                    .style
                    .clone()
                    .or_else(|| header_c_section.as_ref().and_then(|s| s.style.clone()))
                    .or(global_section.style.clone())
                    .unwrap_or(Style::Both);
                let cpp_compat = if cli_overrides.cpp_compat {
                    true
                } else {
                    header_c_section
                        .as_ref()
                        .and_then(|s| s.cpp_compat)
                        .or(global_section.cpp_compat)
                        .unwrap_or(false)
                };
                let enum_prefix_with_name = enum_section
                    .and_then(|s| s.prefix_with_name)
                    .unwrap_or(false);

                // Merge: global [c] → header common → header [c]
                let mut merged = global_section.into_common_overrides();
                if let Some(hs) = header_section {
                    merged = merged.merge(hs.into_common_overrides());
                }
                if let Some(hc) = header_c_section {
                    merged = merged.merge(hc.into_common_overrides());
                }
                let common = base.clone().resolve(merged, include_guard);

                Ok(Config::C(CConfig {
                    common,
                    style,
                    cpp_compat,
                    enum_prefix_with_name,
                }))
            }
            Language::Cxx => {
                let global_section = cxx_section.clone().unwrap_or_default();

                let mut merged = global_section.into_common_overrides();
                if let Some(hs) = header_section {
                    merged = merged.merge(hs.into_common_overrides());
                }
                if let Some(hcxx) = header_cxx_section {
                    merged = merged.merge(hcxx.into_common_overrides());
                }
                let common = base.clone().resolve(merged, include_guard);
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
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();
        assert!(matches!(config_set.default, Config::C(_)));
    }

    #[test]
    fn full_c_config() {
        let toml_str = r#"
preamble = "/* License */"
trailer = "/* End */"
autogen_warning = "// Auto-generated"
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
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();
        match config_set.default {
            Config::C(c) => {
                assert!(matches!(c.style, Style::Tag));
                assert!(c.cpp_compat);
                assert_eq!(c.common.preamble.as_deref(), Some("/* License */"));
                assert_eq!(c.common.trailer.as_deref(), Some("/* End */"));
                assert_eq!(
                    c.common.autogen_warning.as_deref(),
                    Some("// Auto-generated")
                );
                assert!(c.common.include_guard.is_none());
                assert!(!c.common.pragma_once);
                assert_eq!(c.common.sys_includes, vec!["stdint.h", "stdbool.h"]);
                assert_eq!(c.common.includes, vec!["my_types.h"]);
                assert!(!c.common.no_includes);
                assert_eq!(
                    c.common.after_includes.as_deref(),
                    Some("/* after includes */")
                );
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn full_cxx_config_rejected() {
        let toml_str = r#"
preamble = "/* C++ License */"
pragma_once = true

[cxx]
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let err = raw
            .into_config(&Language::Cxx, &CliOverrides::default())
            .unwrap_err();
        assert!(err.message.contains("C++ output is not yet supported"));
    }

    #[test]
    fn section_overrides_common() {
        let toml_str = r#"
preamble = "/* Shared */"
pragma_once = true

[c]
pragma_once = false
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();
        match config_set.default {
            Config::C(c) => {
                assert_eq!(c.common.preamble.as_deref(), Some("/* Shared */"));
                assert!(!c.common.pragma_once);
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn multi_language_config() {
        let toml_str = r#"
preamble = "/* Shared License */"

[c]
style = "Tag"
cpp_compat = true

[cxx]
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();

        // Select C
        let c_config_set = raw
            .clone()
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();
        match &c_config_set.default {
            Config::C(c) => {
                assert!(matches!(c.style, Style::Tag));
                assert!(c.cpp_compat);
                assert_eq!(c.common.preamble.as_deref(), Some("/* Shared License */"));
                assert!(c.common.include_guard.is_none());
            }
            _ => panic!("expected Config::C"),
        }

        // Select C++ — rejected
        let err = raw
            .into_config(&Language::Cxx, &CliOverrides::default())
            .unwrap_err();
        assert!(err.message.contains("C++ output is not yet supported"));
    }

    #[test]
    fn cxx_rejected() {
        let raw: RawConfig = toml::from_str("").unwrap();
        let err = raw
            .into_config(&Language::Cxx, &CliOverrides::default())
            .unwrap_err();
        assert!(err.message.contains("C++ output is not yet supported"));
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
        let config_set = raw.into_config(&Language::C, &overrides).unwrap();
        match config_set.default {
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
        let config_set = raw.into_config(&Language::C, &overrides).unwrap();
        match config_set.default {
            Config::C(c) => {
                assert!(c.cpp_compat);
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn cxx_alias_parses() {
        let toml_str = r#"
[cxx]
preamble = "/* C++ */"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        assert!(raw.cxx.is_some());
    }

    #[test]
    fn package_opaque_mode() {
        let toml_str = r#"
[package.my-dep]
types = "opaque"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(
            raw.package["my-dep"].types,
            Some(PackageTypeMode::Opaque)
        );
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();
        match config_set.default {
            Config::C(c) => {
                assert_eq!(
                    c.common.package_configs["my-dep"].types,
                    PackageTypeMode::Opaque
                );
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn package_skip_mode() {
        let toml_str = r#"
[package.other-dep]
types = "skip"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(
            raw.package["other-dep"].types,
            Some(PackageTypeMode::Skip)
        );
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();
        match config_set.default {
            Config::C(c) => {
                assert_eq!(
                    c.common.package_configs["other-dep"].types,
                    PackageTypeMode::Skip
                );
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn package_versioned_key() {
        let toml_str = r#"
[package."foo@1.0"]
types = "opaque"

[package."foo@2.0"]
types = "skip"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(raw.package.len(), 2);
        assert_eq!(
            raw.package["foo@1.0"].types,
            Some(PackageTypeMode::Opaque)
        );
        assert_eq!(
            raw.package["foo@2.0"].types,
            Some(PackageTypeMode::Skip)
        );
    }

    #[test]
    fn package_empty_section_accepted() {
        let toml_str = r#"
[package.my-dep]
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        assert!(raw.package.contains_key("my-dep"));
        assert_eq!(raw.package["my-dep"].types, None);
        // Empty section produces no PackageConfig entry (types is None → filtered out)
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();
        match config_set.default {
            Config::C(c) => {
                assert!(c.common.package_configs.is_empty());
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn package_unknown_field_rejected() {
        let toml_str = r#"
[package.my-dep]
typos = "opaque"
"#;
        let result: Result<RawConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn package_with_other_config() {
        let toml_str = r#"
preamble = "/* License */"

[package.my-dep]
types = "opaque"

[c]
style = "Tag"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();
        match config_set.default {
            Config::C(c) => {
                assert_eq!(c.common.preamble.as_deref(), Some("/* License */"));
                assert!(matches!(c.style, Style::Tag));
                assert_eq!(
                    c.common.package_configs["my-dep"].types,
                    PackageTypeMode::Opaque
                );
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn include_guard_rejected_at_global_level() {
        let toml_str = r#"
include_guard = "FOO_H"
"#;
        let result: Result<RawConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn header_section_basic() {
        let toml_str = r#"
preamble = "/* Global */"

[header.my-lib]
include_guard = "MY_LIB_H"
preamble = "/* My Lib */"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();

        // Default config has no include_guard and global preamble.
        match &config_set.default {
            Config::C(c) => {
                assert!(c.common.include_guard.is_none());
                assert_eq!(c.common.preamble.as_deref(), Some("/* Global */"));
            }
            _ => panic!("expected Config::C"),
        }

        // Per-header config has include_guard and overridden preamble.
        match config_set.for_header("my-lib") {
            Config::C(c) => {
                assert_eq!(c.common.include_guard.as_deref(), Some("MY_LIB_H"));
                assert_eq!(c.common.preamble.as_deref(), Some("/* My Lib */"));
            }
            _ => panic!("expected Config::C"),
        }

        // Unknown crate falls back to default.
        match config_set.for_header("unknown") {
            Config::C(c) => {
                assert!(c.common.include_guard.is_none());
                assert_eq!(c.common.preamble.as_deref(), Some("/* Global */"));
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn header_section_inherits_global_language() {
        let toml_str = r#"
[c]
style = "Tag"
cpp_compat = true

[header.my-lib]
include_guard = "MY_LIB_H"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();

        // Per-header config inherits global [c] settings.
        match config_set.for_header("my-lib") {
            Config::C(c) => {
                assert!(matches!(c.style, Style::Tag));
                assert!(c.cpp_compat);
                assert_eq!(c.common.include_guard.as_deref(), Some("MY_LIB_H"));
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn header_section_language_overrides_global() {
        let toml_str = r#"
[c]
style = "Tag"
cpp_compat = false

[header.my-lib]
include_guard = "MY_LIB_H"

[header.my-lib.c]
cpp_compat = true
style = "Type"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();

        // Default config uses global [c] settings.
        match &config_set.default {
            Config::C(c) => {
                assert!(matches!(c.style, Style::Tag));
                assert!(!c.cpp_compat);
            }
            _ => panic!("expected Config::C"),
        }

        // Per-header config overrides global [c] settings.
        match config_set.for_header("my-lib") {
            Config::C(c) => {
                assert!(matches!(c.style, Style::Type));
                assert!(c.cpp_compat);
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn header_section_common_overrides_global() {
        let toml_str = r#"
pragma_once = true
documentation = false

[header.my-lib]
pragma_once = false
documentation = true
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();

        match &config_set.default {
            Config::C(c) => {
                assert!(c.common.pragma_once);
                assert!(!c.common.documentation);
            }
            _ => panic!("expected Config::C"),
        }

        match config_set.for_header("my-lib") {
            Config::C(c) => {
                assert!(!c.common.pragma_once);
                assert!(c.common.documentation);
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn header_section_override_priority() {
        // Tests the full priority chain:
        // [header.my-lib.c] > [header.my-lib] > [c] > top-level
        let toml_str = r#"
preamble = "/* top-level */"
documentation = false

[c]
preamble = "/* global-c */"

[header.my-lib]
preamble = "/* header-common */"
documentation = true

[header.my-lib.c]
preamble = "/* header-c */"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();

        // Default: [c] overrides top-level
        match &config_set.default {
            Config::C(c) => {
                assert_eq!(c.common.preamble.as_deref(), Some("/* global-c */"));
                assert!(!c.common.documentation);
            }
            _ => panic!("expected Config::C"),
        }

        // Per-header: [header.my-lib.c] > [header.my-lib] > [c] > top-level
        match config_set.for_header("my-lib") {
            Config::C(c) => {
                assert_eq!(c.common.preamble.as_deref(), Some("/* header-c */"));
                // documentation comes from [header.my-lib] common (not overridden in [header.my-lib.c])
                assert!(c.common.documentation);
            }
            _ => panic!("expected Config::C"),
        }
    }

    #[test]
    fn header_section_unknown_field_rejected() {
        let toml_str = r#"
[header.my-lib]
typo_field = "value"
"#;
        let result: Result<RawConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn header_section_empty_is_valid() {
        let toml_str = r#"
[header.my-lib]
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();
        // Empty header section produces a per-header config identical to default.
        assert!(config_set.per_header.contains_key("my-lib"));
    }

    #[test]
    fn multiple_header_sections() {
        let toml_str = r#"
preamble = "/* Global */"

[header.lib-a]
include_guard = "LIB_A_H"
preamble = "/* Lib A */"

[header.lib-b]
include_guard = "LIB_B_H"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let config_set = raw
            .into_config(&Language::C, &CliOverrides::default())
            .unwrap();

        match config_set.for_header("lib-a") {
            Config::C(c) => {
                assert_eq!(c.common.include_guard.as_deref(), Some("LIB_A_H"));
                assert_eq!(c.common.preamble.as_deref(), Some("/* Lib A */"));
            }
            _ => panic!("expected Config::C"),
        }

        match config_set.for_header("lib-b") {
            Config::C(c) => {
                assert_eq!(c.common.include_guard.as_deref(), Some("LIB_B_H"));
                // Inherits global preamble since not overridden.
                assert_eq!(c.common.preamble.as_deref(), Some("/* Global */"));
            }
            _ => panic!("expected Config::C"),
        }
    }
}
