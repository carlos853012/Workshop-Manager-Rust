use dioxus::prelude::*;

use crate::api::{ApiClient, LicenseInfo};
use crate::config::config;
use crate::routes::Route;
use workshop_common::features::{Feature, LicenseTier};
use workshop_common::{UserRole, Workshop};

/// Estado global de autenticación de la aplicación.
#[derive(Clone, Copy)]
pub struct AuthState {
    pub token: Signal<Option<String>>,
    pub user_email: Signal<Option<String>>,
    pub user_display_name: Signal<Option<String>>,
    pub user_role: Signal<Option<UserRole>>,
    pub workshop: Signal<Option<Workshop>>,
    pub license_info: Signal<Option<LicenseInfo>>,
}

impl AuthState {
    pub fn is_authenticated(&self) -> bool {
        self.token.read().as_ref().is_some()
    }

    pub fn is_trial(&self) -> bool {
        // Desconocido != trial: solo mostramos el badge cuando el servidor
        // confirma que el tier es Trial.
        self.license_info
            .read()
            .as_ref()
            .map(|l| l.is_trial)
            .unwrap_or(false)
    }

    /// Tier efectivo de la licencia. Sin información → `Trial` (solo base).
    pub fn license_tier(&self) -> LicenseTier {
        self.license_info
            .read()
            .as_ref()
            .and_then(|l| l.tier.parse::<LicenseTier>().ok())
            .unwrap_or(LicenseTier::Trial)
    }

    /// ¿La licencia actual incluye `feature`?
    ///
    /// Mientras no se cargó la licencia se asume `Trial`: los módulos DLC
    /// permanecen ocultos hasta confirmar el tier.
    pub fn has_feature(&self, feature: Feature) -> bool {
        self.license_tier().features().contains(&feature)
    }

    pub fn login(
        &mut self,
        token: String,
        email: String,
        display_name: Option<String>,
        role: UserRole,
        workshop: Option<Workshop>,
    ) {
        self.token.set(Some(token));
        self.user_email.set(Some(email));
        self.user_display_name.set(display_name);
        self.user_role.set(Some(role));
        self.workshop.set(workshop);
    }

    pub fn logout(&mut self) {
        self.token.set(None);
        self.user_email.set(None);
        self.user_display_name.set(None);
        self.user_role.set(None);
        self.workshop.set(None);
        self.license_info.set(None);
    }

    pub fn api_client(&self) -> Option<ApiClient> {
        let cfg = config();
        self.token.read().as_ref().and_then(|t| {
            ApiClient::new(
                None,
                cfg.server.api_key.clone(),
                cfg.server.tls_accept_invalid_certs,
            )
            .ok()
            .map(|client| client.with_token(t.clone()))
        })
    }
}

#[component]
pub fn AuthProvider(children: Element) -> Element {
    let token = use_signal(|| None::<String>);
    let user_email = use_signal(|| None::<String>);
    let user_display_name = use_signal(|| None::<String>);
    let user_role = use_signal(|| None::<UserRole>);
    let workshop = use_signal(|| None::<Workshop>);
    let license_info = use_signal(|| None::<LicenseInfo>);
    let mut auth = AuthState {
        token,
        user_email,
        user_display_name,
        user_role,
        workshop,
        license_info,
    };

    use_context_provider(|| auth);

    // Refresca la info de licencia al iniciar sesión: el badge TRIAL y el
    // gating de módulos deben reflejar el tier real del servidor.
    use_effect(move || {
        if auth.token.read().is_some() {
            spawn(async move {
                if let Some(client) = auth.api_client() {
                    if let Ok(info) = client.license_status().await {
                        auth.license_info.set(Some(info));
                    }
                }
            });
        }
    });

    rsx! { {children} }
}

pub fn use_auth() -> AuthState {
    use_context::<AuthState>()
}

#[derive(Clone, PartialEq)]
pub struct OpenTab {
    pub title: String,
    pub route: Route,
}

#[derive(Clone, Copy)]
pub struct TabsState {
    pub tabs: Signal<Vec<OpenTab>>,
}

#[component]
pub fn TabsProvider(children: Element) -> Element {
    let tabs = use_signal(|| {
        vec![OpenTab {
            title: "Dashboard".to_string(),
            route: Route::Dashboard {},
        }]
    });

    use_context_provider(|| TabsState { tabs });

    rsx! { {children} }
}

pub fn use_tabs() -> TabsState {
    use_context::<TabsState>()
}

#[derive(Clone, Copy)]
pub struct SidebarState {
    pub collapsed: Signal<bool>,
    pub hovered: Signal<bool>,
}

impl SidebarState {
    pub fn toggle(&mut self) {
        let current = *self.collapsed.read();
        self.collapsed.set(!current);
    }
}

#[component]
pub fn SidebarProvider(children: Element) -> Element {
    let collapsed = use_signal(|| false);
    let hovered = use_signal(|| false);
    use_context_provider(|| SidebarState { collapsed, hovered });
    rsx! { {children} }
}

pub fn use_sidebar() -> SidebarState {
    use_context::<SidebarState>()
}
