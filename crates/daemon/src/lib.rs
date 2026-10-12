use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    routing::get,
};
use nixos_control_api_types::{
    API_VERSION, ApiError, CapabilityResponse, ConfigurationSummary, HealthResponse, HealthStatus,
    OptionCatalog, PackageCatalog, ServiceCatalog, SystemSummary,
};
use nixos_control_core::ControlPlane;
use nixos_control_nix::ConfigurationTarget;
use serde::Deserialize;
use std::time::Duration;
use tower_http::{request_id::MakeRequestUuid, trace::TraceLayer};

#[derive(Clone)]
struct AppState {
    control_plane: ControlPlane,
}

#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub command_timeout: Duration,
    pub configuration_target: ConfigurationTarget,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            command_timeout: Duration::from_secs(30),
            configuration_target: ConfigurationTarget::default(),
        }
    }
}

pub fn router(config: DaemonConfig) -> Router {
    let state = AppState {
        control_plane: ControlPlane::new(config.command_timeout, config.configuration_target),
    };
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/api", get(capabilities))
        .route("/api/v1/system/summary", get(system_summary))
        .route("/api/v1/configuration", get(configuration_summary))
        .route("/api/v1/options", get(option_catalog))
        .route("/api/v1/packages", get(package_catalog))
        .route("/api/v1/services", get(service_catalog))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(tower_http::request_id::SetRequestIdLayer::new(
            axum::http::HeaderName::from_static("x-request-id"),
            MakeRequestUuid,
        ))
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let system = state.control_plane.system_summary().await;
    let configuration = state.control_plane.configuration_summary();
    let options = state
        .control_plane
        .option_catalog()
        .await
        .map(|catalog| filter_catalog(catalog, "", 100))
        .map_err(|error| error.to_string());
    let packages = state
        .control_plane
        .package_catalog()
        .await
        .map(|catalog| filter_packages(catalog, "", 100))
        .map_err(|error| error.to_string());
    let services = state
        .control_plane
        .service_catalog()
        .await
        .map(|catalog| filter_services(catalog, "", 100))
        .map_err(|error| error.to_string());
    Html(nixos_control_ui::render_dashboard(
        system,
        configuration,
        options,
        packages,
        services,
    ))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: HealthStatus::Ok,
        api_version: API_VERSION.to_owned(),
        daemon_version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}

async fn capabilities() -> Json<CapabilityResponse> {
    Json(CapabilityResponse {
        api_versions: vec![API_VERSION.to_owned()],
        capabilities: vec![
            "configuration-discovery".to_owned(),
            "option-evaluation".to_owned(),
            "package-evaluation".to_owned(),
            "service-discovery".to_owned(),
            "system-summary".to_owned(),
        ],
    })
}

async fn system_summary(State(state): State<AppState>) -> Json<SystemSummary> {
    Json(state.control_plane.system_summary().await)
}

async fn configuration_summary(State(state): State<AppState>) -> Json<ConfigurationSummary> {
    Json(state.control_plane.configuration_summary())
}

async fn option_catalog(
    State(state): State<AppState>,
    Query(query): Query<OptionQuery>,
) -> Result<Json<OptionCatalog>, (StatusCode, Json<ApiError>)> {
    state
        .control_plane
        .option_catalog()
        .await
        .map(|catalog| {
            Json(filter_catalog(
                catalog,
                query.search.as_deref().unwrap_or(""),
                query.limit.unwrap_or(100).min(500),
            ))
        })
        .map_err(|error| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiError {
                    code: "nix_evaluation_failed".to_owned(),
                    message: error.to_string(),
                    operation_id: None,
                }),
            )
        })
}

#[derive(Debug, Deserialize)]
struct OptionQuery {
    #[serde(alias = "q")]
    search: Option<String>,
    limit: Option<usize>,
}

fn filter_catalog(mut catalog: OptionCatalog, search: &str, limit: usize) -> OptionCatalog {
    let query = search.trim().to_lowercase();
    if !query.is_empty() {
        catalog
            .options
            .retain(|option| option.name.to_lowercase().contains(&query));
    }
    catalog.total = catalog.options.len();
    catalog.options.truncate(limit);
    catalog
}

async fn package_catalog(
    State(state): State<AppState>,
    Query(query): Query<OptionQuery>,
) -> Result<Json<PackageCatalog>, (StatusCode, Json<ApiError>)> {
    state
        .control_plane
        .package_catalog()
        .await
        .map(|catalog| {
            Json(filter_packages(
                catalog,
                query.search.as_deref().unwrap_or(""),
                query.limit.unwrap_or(100).min(500),
            ))
        })
        .map_err(|error| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiError {
                    code: "nix_evaluation_failed".to_owned(),
                    message: error.to_string(),
                    operation_id: None,
                }),
            )
        })
}

fn filter_packages(mut catalog: PackageCatalog, search: &str, limit: usize) -> PackageCatalog {
    let query = search.trim().to_lowercase();
    if !query.is_empty() {
        catalog
            .packages
            .retain(|package| package.name.to_lowercase().contains(&query));
    }
    catalog.total = catalog.packages.len();
    catalog.packages.truncate(limit);
    catalog
}

async fn service_catalog(
    State(state): State<AppState>,
    Query(query): Query<OptionQuery>,
) -> Result<Json<ServiceCatalog>, (StatusCode, Json<ApiError>)> {
    state
        .control_plane
        .service_catalog()
        .await
        .map(|catalog| {
            Json(filter_services(
                catalog,
                query.search.as_deref().unwrap_or(""),
                query.limit.unwrap_or(100).min(500),
            ))
        })
        .map_err(|error| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiError {
                    code: "nix_evaluation_failed".to_owned(),
                    message: error.to_string(),
                    operation_id: None,
                }),
            )
        })
}

fn filter_services(mut catalog: ServiceCatalog, search: &str, limit: usize) -> ServiceCatalog {
    let query = search.trim().to_lowercase();
    if !query.is_empty() {
        catalog
            .services
            .retain(|service| service.name.to_lowercase().contains(&query));
    }
    catalog.total = catalog.services.len();
    catalog.enabled = catalog
        .services
        .iter()
        .filter(|service| service.enabled == Some(true))
        .count();
    catalog.services.truncate(limit);
    catalog
}

#[cfg(test)]
mod tests {
    use super::{DaemonConfig, router};
    use axum::{body::Body, http::Request};
    use nixos_control_api_types::{API_VERSION, HealthResponse, HealthStatus};
    use std::time::Duration;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_reports_versioned_api() {
        let response = router(DaemonConfig {
            command_timeout: Duration::from_secs(1),
            ..DaemonConfig::default()
        })
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
        assert!(response.status().is_success());
        let bytes = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let health: HealthResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(health.status, HealthStatus::Ok);
        assert_eq!(health.api_version, API_VERSION);
    }

    #[tokio::test]
    async fn dashboard_renders_discovered_state() {
        let response = router(DaemonConfig::default())
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(response.status().is_success());
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );
        let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains("NixOS Control Plane"));
        assert!(html.contains("Configuration"));
    }

    #[tokio::test]
    async fn configuration_endpoint_is_available() {
        let response = router(DaemonConfig::default())
            .oneshot(
                Request::get("/api/v1/configuration")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_success());
    }
}
