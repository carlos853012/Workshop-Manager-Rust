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
use crate::theme::{DesignTokens, ThemeProvider};

/// Parsea un hex color "#RRGGBB" a [u8; 3].
fn parse_hex_to_rgb(hex: &str) -> [u8; 3] {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            [r, g, b]
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(0);
            [r * 17, g * 17, b * 17]
        }
        _ => [0xF5, 0x9E, 0x0B],
    }
}

fn main() {
    let icon = {
        let tokens = DesignTokens::light();
        let bg = parse_hex_to_rgb(&tokens.colors.icon_bg);
        let fg = parse_hex_to_rgb(&tokens.colors.icon_fg);
        let pixels = workshop_common::icon_data::generate_wrench_icon(32, bg, fg);
        dioxus_desktop::tao::window::Icon::from_rgba(pixels, 32, 32)
            .expect("Failed to create window icon")
    };

    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
        .join("WorkshopManager")
        .join("viewer_data");

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
                .with_icon(icon)
                .with_data_directory(data_dir),
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
