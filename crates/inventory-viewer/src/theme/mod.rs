use dioxus::prelude::*;

pub mod tokens;
pub use tokens::DesignTokens;

/// Estado del tema compartido a través de la aplicación.
#[derive(Clone, Copy, PartialEq)]
pub struct Theme {
    pub is_dark: Signal<bool>,
}

impl Theme {
    /// Devuelve los tokens activos según el modo actual.
    pub fn tokens(&self) -> DesignTokens {
        if self.is_dark.read().clone() {
            DesignTokens::dark()
        } else {
            DesignTokens::light()
        }
    }

    /// Alterna entre modo claro y oscuro.
    pub fn toggle(&mut self) {
        let mut is_dark = self.is_dark;
        let current = *is_dark.read();
        is_dark.set(!current);
    }
}

/// Provee el tema a toda la jerarquía de componentes.
#[component]
pub fn ThemeProvider(children: Element) -> Element {
    let is_dark = use_signal(|| false);
    let theme = Theme { is_dark };
    let css = use_memo(move || theme.tokens().to_css_variables());

    use_context_provider(|| theme);

    rsx! {
        style { "{css}" }
        {children}
    }
}

/// Hook para acceder al tema desde cualquier componente.
pub fn use_theme() -> Theme {
    use_context::<Theme>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_tokens_switch() {
        // No podemos probar Signal fuera de runtime, pero sí los tokens estáticos.
        let light = DesignTokens::light();
        let dark = DesignTokens::dark();
        assert_ne!(light.colors.background, dark.colors.background);
    }
}
