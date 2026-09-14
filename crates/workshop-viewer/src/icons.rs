use dioxus::prelude::*;

/// Nombres de iconos disponibles en la aplicación.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum IconName {
    Home,
    Logout,
    Plus,
    Edit,
    Trash,
    Search,
    ChevronLeft,
    User,
    Users,
    Wrench,
    ShoppingCart,
    Package,
    Truck,
    ChartBar,
    DocumentText,
    Cash,
    Key,
    Eye,
    Download,
    Ban,
    Unlink,
}

impl IconName {
    /// Renderiza el icono como un elemento SVG estilo Heroicons outline.
    pub fn render(self) -> Element {
        match self {
            IconName::Home => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" }
                }
            },
            IconName::Eye => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M2.036 12.322a1.012 1.012 0 0 1 0-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z" }
                    path { d: "M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0z" }
                }
            },
            IconName::Download => rsx! {
                    svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                        path { d: "M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12 12 16.5m0 0L7.5 12m4.5 4.5V3" }
                    }
                },
            IconName::Logout => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" }
                    polyline { points: "16 17 21 12 16 7" }
                    line { x1: "21", y1: "12", x2: "9", y2: "12" }
                }
            },
            IconName::Plus => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M12 4.5v15m7.5-7.5h-15" }
                }
            },
            IconName::Edit => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M16.862 4.487l1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931z" }
                }
            },
            IconName::Trash => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397" }
                }
            },
            IconName::Search => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M21 21l-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607z" }
                }
            },
            IconName::ChevronLeft => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M15.75 19.5 8.25 12l7.5-7.5" }
                }
            },
            IconName::User => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0zM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632z" }
                }
            },
            IconName::Users => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M15 19.128a9.38 9.38 0 0 0 2.625.372 9.337 9.337 0 0 0 4.121-.952 4.125 4.125 0 0 0-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 0 1 8.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0 1 11.964-3.07M12 6.375a3.375 3.375 0 1 1-6.75 0 3.375 3.375 0 0 1 6.75 0Zm8.25 2.25a2.625 2.625 0 1 1-5.25 0 2.625 2.625 0 0 1 5.25 0Z" }
                }
            },
            IconName::Wrench => rsx! {
                svg {
                    xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round", stroke_linejoin: "round",
                    path {
                        d: "M14.7 6.3a4.5 4.5 0 0 0-5.9 5.9l-5.2 5.2a2.1 2.1 0 1 0 3 3l5.2-5.2a4.5 4.5 0 0 0 5.9-5.9l-2.8 2.8-2.1-.7-.7-2.1 2.6-3z"
                    }
                }
            },
            IconName::ShoppingCart => rsx! {
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    class: "icon",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "1.5",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",

                    path { d: "M3 3h2l.4 2M7 13h10l4-8H5.4" }
                    path { d: "M7 13L5.4 5M7 13l-2.293 2.293c-.63.63-.184 1.707.707 1.707H17" }
                    circle { cx: "7.5", cy: "19.5", r: "1.5" }
                    circle { cx: "16.5", cy: "19.5", r: "1.5" }
                }
            },
            IconName::Package => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M20.25 7.5l-.625 10.632a2.25 2.25 0 0 1-2.247 2.118H6.622a2.25 2.25 0 0 1-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z" }
                }
            },
            IconName::Truck => rsx! {
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    class: "icon",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "1.5",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",

                    path { d: "M2.25 6.75h12v9h-12z" }
                    path { d: "M14.25 9.75h3.25l2.75 3.5v2.5h-6z" }
                    path { d: "M2.25 15.75h1.5m4.5 0h6m4.5 0h1.5" }
                    circle { cx: "6", cy: "17.25", r: "1.5" }
                    circle { cx: "16.5", cy: "17.25", r: "1.5" }
                }
            },
            IconName::ChartBar => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 0 1 3 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 0 1-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 0 1-1.125-1.125V4.125z" }
                }
            },
            IconName::DocumentText => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9z" }
                }
            },
            IconName::Cash => rsx! {
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    class: "icon",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "1.5",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",

                    // Billete de atrás (asoma detrás del de adelante)
                    path { d: "M4.5 4.5h16.5a1.5 1.5 0 0 1 1.5 1.5v7.5a1.5 1.5 0 0 1-1.5 1.5H4.5" }

                    // Billete de adelante (principal)
                    path { d: "M2.25 8.25h16.5a1.5 1.5 0 0 1 1.5 1.5v7.5a1.5 1.5 0 0 1-1.5 1.5H2.25a1.5 1.5 0 0 1-1.5-1.5v-7.5a1.5 1.5 0 0 1 1.5-1.5Z" }

                    // Emblema/moneda central
                    circle { cx: "10.5", cy: "13.5", r: "2.25" }

                    // Detalles de textura en las esquinas del billete
                    path { d: "M4.5 10.5v1.5M16.5 10.5v1.5M4.5 15v1.5M16.5 15v1.5" }
                }
            },
            IconName::Key => rsx! {
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    class: "icon",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "1.5",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path { d: "M15.75 5.25a3 3 0 0 1 3 3m3 0a6 6 0 0 1-7.029 5.912c-.563-.097-1.159.026-1.563.43L10.5 17.25H8.25v2.25H6v2.25H2.25v-2.818c0-.597.237-1.17.659-1.591l6.499-6.499c.404-.404.527-1 .43-1.563A6 6 0 1 1 21.75 8.25Z" }
                }
            },
            IconName::Ban => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    circle { cx: "12", cy: "12", r: "9" }
                    path { d: "M5.75 5.75l12.5 12.5" }
                }
            },
            IconName::Unlink => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M13.19 8.688a4.5 4.5 0 0 1 1.242 7.244l-4.5 4.5a4.5 4.5 0 0 1-6.364-6.364l1.757-1.757m9.86-2.648a4.5 4.5 0 0 0-1.242-7.244l-4.5-4.5a4.5 4.5 0 0 0-6.364 6.364L4.25 8.5" }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_render_does_not_panic() {
        // Solo verificamos que renderizar un icono no falle.
        let _ = IconName::Home.render();
        let _ = IconName::Users.render();
        let _ = IconName::Logout.render();
    }
}
