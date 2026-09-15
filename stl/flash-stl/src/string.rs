//! STL string utilities — formatting, parsing, mobile display.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StringFn {
    Trim,
    TrimStart,
    TrimEnd,
    ToUpper,
    ToLower,
    Capitalize,
    Split,
    Join,
    Replace,
    ReplaceAll,
    StartsWith,
    EndsWith,
    Contains,
    Substring,
    Format,
    ParseInt,
    ParseFloat,
    IsBlank,
    IsEmpty,
    Repeat,
    PadStart,
    PadEnd,
    Truncate,
    Ellipsize,
}

impl StringFn {
    pub fn name(&self) -> &'static str {
        match self {
            StringFn::Trim => "trim",
            StringFn::TrimStart => "trim_start",
            StringFn::TrimEnd => "trim_end",
            StringFn::ToUpper => "to_upper",
            StringFn::ToLower => "to_lower",
            StringFn::Capitalize => "capitalize",
            StringFn::Split => "split",
            StringFn::Join => "join",
            StringFn::Replace => "replace",
            StringFn::ReplaceAll => "replace_all",
            StringFn::StartsWith => "starts_with",
            StringFn::EndsWith => "ends_with",
            StringFn::Contains => "contains",
            StringFn::Substring => "substring",
            StringFn::Format => "format",
            StringFn::ParseInt => "parse_int",
            StringFn::ParseFloat => "parse_float",
            StringFn::IsBlank => "is_blank",
            StringFn::IsEmpty => "is_empty",
            StringFn::Repeat => "repeat",
            StringFn::PadStart => "pad_start",
            StringFn::PadEnd => "pad_end",
            StringFn::Truncate => "truncate",
            StringFn::Ellipsize => "ellipsize",
        }
    }

    pub fn all() -> &'static [StringFn] {
        &[
            StringFn::Trim, StringFn::TrimStart, StringFn::TrimEnd,
            StringFn::ToUpper, StringFn::ToLower, StringFn::Capitalize,
            StringFn::Split, StringFn::Join, StringFn::Replace, StringFn::ReplaceAll,
            StringFn::StartsWith, StringFn::EndsWith, StringFn::Contains,
            StringFn::Substring, StringFn::Format, StringFn::ParseInt, StringFn::ParseFloat,
            StringFn::IsBlank, StringFn::IsEmpty, StringFn::Repeat,
            StringFn::PadStart, StringFn::PadEnd, StringFn::Truncate, StringFn::Ellipsize,
        ]
    }
}
