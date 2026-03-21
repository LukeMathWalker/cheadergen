use std::sync::LazyLock;

pub struct Variant {
    pub module_path: &'static [&'static str],
    pub lang: &'static str,
    pub style: &'static str,
    pub cpp_compat: bool,
    pub file_pattern: &'static str,
}

pub const VARIANTS: &[Variant] = &[
    Variant {
        module_path: &["c", "plain"],
        lang: "Language::C",
        style: "Some(Style::Type)",
        cpp_compat: false,
        file_pattern: "{name}.c",
    },
    Variant {
        module_path: &["c", "tag"],
        lang: "Language::C",
        style: "Some(Style::Tag)",
        cpp_compat: false,
        file_pattern: "{name}_tag.c",
    },
    Variant {
        module_path: &["c", "both"],
        lang: "Language::C",
        style: "Some(Style::Both)",
        cpp_compat: false,
        file_pattern: "{name}_both.c",
    },
    Variant {
        module_path: &["c", "compat"],
        lang: "Language::C",
        style: "Some(Style::Type)",
        cpp_compat: true,
        file_pattern: "{name}.compat.c",
    },
    Variant {
        module_path: &["c", "tag_compat"],
        lang: "Language::C",
        style: "Some(Style::Tag)",
        cpp_compat: true,
        file_pattern: "{name}_tag.compat.c",
    },
    Variant {
        module_path: &["c", "both_compat"],
        lang: "Language::C",
        style: "Some(Style::Both)",
        cpp_compat: true,
        file_pattern: "{name}_both.compat.c",
    },
    Variant {
        module_path: &["cpp", "plain"],
        lang: "Language::Cxx",
        style: "None",
        cpp_compat: false,
        file_pattern: "{name}.cpp",
    },
    Variant {
        module_path: &["cython", "plain"],
        lang: "Language::Cython",
        style: "Some(Style::Type)",
        cpp_compat: false,
        file_pattern: "{name}.pyx",
    },
    Variant {
        module_path: &["cython", "tag"],
        lang: "Language::Cython",
        style: "Some(Style::Tag)",
        cpp_compat: false,
        file_pattern: "{name}_tag.pyx",
    },
];

static VARIANT_PATH_STRINGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut paths: Vec<String> = VARIANTS.iter().map(|v| v.module_path.join("/")).collect();
    paths.push("symbol".to_owned());
    paths
});

/// All valid variant path strings, including "symbol".
pub fn variant_path_strings() -> &'static [String] {
    &VARIANT_PATH_STRINGS
}
