use dioxus::prelude::*;

#[derive(Clone, PartialEq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Primary,
    Success,
    Warning,
    Danger,
    Info,
}

#[component]
pub fn Badge(
    children: Element,
    #[props(default = BadgeVariant::Default)] variant: BadgeVariant,
) -> Element {
    let mut classes = vec!["badge".to_string()];
    match variant {
        BadgeVariant::Primary => classes.push("badge-primary".to_string()),
        BadgeVariant::Success => classes.push("badge-success".to_string()),
        BadgeVariant::Warning => classes.push("badge-warning".to_string()),
        BadgeVariant::Danger => classes.push("badge-danger".to_string()),
        BadgeVariant::Info => classes.push("badge-info".to_string()),
        BadgeVariant::Default => {}
    }
    let class_str = classes.join(" ");

    rsx! {
        span { class: "{class_str}", {children} }
    }
}
