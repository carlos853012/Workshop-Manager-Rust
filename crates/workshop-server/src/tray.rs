use crate::device_key;
use sqlx::PgPool;
use std::cell::Cell;
use tokio::sync::oneshot::Sender;

use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::windows::EventLoopBuilderExtWindows;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
use winreg::RegKey;

#[allow(dead_code)]
const AUTOSTART_VALUE: &str = "WorkshopManagerServer";
#[allow(dead_code)]
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

#[allow(dead_code)]
enum TrayEvent {
    Menu(MenuEvent),
}

#[allow(dead_code)]
fn copy_api_key_action(config: &super::state::ServerConfig) {
    match clipboard_win::set_clipboard_string(&config.api_key) {
        Ok(()) => tracing::info!("API Key copiada al portapapeles"),
        Err(error) => tracing::error!(%error, "No se pudo copiar la API Key"),
    }
}

#[allow(dead_code)]
fn generate_key_action(pool: &PgPool) {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!(%error, "No se pudo iniciar runtime para generar Device Key");
            return;
        }
    };

    match runtime.block_on(device_key::create(pool)) {
        Ok(key) => match clipboard_win::set_clipboard_string(&key) {
            Ok(()) => tracing::info!("Device Key generada y copiada al portapapeles"),
            Err(error) => tracing::error!(%error, "Device Key generada, pero no se pudo copiar"),
        },
        Err(error) => tracing::error!(%error, "No se pudo generar Device Key"),
    }
}

#[allow(dead_code)]
fn is_autostart_enabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey_with_flags(RUN_KEY, KEY_READ)
        .and_then(|key| key.get_value::<String, _>(AUTOSTART_VALUE))
        .is_ok()
}

#[allow(dead_code)]
fn set_autostart(enabled: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu
        .open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE)
        .map_err(|error| format!("No se pudo abrir autostart: {error}"))?;

    if enabled {
        let executable = std::env::current_exe()
            .map_err(|error| format!("No se pudo localizar el servidor: {error}"))?;
        run.set_value(AUTOSTART_VALUE, &executable.to_string_lossy().to_string())
            .map_err(|error| format!("No se pudo activar autostart: {error}"))?;
    } else {
        run.delete_value(AUTOSTART_VALUE)
            .map_err(|error| format!("No se pudo desactivar autostart: {error}"))?;
    }

    Ok(())
}

#[allow(dead_code)]
fn tray_icon(config: &super::state::ServerConfig) -> Result<Icon, String> {
    let pixels =
        workshop_common::icon_data::generate_wrench_icon(32, config.icon_bg, config.icon_fg);
    Icon::from_rgba(pixels, 32, 32).map_err(|error| format!("No se pudo crear el icono: {error}"))
}

#[allow(dead_code)]
pub fn run(
    shutdown_tx: Sender<()>,
    pool: PgPool,
    config: super::state::ServerConfig,
    license_info: Option<String>,
) {
    let mut event_loop_builder = EventLoopBuilder::<TrayEvent>::with_user_event();
    event_loop_builder.with_any_thread(true);
    let event_loop = event_loop_builder.build();

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(TrayEvent::Menu(event));
    }));

    let menu = Menu::new();

    let license_text = license_info.unwrap_or_else(|| "Trial (7 días)".to_string());
    let status_item = MenuItem::new(format!("Licencia: {license_text}"), false, None);
    let copy_api_key_item = MenuItem::new("Copiar API Key", true, None);
    let generate_key_item = MenuItem::new("Copiar Device Key", true, None);
    let autostart_item =
        CheckMenuItem::new("Ejecutar al inicio", true, is_autostart_enabled(), None);
    let exit_item = MenuItem::new("Salir del servidor", true, None);

    if let Err(error) = menu.append_items(&[
        &status_item,
        &PredefinedMenuItem::separator(),
        &copy_api_key_item,
        &generate_key_item,
        &PredefinedMenuItem::separator(),
        &autostart_item,
        &PredefinedMenuItem::separator(),
        &exit_item,
    ]) {
        tracing::error!(error = %error, "No se pudo crear el menú del tray");
        let _ = shutdown_tx.send(());
        return;
    }

    let icon = match tray_icon(&config) {
        Ok(icon) => icon,
        Err(error) => {
            tracing::error!(%error, "No se pudo crear el icono del tray");
            let _ = shutdown_tx.send(());
            return;
        }
    };

    let tray = match TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("WorkshopManager Server")
        .with_icon(icon)
        .build()
    {
        Ok(tray) => tray,
        Err(error) => {
            tracing::error!(%error, "No se pudo crear el tray icon");
            let _ = shutdown_tx.send(());
            return;
        }
    };

    let autostart_enabled = Cell::new(is_autostart_enabled());
    let mut shutdown_tx = Some(shutdown_tx);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::UserEvent(TrayEvent::Menu(event)) = event {
            if event.id == copy_api_key_item.id() {
                copy_api_key_action(&config);
            } else if event.id == generate_key_item.id() {
                generate_key_action(&pool);
            } else if event.id == autostart_item.id() {
                let enabled = !autostart_enabled.get();
                if let Err(error) = set_autostart(enabled) {
                    tracing::error!(%error, "No se pudo cambiar autostart");
                } else {
                    autostart_enabled.set(enabled);
                    autostart_item.set_checked(enabled);
                }
            } else if event.id == exit_item.id() {
                if let Some(sender) = shutdown_tx.take() {
                    let _ = sender.send(());
                }
                *control_flow = ControlFlow::Exit;
            }
        }

        _ = &tray;
    });
}
