use dioxus::prelude::*;

/// Variantes visuales de un botón.
#[derive(Clone, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
    Ghost,
    Link,
}

/// Tamaños de botón.
#[derive(Clone, PartialEq, Default)]
pub enum ButtonSize {
    #[default]
    Md,
    Sm,
    Lg,
}

#[component]
pub fn Button(
    children: Element,
    #[props(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[props(default = ButtonSize::Md)] size: ButtonSize,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] loading: bool,
    onclick: Option<EventHandler<MouseEvent>>,
    class: Option<String>,
) -> Element {
    let mut classes = vec!["btn".to_string()];

    match variant {
        ButtonVariant::Primary => classes.push("btn-primary".to_string()),
        ButtonVariant::Secondary => {}
        ButtonVariant::Danger => classes.push("btn-danger".to_string()),
        ButtonVariant::Ghost => classes.push("btn-ghost".to_string()),
        ButtonVariant::Link => classes.push("btn-ghost".to_string()),
    }

    match size {
        ButtonSize::Sm => classes.push("btn-sm".to_string()),
        ButtonSize::Md => {}
        ButtonSize::Lg => classes.push("btn-lg".to_string()),
    }

    if let Some(extra) = class {
        classes.push(extra);
    }

    let class_str = classes.join(" ");

    rsx! {
        button {
            class: "{class_str}",
            disabled: disabled || loading,
            onclick: move |evt| {
                if let Some(handler) = onclick.as_ref() {
                    handler.call(evt);
                }
            },
            if loading {
                span { class: "spinner" }
            }
            {children}
        }
    }
}
