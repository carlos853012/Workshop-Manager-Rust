use dioxus::prelude::*;
use dioxus_router::prelude::*;
use workshop_common::money::format_clp;
use rust_decimal::Decimal;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::icons::IconName;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;

#[component]
pub fn Dashboard() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let auth = use_auth();
    let navigator = dioxus_router::prelude::use_navigator();
    let loading = use_signal(|| true);
    let error = use_signal(|| None::<String>);

    let total_products = use_signal(|| 0i64);
    let total_sales = use_signal(|| 0i64);
    let pending_repairs = use_signal(|| 0i64);
    let total_suppliers = use_signal(|| 0i64);
    let total_revenue = use_signal(|| Decimal::ZERO);
    let average_sale = use_signal(|| Decimal::ZERO);
    let total_customers = use_signal(|| 0i64);
    let in_progress_repairs = use_signal(|| 0i64);

    let load_data = move || {
        let client = auth.api_client();
        let mut loading_set = loading;
        let mut error_set = error;
        let mut total_products_set = total_products;
        let mut total_sales_set = total_sales;
        let mut pending_repairs_set = pending_repairs;
        let mut total_suppliers_set = total_suppliers;
        let mut total_revenue_set = total_revenue;
        let mut average_sale_set = average_sale;
        let mut total_customers_set = total_customers;
        let mut in_progress_repairs_set = in_progress_repairs;

        loading_set.set(true);
        error_set.set(None);

        let mut auth = auth;
        spawn(async move {
            if let Some(client) = client {
                match client.get_dashboard().await {
                    Ok(d) => {
                        total_products_set.set(d.total_products);
                        total_sales_set.set(d.total_sales);
                        pending_repairs_set.set(d.pending_repairs);
                        total_revenue_set.set(d.total_revenue);
                        average_sale_set.set(d.average_sale);
                    }
                    Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                        auth.logout();
                        navigator.push(Route::Login {});
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
                    }
                }
                if let Ok(k) = client.get_kpis().await {
                    total_suppliers_set.set(k.total_suppliers);
                    total_customers_set.set(k.total_customers);
                    in_progress_repairs_set.set(k.in_progress_repairs);
                }
            } else {
                error_set.set(Some("No hay cliente API".to_string()));
            }
            loading_set.set(false);
        });
    };

    use_effect(move || {
        load_data();
    });

    rsx! {
        AppShell { title: "Dashboard".to_string(), active_route: Route::Dashboard {},
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            if *loading.read() {
                div { class: "empty-state", Spinner {} }
            } else {
                div { class: "grid grid-4",
                    DashboardCard {
                        title: "Ingresos".to_string(),
                        value: format_clp(*total_revenue.read()),
                        icon: IconName::Cash,
                        subtitle: Some(format!("Promedio: {}", format_clp(*average_sale.read()))),
                    }
                    DashboardCard {
                        title: "Ventas".to_string(),
                        value: total_sales.read().to_string(),
                        icon: IconName::ShoppingCart,

                    }
                    DashboardCard {
                        title: "Productos".to_string(),
                        value: total_products.read().to_string(),
                        icon: IconName::Package,
                    }
                    DashboardCard {
                        title: "Reparaciones Pendientes".to_string(),
                        value: pending_repairs.read().to_string(),
                        icon: IconName::Wrench,
                        subtitle: Some(format!("En Progreso: {}", in_progress_repairs.read())),
                    }
                    DashboardCard {
                        title: "Clientes".to_string(),
                        value: total_customers.read().to_string(),
                        icon: IconName::User,
                    }


                }
            }
        }
    }
}

#[component]
pub fn Root() -> Element {
    let auth = use_auth();
    let navigator = use_navigator();

    if auth.is_authenticated() {
        navigator.push(Route::Dashboard {});
    } else {
        navigator.push(Route::Setup {});
    }

    rsx! {}
}

#[component]
fn DashboardCard(
    title: String,
    value: String,
    icon: IconName,
    subtitle: Option<String>,
) -> Element {
    rsx! {
        Card {
            div { class: "flex items-center gap-md",
                span { class: "text-2xl", {icon.render()} }
                div {
                    p { class: "text-muted text-sm", "{title}" }
                    p { class: "text-xl font-semibold", "{value}" }
                    if let Some(sub) = subtitle {
                        p { class: "text-xs text-muted", "{sub}" }
                    }
                }
            }
        }
    }
}
