use dioxus::prelude::*;

use crate::api::ApiClient;
use crate::config::config;
use crate::routes::Route;
use inventory_common::{UserRole, Workshop};

/// Estado global de autenticación de la aplicación.
#[derive(Clone, Copy)]
pub struct AuthState {
    pub token: Signal<Option<String>>,
    pub user_email: Signal<Option<String>>,
    pub user_display_name: Signal<Option<String>>,
    pub user_role: Signal<Option<UserRole>>,
    pub workshop: Signal<Option<Workshop>>,
}

impl AuthState {
    pub fn is_authenticated(&self) -> bool {
        self.token.read().as_ref().is_some()
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
    }

    #[allow(dead_code)]
    pub fn api_client(&self) -> Option<ApiClient> {
        let cfg = config();
        self.token.read().as_ref().and_then(|t| {
            ApiClient::new(None, cfg.server.api_key.clone(), true)
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
    let auth = AuthState {
        token,
        user_email,
        user_display_name,
        user_role,
        workshop,
    };

    use_context_provider(|| auth);

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
