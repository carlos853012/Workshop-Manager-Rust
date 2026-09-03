use dioxus::prelude::*;

/// Nombres de iconos disponibles en la aplicación.
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
pub enum IconName {
    Home,
    Login,
    Logout,
    Plus,
    Edit,
    Trash,
    Search,
    Refresh,
    ChevronLeft,
    ChevronRight,
    User,
    Users,
    Wrench,
    ShoppingCart,
    Package,
    Truck,
    ChartBar,
    DocumentText,
    Cog,
    Moon,
    Sun,
    Check,
    X,
    Exclamation,
    InformationCircle,
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
            IconName::Login => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M15.75 9V5.25A2.25 2.25 0 0 0 13.5 3h-6a2.25 2.25 0 0 0-2.25 2.25v13.5A2.25 2.25 0 0 0 7.5 21h6a2.25 2.25 0 0 0 2.25-2.25V15m3 0 3-3m0 0-3-3m3 3H9" }
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
            IconName::Refresh => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7" }
                }
            },
            IconName::ChevronLeft => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M15.75 19.5 8.25 12l7.5-7.5" }
                }
            },
            IconName::ChevronRight => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M8.25 4.5l7.5 7.5-7.5 7.5" }
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
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M2.25 3h1.386c.51 0 .955.343 1.087.835l.383 1.437M7.5 14.25a3 3 0 0 0-3 3h15.75m-12.75-3h11.218c1.898 0 3.468-1.424 3.688-3.281l.375-3.375a1.125 1.125 0 0 0-1.119-1.244H5.25m8.25 12.75h.008v.008H13.5v-.008zM11.25 15.75h.008v.008h-.008v-.008zm-3 0h.008v.008H8.25v-.008z" }
                }
            },
            IconName::Package => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M20.25 7.5l-.625 10.632a2.25 2.25 0 0 1-2.247 2.118H6.622a2.25 2.25 0 0 1-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z" }
                }
            },
            IconName::Truck => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M8.25 18.75a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h6m-9 0H3.375a1.125 1.125 0 0 1-1.125-1.125V14.25m17.25 4.5a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m3 0h1.5m-15 0H3.375A1.125 1.125 0 0 1 2.25 18.75V9.75c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v1.5m0-1.5h8.25m-8.25 0V5.25A2.25 2.25 0 0 1 5.25 3h10.5a2.25 2.25 0 0 1 2.25 2.25v4.5m-12.75 0h12.75" }
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
            IconName::Cog => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 0 1 0 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.212 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 0 1 0-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" }
                    path { d: "M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0z" }
                }
            },
            IconName::Moon => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M21.752 15.002A9.718 9.718 0 0 1 18 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 0 0 3 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 0 0 9.002-5.998z" }
                }
            },
            IconName::Sun => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M12 3v2.25m6.364.386-1.591 1.591M21 12h-2.25m-.386 6.364-1.591-1.591M12 18.75V21m-4.773-4.227-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0z" }
                }
            },
            IconName::Check => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M4.5 12.75l6 6 9-13.5" }
                }
            },
            IconName::X => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M6 18L18 6M6 6l12 12" }
                }
            },
            IconName::Exclamation => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" }
                }
            },
            IconName::InformationCircle => rsx! {
                svg { xmlns: "http://www.w3.org/2000/svg", class: "icon", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                    path { d: "M11.25 11.25l.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0zM9.75 9.75c0 .414.168.75.375.75s.375-.336.375-.75-.168-.75-.375-.75-.375.336-.375.75z" }
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
