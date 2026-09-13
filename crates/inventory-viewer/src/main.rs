mod api;
mod app_state;
mod components;
mod config;
mod i18n;
mod icons;
mod pages;
mod routes;
mod theme;

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use dioxus_router::prelude::*;

use crate::app_state::{AuthProvider, TabsProvider};
use crate::routes::Route;
use crate::theme::ThemeProvider;

fn main() {
    let icon = {
        let pixels = inventory_common::icon_data::generate_wrench_icon(32);
        dioxus_desktop::tao::window::Icon::from_rgba(pixels, 32, 32)
            .expect("Failed to create window icon")
    };

    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title("WorkshopManager")
                        .with_inner_size(dioxus_desktop::tao::dpi::LogicalSize::new(1500.0, 900.0))
                        .with_min_inner_size(dioxus_desktop::tao::dpi::LogicalSize::new(
                            1200.0, 50.0,
                        ))
                        .with_position(dioxus_desktop::tao::dpi::LogicalPosition::new(210.0, 1.0)),
                )
                .with_icon(icon),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        style { {include_str!("../index.css")} }
        ThemeProvider {
            AuthProvider {
                TabsProvider {
                    Router::<Route> {}
                }
            }
        }
    }
}
