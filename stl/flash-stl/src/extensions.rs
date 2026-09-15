//! Language extensions — animation curves, themes, design tokens.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Spring { damping: f32, stiffness: f32 },
    Decay,
}

impl AnimationCurve {
    pub fn name(&self) -> &'static str {
        match self {
            AnimationCurve::Linear => "linear",
            AnimationCurve::EaseIn => "ease_in",
            AnimationCurve::EaseOut => "ease_out",
            AnimationCurve::EaseInOut => "ease_in_out",
            AnimationCurve::Spring { .. } => "spring",
            AnimationCurve::Decay => "decay",
        }
    }

    pub fn all_named() -> &'static [&'static str] {
        &["linear", "ease_in", "ease_out", "ease_in_out", "spring", "decay"]
    }
}

/// Design tokens — resolve at compile time to theme values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DesignToken {
    Primary,
    Secondary,
    Surface,
    Background,
    Error,
    OnPrimary,
    OnSurface,
    Divider,
    Accent,
}

impl DesignToken {
    pub fn name(&self) -> &'static str {
        match self {
            DesignToken::Primary => ".primary",
            DesignToken::Secondary => ".secondary",
            DesignToken::Surface => ".surface",
            DesignToken::Background => ".background",
            DesignToken::Error => ".error",
            DesignToken::OnPrimary => ".on_primary",
            DesignToken::OnSurface => ".on_surface",
            DesignToken::Divider => ".divider",
            DesignToken::Accent => ".accent",
        }
    }

    pub fn all() -> &'static [DesignToken] {
        &[
            DesignToken::Primary, DesignToken::Secondary, DesignToken::Surface,
            DesignToken::Background, DesignToken::Error, DesignToken::OnPrimary,
            DesignToken::OnSurface, DesignToken::Divider, DesignToken::Accent,
        ]
    }
}

/// Complete STL catalog for documentation generation.
pub struct StlCatalog;

impl StlCatalog {
    pub fn export_json() -> String {
        use std::fmt::Write;
        let mut out = String::from("{\n  \"modules\": [\n");
        let _ = writeln!(out, "    {{\"name\": \"collections\", \"functions\": {:?}}},",
            crate::collections::CollectionFn::all().iter().map(|f| f.name()).collect::<Vec<_>>());
        let _ = writeln!(out, "    {{\"name\": \"string\", \"functions\": {:?}}},",
            crate::string::StringFn::all().iter().map(|f| f.name()).collect::<Vec<_>>());
        let _ = writeln!(out, "    {{\"name\": \"math\", \"functions\": {:?}}},",
            crate::math::MathFn::all().iter().map(|f| f.name()).collect::<Vec<_>>());
        let _ = writeln!(out, "    {{\"name\": \"mobile\", \"functions\": {:?}}},",
            crate::mobile::MobileFn::all().iter().map(|f| f.name()).collect::<Vec<_>>());
        let _ = writeln!(out, "    {{\"name\": \"modifiers\", \"items\": {:?}}}",
            crate::modifiers::ModifierExt::all().iter().map(|m| m.name()).collect::<Vec<_>>());
        out.push_str("\n  ]\n}");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_catalog_json() {
        let json = StlCatalog::export_json();
        assert!(json.contains("\"collections\""));
        assert!(json.contains("\"mobile\""));
        // Write to site/api/stl.json for documentation site (from repo root)
        for base in ["site/api/stl.json", "../site/api/stl.json", "../../site/api/stl.json"] {
            let path = std::path::Path::new(base);
            if path.parent().map(|p| p.exists()).unwrap_or(false) {
                std::fs::write(path, &json).expect("write stl.json");
                break;
            }
        }
    }
}
