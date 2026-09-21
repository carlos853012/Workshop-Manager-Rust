use serde::{Deserialize, Serialize};
use workshop_common::dto::{
    AddRepairPartRequest, ApiResponse, ClientHistoryResponse, ClientReport, ClientSearchResult,
    CreateProductRequest, CreateRepairRequest, CreateSaleRequest, CreateSupplierRequest,
    CreateUserRequest, DashboardResponse, DeviceKeySummary, KpisResponse, LoginRequest,
    LoginResponse, PaginatedResponse, PosProductResponse, RegisterRequest, RepairDetail,
    RepairPartResponse, RevenueResponse, SaleDetailResponse, TopProductResponse,
    UpdateRepairRequest, UpdateUserRequest,
};
use workshop_common::{AuditLog, Product, Repair, Sale, Supplier, User};

const DEFAULT_TIMEOUT_SECONDS: u64 = 60;

/// Error posible al llamar a la API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiError {
    Network(String),
    Unauthorized,
    Forbidden,
    NotFound(String),
    Validation(String),
    Server(String),
    Unknown(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(_) => write!(f, "{}", self.user_message()),
            ApiError::Unauthorized => write!(f, "La sesión ha expirado."),
            ApiError::Forbidden => write!(f, "No tienes permisos para realizar esta acción."),
            ApiError::NotFound(_) => write!(f, "El recurso solicitado no fue encontrado."),
            ApiError::Validation(_) => write!(f, "Revisa los datos ingresados."),
            ApiError::Server(_) => write!(f, "El servidor no pudo completar la operación."),
            ApiError::Unknown(_) => write!(f, "Ocurrió un error inesperado."),
        }
    }
}

impl ApiError {
    pub fn user_message(&self) -> &'static str {
        match self {
            ApiError::Network(_) => {
                "No se pudo conectar con el servidor. Verifica que esté encendido."
            }
            ApiError::Unauthorized => "La sesión ha expirado.",
            ApiError::Forbidden => "No tienes permisos para realizar esta acción.",
            ApiError::NotFound(_) => "El recurso solicitado no fue encontrado.",
            ApiError::Validation(_) => "Revisa los datos ingresados.",
            ApiError::Server(_) => "El servidor no pudo completar la operación.",
            ApiError::Unknown(_) => "Ocurrió un error inesperado.",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub is_trial: bool,
    pub tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub iva_rate: f64,
}

/// Cliente HTTP centralizado para comunicarse con el servidor.
#[derive(Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    device_key: String,
    token: Option<String>,
}

#[allow(dead_code)]
impl ApiClient {
    /// Crea un cliente nuevo.
    ///
    /// - `base_url`: URL base del servidor. Si es `None`, se usa el valor por defecto
    ///   de la configuración (`https://127.0.0.1:8443`).
    /// - `api_key`: clave compartida con el servidor para autenticar al viewer desktop.
    /// - `accept_invalid_certs`: si es `true`, permite conectar al certificado autofirmado
    ///   en desarrollo local.
    pub fn new(
        base_url: Option<String>,
        api_key: String,
        accept_invalid_certs: bool,
    ) -> Result<Self, ApiError> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(accept_invalid_certs)
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECONDS))
            .build()
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to build HTTP client");
                ApiError::Network(format!("Failed to build HTTP client: {}", e))
            })?;

        let config = crate::config::config();
        let resolved_url =
            crate::config::resolve_base_url(&base_url.unwrap_or(config.server.base_url));
        Ok(Self {
            client,
            base_url: resolved_url,
            api_key,
            device_key: config.server.device_key,
            token: None,
        })
    }

    /// Establece el token JWT usado en las siguientes peticiones.
    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    fn auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {}", t))
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// GET /api/auth/setup-status (público, sin JWT)
    pub async fn setup_status(&self) -> Result<bool, ApiError> {
        #[derive(serde::Deserialize)]
        struct SetupStatus {
            has_users: bool,
        }
        let url = self.url("/api/auth/setup-status");
        let resp = self
            .client
            .get(&url)
            .header("X-WorkshopManager-Key", &self.api_key)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(url = %url, error = %e, "Network error checking setup status");
                ApiError::Network(e.to_string())
            })?;
        let status = resp.status();
        if !status.is_success() {
            return Err(ApiError::Unknown(format!("HTTP {status}")));
        }
        let body = resp
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        let parsed: ApiResponse<SetupStatus> = serde_json::from_str(&body)
            .map_err(|e| ApiError::Unknown(format!("JSON error: {e}")))?;
        Ok(parsed.data.map(|d| d.has_users).unwrap_or(false))
    }

    /// POST /api/auth/login
    pub async fn login(&self, email: &str, password: &str) -> Result<LoginResponse, ApiError> {
        let request = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        };
        self.post("/api/auth/login", &request).await
    }

    /// POST /api/auth/register
    /// Solo funciona si no existe ningún usuario (primer admin).
    pub async fn register(
        &self,
        workshop_name: &str,
        workshop_address: &str,
        workshop_city: &str,
        admin_name: &str,
        email: &str,
        password: &str,
    ) -> Result<LoginResponse, ApiError> {
        let request = RegisterRequest {
            workshop_name: workshop_name.to_string(),
            workshop_address: workshop_address.to_string(),
            workshop_city: workshop_city.to_string(),
            admin_name: admin_name.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        };
        self.post("/api/auth/register", &request).await
    }

    /// GET /api/auth/status
    pub async fn status(&self) -> Result<User, ApiError> {
        self.get("/api/auth/status").await
    }

    pub async fn license_status(&self) -> Result<LicenseInfo, ApiError> {
        self.get("/api/auth/license").await
    }

<<<<<<< HEAD
=======
    /// POST /api/auth/activate
    pub async fn activate_license(&self, license_key: &str) -> Result<LicenseInfo, ApiError> {
        #[derive(serde::Serialize)]
        struct ActivateReq<'a> {
            license_key: &'a str,
        }
        #[derive(serde::Deserialize)]
        struct ActivateResp {
            is_trial: bool,
            tier: String,
        }
        let resp: ActivateResp = self
            .post("/api/auth/activate", &ActivateReq { license_key })
            .await?;
        Ok(LicenseInfo {
            is_trial: resp.is_trial,
            tier: resp.tier,
        })
    }

>>>>>>> 378ab33 (feat: license architecture, security fixes, IVA dynamic, API docs)
    /// GET /api/config
    pub async fn get_config(&self) -> Result<ServerConfig, ApiError> {
        self.get("/api/config").await
    }

    /// GET /api/products
    pub async fn list_products(
        &self,
        page: i32,
        per_page: i32,
    ) -> Result<PaginatedResponse<Product>, ApiError> {
        self.get_with_query(
            "/api/products",
            &[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ],
        )
        .await
    }

    /// GET /api/sales
    pub async fn list_sales(
        &self,
        page: i32,
        per_page: i32,
    ) -> Result<PaginatedResponse<Sale>, ApiError> {
        self.get_with_query(
            "/api/sales",
            &[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ],
        )
        .await
    }

    /// GET /api/sales/:id
    pub async fn get_sale(&self, id: uuid::Uuid) -> Result<SaleDetailResponse, ApiError> {
        self.get(&format!("/api/sales/{}", id)).await
    }

    /// GET /api/repairs
    pub async fn list_repairs(
        &self,
        page: i32,
        per_page: i32,
    ) -> Result<PaginatedResponse<Repair>, ApiError> {
        self.get_with_query(
            "/api/repairs",
            &[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ],
        )
        .await
    }

    /// GET /api/suppliers
    pub async fn list_suppliers(
        &self,
        page: i32,
        per_page: i32,
    ) -> Result<PaginatedResponse<Supplier>, ApiError> {
        self.get_with_query(
            "/api/suppliers",
            &[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ],
        )
        .await
    }

    /// POST /api/products
    pub async fn create_product(
        &self,
        request: &CreateProductRequest,
    ) -> Result<Product, ApiError> {
        self.post("/api/products", request).await
    }

    /// PUT /api/products/:id
    pub async fn update_product(
        &self,
        id: uuid::Uuid,
        request: &CreateProductRequest,
    ) -> Result<Product, ApiError> {
        self.put(&format!("/api/products/{}", id), request).await
    }

    /// DELETE /api/products/:id
    pub async fn delete_product(&self, id: uuid::Uuid) -> Result<(), ApiError> {
        self.delete(&format!("/api/products/{}", id)).await
    }

    /// POST /api/sales
    pub async fn create_sale(&self, request: &CreateSaleRequest) -> Result<Sale, ApiError> {
        self.post("/api/sales", request).await
    }

    /// POST /api/sales/:id/cancel
    pub async fn cancel_sale(&self, id: uuid::Uuid) -> Result<Sale, ApiError> {
        self.post(&format!("/api/sales/{}/cancel", id), &()).await
    }

    /// POST /api/repairs
    pub async fn create_repair(&self, request: &CreateRepairRequest) -> Result<Repair, ApiError> {
        self.post("/api/repairs", request).await
    }

    /// GET /api/repairs/:id
    pub async fn get_repair(&self, id: uuid::Uuid) -> Result<RepairDetail, ApiError> {
        self.get(&format!("/api/repairs/{}", id)).await
    }

    /// PUT /api/repairs/:id
    pub async fn update_repair(
        &self,
        id: uuid::Uuid,
        request: &UpdateRepairRequest,
    ) -> Result<RepairDetail, ApiError> {
        self.put(&format!("/api/repairs/{}", id), request).await
    }

    /// GET /api/repairs/:id/parts
    pub async fn list_repair_parts(
        &self,
        repair_id: uuid::Uuid,
    ) -> Result<Vec<RepairPartResponse>, ApiError> {
        self.get(&format!("/api/repairs/{}/parts", repair_id)).await
    }

    /// POST /api/repairs/:id/parts
    pub async fn add_repair_part(
        &self,
        repair_id: uuid::Uuid,
        request: &AddRepairPartRequest,
    ) -> Result<RepairPartResponse, ApiError> {
        self.post(&format!("/api/repairs/{}/parts", repair_id), request)
            .await
    }

    /// DELETE /api/repairs/:id/parts/:part_id
    pub async fn remove_repair_part(
        &self,
        repair_id: uuid::Uuid,
        part_id: uuid::Uuid,
    ) -> Result<(), ApiError> {
        self.delete(&format!("/api/repairs/{}/parts/{}", repair_id, part_id))
            .await
    }

    /// POST /api/suppliers
    pub async fn create_supplier(
        &self,
        request: &CreateSupplierRequest,
    ) -> Result<Supplier, ApiError> {
        self.post("/api/suppliers", request).await
    }

    /// GET /api/products/lookup?barcode=XXX
    pub async fn lookup_product_by_barcode(
        &self,
        barcode: &str,
    ) -> Result<PosProductResponse, ApiError> {
        self.get_with_query("/api/products/lookup", &[("barcode", barcode.to_string())])
            .await
    }

    // ==================== ANALYTICS ====================

    /// GET /api/analytics/dashboard
    pub async fn get_dashboard(&self) -> Result<DashboardResponse, ApiError> {
        self.get("/api/analytics/dashboard").await
    }

    /// GET /api/analytics/kpis
    pub async fn get_kpis(&self) -> Result<KpisResponse, ApiError> {
        self.get("/api/analytics/kpis").await
    }

    /// GET /api/analytics/revenue?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD
    pub async fn get_revenue(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<RevenueResponse, ApiError> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(s) = start_date {
            params.push(("start_date", s.to_string()));
        }
        if let Some(e) = end_date {
            params.push(("end_date", e.to_string()));
        }
        self.get_with_query("/api/analytics/revenue", &params).await
    }

    /// GET /api/analytics/top-products
    pub async fn get_top_products(&self) -> Result<TopProductResponse, ApiError> {
        self.get("/api/analytics/top-products").await
    }

    // ==================== REPORTS ====================

    /// GET /api/reports/clients
    pub async fn list_clients(&self) -> Result<Vec<ClientReport>, ApiError> {
        self.get("/api/reports/clients").await
    }

    /// GET /api/reports/client-history?email=XXX
    pub async fn get_client_history(&self, email: &str) -> Result<ClientHistoryResponse, ApiError> {
        self.get_with_query(
            "/api/reports/client-history",
            &[("email", email.to_string())],
        )
        .await
    }

    /// GET /api/reports/client-search?query=XXX
    pub async fn client_search(&self, query: &str) -> Result<Vec<ClientSearchResult>, ApiError> {
        self.get_with_query(
            "/api/reports/client-search",
            &[("query", query.to_string())],
        )
        .await
    }

    /// GET /api/reports/client-certificate.pdf?email=XXX&plate=YYY
    /// Returns raw PDF bytes.
    pub async fn download_certificate(
        &self,
        email: Option<&str>,
        name: Option<&str>,
        plate: Option<&str>,
    ) -> Result<Vec<u8>, ApiError> {
        let mut query = Vec::new();
        if let Some(e) = email {
            query.push(("email".to_string(), e.to_string()));
        }
        if let Some(n) = name {
            query.push(("name".to_string(), n.to_string()));
        }
        if let Some(p) = plate {
            query.push(("plate".to_string(), p.to_string()));
        }
        let url = self.url("/api/reports/client-certificate.pdf");
        let mut request = self
            .client
            .get(&url)
            .query(&query)
            .header("X-WorkshopManager-Key", &self.api_key);
        if !self.device_key.is_empty() {
            request = request.header("X-WorkshopManager-Device-Key", &self.device_key);
        }
        if let Some(header) = self.auth_header() {
            request = request.header("Authorization", header);
        }
        let response = request.send().await.map_err(|e| {
            tracing::error!(url = %url, error = %e, "Network error downloading certificate");
            ApiError::Network(e.to_string())
        })?;
        let status = response.status();
        if status.is_success() {
            response
                .bytes()
                .await
                .map(|b| b.to_vec())
                .map_err(|e| ApiError::Network(e.to_string()))
        } else {
            match status.as_u16() {
                401 => Err(ApiError::Unauthorized),
                403 => Err(ApiError::Forbidden),
                404 => Err(ApiError::NotFound("Certificado no encontrado".into())),
                500..=599 => {
                    let body = response.text().await.unwrap_or_default();
                    Err(ApiError::Server(body))
                }
                _ => Err(ApiError::Unknown(format!("HTTP {}", status))),
            }
        }
    }

    // ==================== USERS ====================

    /// GET /api/users
    pub async fn list_users(
        &self,
        page: i32,
        per_page: i32,
    ) -> Result<PaginatedResponse<User>, ApiError> {
        self.get_with_query(
            "/api/users",
            &[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ],
        )
        .await
    }

    /// POST /api/users
    pub async fn create_user(&self, request: &CreateUserRequest) -> Result<User, ApiError> {
        self.post("/api/users", request).await
    }

    /// PUT /api/users/:id
    pub async fn update_user(
        &self,
        id: uuid::Uuid,
        request: &UpdateUserRequest,
    ) -> Result<User, ApiError> {
        self.put(&format!("/api/users/{}", id), request).await
    }

    /// DELETE /api/users/:id
    pub async fn delete_user(&self, id: uuid::Uuid) -> Result<(), ApiError> {
        self.delete(&format!("/api/users/{}", id)).await
    }

    // ==================== DEVICE KEYS ====================

    /// POST /api/device-keys
    pub async fn generate_device_key(&self) -> Result<String, ApiError> {
        #[derive(serde::Deserialize)]
        struct GeneratedDeviceKey {
            key: String,
        }
        let resp: GeneratedDeviceKey = self
            .post("/api/device-keys", &serde_json::json!({}))
            .await?;
        Ok(resp.key)
    }

    /// GET /api/device-keys
    pub async fn list_device_keys(&self) -> Result<Vec<DeviceKeySummary>, ApiError> {
        self.get("/api/device-keys").await
    }

    /// POST /api/device-keys/:id/revoke
    pub async fn revoke_device_key(&self, id: uuid::Uuid) -> Result<(), ApiError> {
        self.post(
            &format!("/api/device-keys/{}/revoke", id),
            &serde_json::json!({}),
        )
        .await
    }

    /// POST /api/device-keys/:id/unbind
    pub async fn unbind_device_key(&self, id: uuid::Uuid) -> Result<(), ApiError> {
        self.post(
            &format!("/api/device-keys/{}/unbind", id),
            &serde_json::json!({}),
        )
        .await
    }

    /// GET /api/audit
    pub async fn list_audit_logs(
        &self,
        page: i32,
        per_page: i32,
    ) -> Result<PaginatedResponse<AuditLog>, ApiError> {
        self.get_with_query(
            "/api/audit",
            &[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ],
        )
        .await
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let mut request = self
            .client
            .get(self.url(path))
            .header("X-WorkshopManager-Key", &self.api_key);
        if !self.device_key.is_empty() {
            request = request.header("X-WorkshopManager-Device-Key", &self.device_key);
        }
        if let Some(header) = self.auth_header() {
            request = request.header("Authorization", header);
        }
        self.handle_response(request.send().await).await
    }

    async fn get_with_query<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, ApiError> {
        let mut request = self
            .client
            .get(self.url(path))
            .query(query)
            .header("X-WorkshopManager-Key", &self.api_key);
        if !self.device_key.is_empty() {
            request = request.header("X-WorkshopManager-Device-Key", &self.device_key);
        }
        if let Some(header) = self.auth_header() {
            request = request.header("Authorization", header);
        }
        self.handle_response(request.send().await).await
    }

    async fn post<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let mut request = self
            .client
            .post(self.url(path))
            .json(body)
            .header("X-WorkshopManager-Key", &self.api_key);
        if !self.device_key.is_empty() {
            request = request.header("X-WorkshopManager-Device-Key", &self.device_key);
        }
        if let Some(header) = self.auth_header() {
            request = request.header("Authorization", header);
        }
        self.handle_response(request.send().await).await
    }

    async fn put<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let mut request = self
            .client
            .put(self.url(path))
            .json(body)
            .header("X-WorkshopManager-Key", &self.api_key);
        if !self.device_key.is_empty() {
            request = request.header("X-WorkshopManager-Device-Key", &self.device_key);
        }
        if let Some(header) = self.auth_header() {
            request = request.header("Authorization", header);
        }
        self.handle_response(request.send().await).await
    }

    async fn delete<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let mut request = self
            .client
            .delete(self.url(path))
            .header("X-WorkshopManager-Key", &self.api_key);
        if !self.device_key.is_empty() {
            request = request.header("X-WorkshopManager-Device-Key", &self.device_key);
        }
        if let Some(header) = self.auth_header() {
            request = request.header("Authorization", header);
        }
        self.handle_response(request.send().await).await
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<T, ApiError> {
        let response = match response {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(
                    url = %self.base_url,
                    error = %e,
                    "Network error in API request"
                );
                return Err(ApiError::Network(e.to_string()));
            }
        };
        let status = response.status();
        let url = response.url().clone();
        let body_text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(format!("Failed to read response body: {}", e)))?;

        if status.is_success() {
            let parsed: ApiResponse<T> = serde_json::from_str(&body_text).map_err(|e| {
                tracing::error!(
                    url = %url,
                    status = %status,
                    error = %e,
                    body = %body_text,
                    "JSON parse error in API response"
                );
                ApiError::Unknown(format!("JSON parse error: {}", e))
            })?;
            match parsed.data {
                Some(data) => Ok(data),
                None => serde_json::from_str("null")
                    .map_err(|e| ApiError::Unknown(format!("Empty response data: {}", e))),
            }
        } else {
            tracing::warn!(
                url = %url,
                status = %status,
                body = %body_text,
                "API request returned error status"
            );
            match status.as_u16() {
                401 => Err(ApiError::Unauthorized),
                403 => Err(ApiError::Forbidden),
                404 => Err(ApiError::NotFound(body_text)),
                422 => Err(ApiError::Validation(body_text)),
                500..=599 => Err(ApiError::Server(body_text)),
                _ => Err(ApiError::Unknown(body_text)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_new() {
        let client = ApiClient::new(None, "test-key".to_string(), true);
        assert!(client.is_ok());
    }

    #[test]
    fn test_api_client_default_url() {
        let client = ApiClient::new(None, "test-key".to_string(), true).unwrap();
        assert_eq!(
            client.base_url,
            crate::config::resolve_base_url(&crate::config::config().server.base_url)
        );
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::Unauthorized;
        assert_eq!(err.to_string(), "La sesión ha expirado.");
    }
}
