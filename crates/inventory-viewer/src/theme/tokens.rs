use serde::{Deserialize, Serialize};

/// Colección de tokens de diseño para colores, tipografía y espaciado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignTokens {
    pub colors: ColorTokens,
    pub typography: TypographyTokens,
    pub spacing: SpacingTokens,
    pub radii: RadiusTokens,
    pub shadows: ShadowTokens,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorTokens {
    pub background: String,
    pub surface: String,
    pub surface_hover: String,
    pub border: String,
    pub text: String,
    pub text_muted: String,
    pub primary: String,
    pub primary_hover: String,
    pub primary_text: String,
    pub danger: String,
    pub danger_hover: String,
    pub warning: String,
    pub success: String,
    pub info: String,
    #[serde(default = "default_nav_bg")]
    pub nav_bg: String,
    #[serde(default = "default_nav_text")]
    pub nav_text: String,
    #[serde(default = "default_nav_text_muted")]
    pub nav_text_muted: String,
    #[serde(default = "default_nav_hover")]
    pub nav_hover: String,
    #[serde(default = "default_icon_bg")]
    pub icon_bg: String,
    #[serde(default = "default_icon_fg")]
    pub icon_fg: String,
}

fn default_nav_bg() -> String { "#1E2126".to_string() }
fn default_nav_text() -> String { "#e5e7eb".to_string() }
fn default_nav_text_muted() -> String { "#9ca3af".to_string() }
fn default_nav_hover() -> String { "#2a2d33".to_string() }
fn default_icon_bg() -> String { "#2563EB".to_string() }
fn default_icon_fg() -> String { "#FFFFFF".to_string() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypographyTokens {
    pub font_family: String,
    pub font_size_sm: String,
    pub font_size_base: String,
    pub font_size_lg: String,
    pub font_size_xl: String,
    pub font_size_2xl: String,
    pub line_height: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpacingTokens {
    pub xs: String,
    pub sm: String,
    pub md: String,
    pub lg: String,
    pub xl: String,
    pub xxl: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadiusTokens {
    pub sm: String,
    pub md: String,
    pub lg: String,
    pub full: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowTokens {
    pub sm: String,
    pub md: String,
    pub lg: String,
}

impl DesignTokens {
    /// Tokens para modo claro.
    pub fn light() -> Self {
        Self::from_toml(include_str!("../../assets/tokens-light.toml"))
    }

    /// Tokens para modo oscuro.
    pub fn dark() -> Self {
        Self::from_toml(include_str!("../../assets/tokens-dark.toml"))
    }

    /// Carga tokens desde una cadena TOML.
    pub fn from_toml(toml: &str) -> Self {
        toml::from_str(toml).unwrap_or_else(|_| Self::default_tokens())
    }

    /// Genera un bloque de variables CSS listo para inyectar en un tag `<style>`.
    pub fn to_css_variables(&self) -> String {
        format!(
            ":root {{
  --bg: {bg}; --surface: {surface}; --surface-hover: {surface_hover}; --border: {border};
  --text: {text}; --text-muted: {text_muted}; --primary: {primary}; --primary-hover: {primary_hover}; --primary-text: {primary_text};
  --danger: {danger}; --danger-hover: {danger_hover}; --warning: {warning}; --success: {success}; --info: {info};
  --nav-bg: {nav_bg}; --nav-text: {nav_text}; --nav-text-muted: {nav_text_muted}; --nav-hover: {nav_hover};
  --icon-bg: {icon_bg}; --icon-fg: {icon_fg};
  --font-family: {font_family}; --font-size-sm: {font_size_sm}; --font-size-base: {font_size_base}; --font-size-lg: {font_size_lg}; --font-size-xl: {font_size_xl}; --font-size-2xl: {font_size_2xl}; --line-height: {line_height};
  --space-xs: {xs}; --space-sm: {sm}; --space-md: {md}; --space-lg: {lg}; --space-xl: {xl}; --space-xxl: {xxl};
  --radius-sm: {radius_sm}; --radius-md: {radius_md}; --radius-lg: {radius_lg}; --radius-full: {radius_full};
  --shadow-sm: {shadow_sm}; --shadow-md: {shadow_md}; --shadow-lg: {shadow_lg};
}}",
            bg = self.colors.background,
            surface = self.colors.surface,
            surface_hover = self.colors.surface_hover,
            border = self.colors.border,
            text = self.colors.text,
            text_muted = self.colors.text_muted,
            primary = self.colors.primary,
            primary_hover = self.colors.primary_hover,
            primary_text = self.colors.primary_text,
            danger = self.colors.danger,
            danger_hover = self.colors.danger_hover,
            warning = self.colors.warning,
            success = self.colors.success,
            info = self.colors.info,
            nav_bg = self.colors.nav_bg,
            nav_text = self.colors.nav_text,
            nav_text_muted = self.colors.nav_text_muted,
            nav_hover = self.colors.nav_hover,
            icon_bg = self.colors.icon_bg,
            icon_fg = self.colors.icon_fg,
            font_family = self.typography.font_family,
            font_size_sm = self.typography.font_size_sm,
            font_size_base = self.typography.font_size_base,
            font_size_lg = self.typography.font_size_lg,
            font_size_xl = self.typography.font_size_xl,
            font_size_2xl = self.typography.font_size_2xl,
            line_height = self.typography.line_height,
            xs = self.spacing.xs,
            sm = self.spacing.sm,
            md = self.spacing.md,
            lg = self.spacing.lg,
            xl = self.spacing.xl,
            xxl = self.spacing.xxl,
            radius_sm = self.radii.sm,
            radius_md = self.radii.md,
            radius_lg = self.radii.lg,
            radius_full = self.radii.full,
            shadow_sm = self.shadows.sm,
            shadow_md = self.shadows.md,
            shadow_lg = self.shadows.lg,
        )
    }

    fn default_tokens() -> Self {
        Self::light()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_tokens_load() {
        let tokens = DesignTokens::light();
        assert_eq!(tokens.colors.background, "#F5F3F0");
        assert_eq!(tokens.colors.primary, "#F59E0B");
        assert_eq!(tokens.colors.nav_bg, "#1E2126");
    }

    #[test]
    fn test_dark_tokens_load() {
        let tokens = DesignTokens::dark();
        assert_eq!(tokens.colors.background, "#1E2126");
        assert_eq!(tokens.colors.primary, "#F59E0B");
    }

    #[test]
    fn test_to_css_variables_contains_primary() {
        let css = DesignTokens::light().to_css_variables();
        assert!(css.contains("--primary"));
        assert!(css.contains("--nav-bg"));
    }
}
