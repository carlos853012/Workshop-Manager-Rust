use dioxus::prelude::*;

use crate::api::ApiClient;

/// Estado global de autenticación de la aplicación.
#[derive(Clone, Copy)]
pub struct AuthState {
    pub token: Signal<Option<String>>,
    pub user_email: Signal<Option<String>>,
}

impl AuthState {
    pub fn is_authenticated(&self) -> bool {
        self.token.read().as_ref().is_some()
    }

    pub fn login(&mut self, token: String, email: String) {
        self.token.set(Some(token));
        self.user_email.set(Some(email));
    }

    pub fn logout(&mut self) {
        self.token.set(None);
        self.user_email.set(None);
    }

    #[allow(dead_code)]
    pub fn api_client(&self) -> Option<ApiClient> {
        self.token.read().as_ref().and_then(|t| {
            ApiClient::new(None, true)
                .ok()
                .map(|client| client.with_token(t.clone()))
        })
    }
}

#[component]
pub fn AuthProvider(children: Element) -> Element {
    let token = use_signal(|| None::<String>);
    let user_email = use_signal(|| None::<String>);
    let auth = AuthState { token, user_email };

    use_context_provider(|| auth);

    rsx! { {children} }
}

pub fn use_auth() -> AuthState {
    use_context::<AuthState>()
}
