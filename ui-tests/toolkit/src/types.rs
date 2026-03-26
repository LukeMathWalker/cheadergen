#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Cxx,
    C,
    Cython,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    Both,
    Tag,
    Type,
}

pub fn style_str(style: Style) -> &'static str {
    match style {
        Style::Both => "both",
        Style::Tag => "tag",
        Style::Type => "type",
    }
}

pub fn language_extension(lang: Language) -> &'static str {
    match lang {
        Language::C => "h",
        Language::Cxx => "hpp",
        Language::Cython => "pyx",
    }
}
