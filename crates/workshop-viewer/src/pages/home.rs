use dioxus::prelude::*;
use dioxus_router::prelude::*;
use rust_decimal::Decimal;
use workshop_common::dto::{RevenueDataPoint, TopProductItem};
use workshop_common::money::format_clp;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::line_chart::LineChart;
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::molecules::date_filter::{DateFilter, DateRange};
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

    let revenue_data = use_signal(Vec::<RevenueDataPoint>::new);
    let revenue_loading = use_signal(|| false);
    let top_products = use_signal(Vec::<TopProductItem>::new);

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
                    Err(ApiError::Unauthorized) => {
                        auth.logout();
                        navigator.push(Route::Login {});
                    }
                    Err(ApiError::Forbidden) => {
                        error_set.set(Some(
                            "No tienes permisos para acceder a esta sección.".to_string(),
                        ));
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

    let load_revenue = move |range: DateRange| {
        let client = auth.api_client();
        let mut revenue_data_set = revenue_data;
        let mut revenue_loading_set = revenue_loading;
        let mut top_products_set = top_products;

        revenue_loading_set.set(true);

        spawn(async move {
            if let Some(client) = client {
                let start = if range.start.is_empty() {
                    None
                } else {
                    Some(range.start.as_str())
                };
                let end = if range.end.is_empty() {
                    None
                } else {
                    Some(range.end.as_str())
                };
                match client.get_revenue(start, end).await {
                    Ok(response) => {
                        revenue_data_set.set(response.data);
                    }
                    Err(_) => {
                        revenue_data_set.set(Vec::new());
                    }
                }
                match client.get_top_products().await {
                    Ok(response) => {
                        top_products_set.set(response.data);
                    }
                    Err(_) => {
                        top_products_set.set(Vec::new());
                    }
                }
            }
            revenue_loading_set.set(false);
        });
    };

    use_effect(move || {
        load_data();
        load_revenue(DateRange::default());
    });

    let on_date_change = move |range: DateRange| {
        load_revenue(range);
    };

    rsx! {
        AppShell { title: "Dashboard".to_string(), active_route: Route::Dashboard {},
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            }
            if *loading.read() {
                div { class: "empty-state", Spinner {} }
            } else {
                div { class: "grid grid-4 mb-lg",
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

                DateFilter { on_change: on_date_change }

                {
                    let chart_data: Vec<(String, f64)> = revenue_data
                        .read()
                        .iter()
                        .map(|d| {
                            let sales = d.sales.to_string().parse::<f64>().unwrap_or(0.0);
                            (d.period.clone(), sales)
                        })
                        .collect();

                    rsx! {
                        div { class: "revenue-grid",
                            Card { title: "Ingresos por período".to_string(),
                                class: if *revenue_loading.read() { "chart-loading".to_string() } else { String::new() },
                                LineChart {
                                    data: chart_data,
                                    width: 700,
                                    height: 350,
                                    class: Some("revenue-chart".to_string()),
                                }
                            }
                            Card { title: "Top Productos".to_string(),
                                div { class: "revenue-ranking",
                                    if top_products.read().is_empty() {
                                        p { class: "text-muted text-sm", "Sin ventas en este período" }
                                    } else {
                                        for item in top_products.read().iter() {
                                            div { class: "revenue-ranking-item",
                                                span { class: "revenue-ranking-label", "{item.product_name}" }
                                                span { class: "revenue-ranking-value", "{item.total_quantity} uds" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
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
    let checked = use_signal(|| false);

    use_effect(move || {
        let mut checked_set = checked;
        let auth = auth;
        let navigator = navigator;
        spawn(async move {
            if auth.is_authenticated() {
                navigator.push(Route::Dashboard {});
                checked_set.set(true);
                return;
            }
            let cfg = crate::config::config();
            let client = match crate::api::ApiClient::new(
                None,
                cfg.server.api_key.clone(),
                cfg.server.tls_accept_invalid_certs,
            ) {
                Ok(c) => c,
                Err(_) => {
                    navigator.push(Route::Setup {});
                    checked_set.set(true);
                    return;
                }
            };
            match client.setup_status().await {
                Ok(has_users) => {
                    if has_users {
                        navigator.push(Route::Login {});
                    } else {
                        navigator.push(Route::Setup {});
                    }
                }
                Err(_) => {
                    navigator.push(Route::Setup {});
                }
            }
            checked_set.set(true);
        });
    });

    if *checked.read() {
        rsx! {}
    } else {
        rsx! {
            div { class: "login-page",
                div { class: "card login-card",
                    div { class: "card-body empty-state",
                        crate::components::atoms::spinner::Spinner {}
                    }
                }
            }
        }
    }
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
