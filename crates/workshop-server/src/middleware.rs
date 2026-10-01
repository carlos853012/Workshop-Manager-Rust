use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use workshop_common::features::{Feature, License, LicenseTier};
use workshop_common::UserRole;

use crate::auth::{self, Claims};
use crate::state::AppState;

/// Usuario autenticado inyectado en la request por el middleware.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: uuid::Uuid,
    #[allow(dead_code)]
    pub email: String,
    pub role: UserRole,
    pub workshop_id: uuid::Uuid,
}

impl AuthenticatedUser {
    pub fn from_claims(claims: Claims) -> Result<Self, StatusCode> {
        let id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;
        let role = match claims.role.as_str() {
            "admin" => UserRole::Admin,
            "mechanic" => UserRole::Mechanic,
            "seller" => UserRole::Seller,
            _ => return Err(StatusCode::UNAUTHORIZED),
        };
        let workshop_id =
            uuid::Uuid::parse_str(&claims.workshop_id).map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(Self {
            id,
            email: claims.email,
            role,
            workshop_id,
        })
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin)
    }
}

/// Middleware que exige la API key compartida con el viewer desktop.
pub async fn api_key_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let key = request
        .headers()
        .get("X-WorkshopManager-Key")
        .and_then(|v| v.to_str().ok());

    match key {
        Some(k) if k == state.config.api_key => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Middleware que exige un JWT válido en el header `Authorization: Bearer <token>`.
pub async fn authenticate_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = extract_bearer_token(&request)?;
    let claims = auth::validate_token(token, &state.secrets.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = AuthenticatedUser::from_claims(claims)?;
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

/// Middleware que exige que el usuario autenticado sea admin.
pub async fn require_admin_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

/// Helper puro: ¿la licencia incluye `feature`?
///
/// Sin licencia se asume el tier `Trial` (solo features base), de modo que
/// una instalación nueva nunca obtiene módulos DLC.
pub fn license_allows(license: Option<&License>, feature: &Feature) -> bool {
    match license {
        Some(lic) => lic.has_feature(feature),
        None => LicenseTier::Trial.features().contains(feature),
    }
}

/// Middleware que exige que la licencia activa incluya `feature`.
///
/// Se construye con estado `(AppState, Feature)` vía `from_fn_with_state`.
/// Devuelve `403 FORBIDDEN` si el tier no incluye la feature.
pub async fn require_feature(
    State((state, feature)): State<(AppState, Feature)>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let allowed = {
        let license = state.license.read().await;
        license_allows(license.as_ref(), &feature)
    };

    if allowed {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

fn extract_bearer_token(request: &Request) -> Result<&str, StatusCode> {
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let auth_value = auth_header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;

    auth_value
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)
}

#[cfg(test)]
mod tests {
    use super::*;
    use workshop_common::UserRole;

    #[test]
    fn test_authenticated_user_from_claims_admin() {
        let claims = Claims {
            sub: uuid::Uuid::new_v4().to_string(),
            email: "admin@example.com".to_string(),
            role: "admin".to_string(),
            workshop_id: uuid::Uuid::new_v4().to_string(),
            exp: 1234567890,
        };

        let user = AuthenticatedUser::from_claims(claims).unwrap();
        assert!(user.is_admin());
        assert_eq!(user.email, "admin@example.com");
    }

    #[test]
    fn test_authenticated_user_from_claims_seller() {
        let claims = Claims {
            sub: uuid::Uuid::new_v4().to_string(),
            email: "seller@example.com".to_string(),
            role: "seller".to_string(),
            workshop_id: uuid::Uuid::new_v4().to_string(),
            exp: 1234567890,
        };

        let user = AuthenticatedUser::from_claims(claims).unwrap();
        assert!(!user.is_admin());
        assert!(matches!(user.role, UserRole::Seller));
    }

    #[test]
    fn test_authenticated_user_invalid_role() {
        let claims = Claims {
            sub: uuid::Uuid::new_v4().to_string(),
            email: "x@example.com".to_string(),
            role: "superuser".to_string(),
            workshop_id: uuid::Uuid::new_v4().to_string(),
            exp: 1234567890,
        };

        assert!(AuthenticatedUser::from_claims(claims).is_err());
    }

    #[test]
    fn test_authenticated_user_invalid_uuid() {
        let claims = Claims {
            sub: "not-a-uuid".to_string(),
            email: "x@example.com".to_string(),
            role: "admin".to_string(),
            workshop_id: uuid::Uuid::new_v4().to_string(),
            exp: 1234567890,
        };

        assert!(AuthenticatedUser::from_claims(claims).is_err());
    }

    fn make_license(tier: LicenseTier) -> License {
        License {
            license_key: "TEST".to_string(),
            tier,
            hardware_hash: "abc".to_string(),
            max_viewers: 1,
            max_transfers: 0,
            transfer_count: 0,
            activated_at: chrono::Utc::now(),
            expires_at: None,
        }
    }

    #[test]
    fn test_license_allows_none_is_trial_only() {
        assert!(license_allows(None, &Feature::Inventory));
        assert!(!license_allows(None, &Feature::ClientHistory));
    }

    #[test]
    fn test_license_allows_by_tier() {
        let trial = make_license(LicenseTier::Trial);
        assert!(!license_allows(Some(&trial), &Feature::ClientHistory));

        let base = make_license(LicenseTier::Base);
        assert!(!license_allows(Some(&base), &Feature::ClientHistory));

        let reports = make_license(LicenseTier::Reports);
        assert!(license_allows(Some(&reports), &Feature::ClientHistory));
        assert!(license_allows(Some(&reports), &Feature::PdfReports));
    }
}
