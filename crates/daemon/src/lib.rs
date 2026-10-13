use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
    routing::get,
};
use nixos_control_api_types::{
    API_VERSION, ApiError, CapabilityResponse, ConfigurationSummary, GenerationCatalog,
    GenerationComparison, GenerationDetail, HealthResponse, HealthStatus,
    ManagedConfigurationSummary, OptionCatalog, OptionDetail, PackageCatalog,
    RuntimeServiceCatalog, ServiceCatalog, ServiceDetail, SystemSummary,
};
use nixos_control_core::ControlPlane;
use nixos_control_nix::{ConfigurationTarget, InspectError};
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
        .route("/configuration", get(configuration_page))
        .route("/configuration/options/{option}", get(option_detail_page))
        .route("/packages", get(packages_page))
        .route("/services", get(services_page))
        .route("/services/{service}", get(service_detail_page))
        .route("/runtime", get(runtime_page))
        .route("/generations", get(generations_page))
        .route("/generations/{generation}", get(generation_detail_page))
        .route(
            "/generations/{from}/compare/{to}",
            get(generation_comparison_page),
        )
        .route("/health", get(health))
        .route("/api", get(capabilities))
        .route("/api/v1/system/summary", get(system_summary))
        .route("/api/v1/generations", get(generations))
        .route("/api/v1/generations/{generation}", get(generation_detail))
        .route(
            "/api/v1/generations/{from}/compare/{to}",
            get(generation_comparison),
        )
        .route("/api/v1/configuration", get(configuration_summary))
        .route(
            "/api/v1/configuration/managed",
            get(managed_configuration_summary),
        )
        .route("/api/v1/options", get(option_catalog))
        .route("/api/v1/options/{option}", get(option_detail))
        .route("/api/v1/packages", get(package_catalog))
        .route("/api/v1/services", get(service_catalog))
        .route("/api/v1/services/{service}", get(service_detail))
        .route("/api/v1/runtime/services", get(runtime_services))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(tower_http::request_id::SetRequestIdLayer::new(
            axum::http::HeaderName::from_static("x-request-id"),
            MakeRequestUuid,
        ))
}

async fn index(State(state): State<AppState>) -> Html<String> {
    render_page(state, nixos_control_ui::DashboardPage::Overview).await
}

async fn configuration_page(State(state): State<AppState>) -> Html<String> {
    render_page(state, nixos_control_ui::DashboardPage::Configuration).await
}

async fn option_detail_page(
    State(state): State<AppState>,
    Path(option): Path<String>,
) -> Html<String> {
    let system = state.control_plane.system_summary().await;
    let configuration = state.control_plane.configuration_summary();
    let detail = if valid_option_name(&option) {
        state
            .control_plane
            .option_detail(&option)
            .await
            .map_err(|error| error.to_string())
    } else {
        Err("Invalid option name".to_owned())
    };
    Html(nixos_control_ui::render_option_detail(
        system,
        configuration,
        detail,
    ))
}

async fn packages_page(State(state): State<AppState>) -> Html<String> {
    render_page(state, nixos_control_ui::DashboardPage::Packages).await
}

async fn services_page(State(state): State<AppState>) -> Html<String> {
    render_page(state, nixos_control_ui::DashboardPage::Services).await
}

async fn service_detail_page(
    State(state): State<AppState>,
    Path(service): Path<String>,
) -> Html<String> {
    let system = state.control_plane.system_summary().await;
    let configuration = state.control_plane.configuration_summary();
    let detail = if valid_service_name(&service) {
        state
            .control_plane
            .service_detail(&service)
            .await
            .map_err(|error| error.to_string())
    } else {
        Err("Invalid service namespace".to_owned())
    };
    Html(nixos_control_ui::render_service_detail(
        system,
        configuration,
        detail,
    ))
}

async fn runtime_page(State(state): State<AppState>) -> Html<String> {
    render_page(state, nixos_control_ui::DashboardPage::Runtime).await
}

async fn generations_page(State(state): State<AppState>) -> Html<String> {
    render_page(state, nixos_control_ui::DashboardPage::Generations).await
}

async fn generation_detail_page(
    State(state): State<AppState>,
    Path(generation): Path<u64>,
) -> Html<String> {
    let system = state.control_plane.system_summary().await;
    let detail = state
        .control_plane
        .generation_detail(generation)
        .await
        .map_err(|error| error.to_string());
    Html(nixos_control_ui::render_generation_detail(system, detail))
}

async fn generation_comparison_page(
    State(state): State<AppState>,
    Path((from, to)): Path<(u64, u64)>,
) -> Html<String> {
    let system = state.control_plane.system_summary().await;
    let comparison = state
        .control_plane
        .compare_generations(from, to)
        .await
        .map_err(|error| error.to_string());
    Html(nixos_control_ui::render_generation_comparison(
        system, comparison,
    ))
}

async fn render_page(state: AppState, page: nixos_control_ui::DashboardPage) -> Html<String> {
    let system = state.control_plane.system_summary().await;
    let configuration = state.control_plane.configuration_summary();
    let managed_configuration = state.control_plane.managed_configuration_summary();
    let host = configuration.host.clone().unwrap_or_default();
    let options = if page == nixos_control_ui::DashboardPage::Configuration {
        state
            .control_plane
            .option_catalog()
            .await
            .map(|catalog| filter_catalog(catalog, "", 100))
            .map_err(|error| error.to_string())
    } else {
        Ok(OptionCatalog {
            host: host.clone(),
            total: 0,
            options: Vec::new(),
        })
    };
    let packages = if page == nixos_control_ui::DashboardPage::Packages {
        state
            .control_plane
            .package_catalog()
            .await
            .map(|catalog| filter_packages(catalog, "", None, 100))
            .map_err(|error| error.to_string())
    } else {
        Ok(PackageCatalog {
            host: host.clone(),
            total: 0,
            packages: Vec::new(),
            sources: Vec::new(),
        })
    };
    let services = if page == nixos_control_ui::DashboardPage::Services {
        state
            .control_plane
            .service_catalog()
            .await
            .map(|catalog| filter_services(catalog, "", 100))
            .map_err(|error| error.to_string())
    } else {
        Ok(ServiceCatalog {
            host,
            total: 0,
            enabled: 0,
            services: Vec::new(),
        })
    };
    let runtime_services = if page == nixos_control_ui::DashboardPage::Runtime {
        state
            .control_plane
            .runtime_services()
            .await
            .map(|catalog| filter_runtime_services(catalog, "", 100))
            .map_err(|error| error.to_string())
    } else {
        Ok(RuntimeServiceCatalog {
            total: 0,
            failed: 0,
            services: Vec::new(),
        })
    };
    let generations = if page == nixos_control_ui::DashboardPage::Generations {
        state.control_plane.generations()
    } else {
        GenerationCatalog {
            total: 0,
            generations: Vec::new(),
        }
    };
    Html(nixos_control_ui::render_dashboard(
        page,
        nixos_control_ui::DashboardData {
            system,
            configuration,
            managed_configuration,
            option_catalog: options,
            package_catalog: packages,
            service_catalog: services,
            runtime_services,
            generations,
        },
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
            "generation-history".to_owned(),
            "managed-configuration-ownership".to_owned(),
            "option-evaluation".to_owned(),
            "option-detail".to_owned(),
            "package-evaluation".to_owned(),
            "service-discovery".to_owned(),
            "systemd-service-status".to_owned(),
            "system-summary".to_owned(),
        ],
    })
}

async fn system_summary(State(state): State<AppState>) -> Json<SystemSummary> {
    Json(state.control_plane.system_summary().await)
}

async fn generations(State(state): State<AppState>) -> Json<GenerationCatalog> {
    Json(state.control_plane.generations())
}

async fn generation_detail(
    State(state): State<AppState>,
    Path(generation): Path<u64>,
) -> Result<Json<GenerationDetail>, (StatusCode, Json<ApiError>)> {
    state
        .control_plane
        .generation_detail(generation)
        .await
        .map(Json)
        .map_err(|error| {
            let (status, code) = if matches!(error, InspectError::GenerationNotFound(_)) {
                (StatusCode::NOT_FOUND, "generation_not_found")
            } else {
                (StatusCode::SERVICE_UNAVAILABLE, "nix_inspection_failed")
            };
            (
                status,
                Json(ApiError {
                    code: code.to_owned(),
                    message: error.to_string(),
                    operation_id: None,
                }),
            )
        })
}

async fn generation_comparison(
    State(state): State<AppState>,
    Path((from, to)): Path<(u64, u64)>,
) -> Result<Json<GenerationComparison>, (StatusCode, Json<ApiError>)> {
    state
        .control_plane
        .compare_generations(from, to)
        .await
        .map(Json)
        .map_err(|error| {
            let status = if matches!(error, InspectError::GenerationNotFound(_)) {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            };
            (
                status,
                Json(ApiError {
                    code: "generation_comparison_failed".to_owned(),
                    message: error.to_string(),
                    operation_id: None,
                }),
            )
        })
}

async fn configuration_summary(State(state): State<AppState>) -> Json<ConfigurationSummary> {
    Json(state.control_plane.configuration_summary())
}

async fn managed_configuration_summary(
    State(state): State<AppState>,
) -> Json<ManagedConfigurationSummary> {
    Json(state.control_plane.managed_configuration_summary())
}

async fn option_catalog(
    State(state): State<AppState>,
    Query(query): Query<CatalogQuery>,
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
struct CatalogQuery {
    #[serde(alias = "q")]
    search: Option<String>,
    limit: Option<usize>,
    source: Option<String>,
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
    Query(query): Query<CatalogQuery>,
) -> Result<Json<PackageCatalog>, (StatusCode, Json<ApiError>)> {
    state
        .control_plane
        .package_catalog()
        .await
        .map(|catalog| {
            Json(filter_packages(
                catalog,
                query.search.as_deref().unwrap_or(""),
                query.source.as_deref(),
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

async fn option_detail(
    State(state): State<AppState>,
    Path(option): Path<String>,
) -> Result<Json<OptionDetail>, (StatusCode, Json<ApiError>)> {
    if !valid_option_name(&option) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: "invalid_option".to_owned(),
                message: "option must be a valid dotted NixOS option name".to_owned(),
                operation_id: None,
            }),
        ));
    }
    state
        .control_plane
        .option_detail(&option)
        .await
        .map(Json)
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

fn filter_packages(
    mut catalog: PackageCatalog,
    search: &str,
    source: Option<&str>,
    limit: usize,
) -> PackageCatalog {
    let query = search.trim().to_lowercase();
    if !query.is_empty() {
        catalog
            .packages
            .retain(|package| package.name.to_lowercase().contains(&query));
    }
    if let Some(source) = source.filter(|source| !source.is_empty()) {
        catalog
            .packages
            .retain(|package| package.source.as_deref() == Some(source));
    }
    catalog.total = catalog.packages.len();
    catalog.packages.truncate(limit);
    catalog
}

async fn service_catalog(
    State(state): State<AppState>,
    Query(query): Query<CatalogQuery>,
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

async fn service_detail(
    State(state): State<AppState>,
    Path(service): Path<String>,
) -> Result<Json<ServiceDetail>, (StatusCode, Json<ApiError>)> {
    if !valid_service_name(&service) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: "invalid_service".to_owned(),
                message: "service must be a valid configured service name".to_owned(),
                operation_id: None,
            }),
        ));
    }
    state
        .control_plane
        .service_detail(&service)
        .await
        .map(Json)
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

fn valid_service_name(service: &str) -> bool {
    !service.is_empty()
        && service
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

fn valid_option_name(option: &str) -> bool {
    !option.is_empty()
        && option.split('.').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
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

async fn runtime_services(
    State(state): State<AppState>,
    Query(query): Query<CatalogQuery>,
) -> Result<Json<RuntimeServiceCatalog>, (StatusCode, Json<ApiError>)> {
    state
        .control_plane
        .runtime_services()
        .await
        .map(|catalog| {
            Json(filter_runtime_services(
                catalog,
                query.search.as_deref().unwrap_or(""),
                query.limit.unwrap_or(100).min(500),
            ))
        })
        .map_err(|error| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiError {
                    code: "systemd_unavailable".to_owned(),
                    message: error.to_string(),
                    operation_id: None,
                }),
            )
        })
}

fn filter_runtime_services(
    mut catalog: RuntimeServiceCatalog,
    search: &str,
    limit: usize,
) -> RuntimeServiceCatalog {
    let query = search.trim().to_lowercase();
    if !query.is_empty() {
        catalog.services.retain(|service| {
            service.unit.to_lowercase().contains(&query)
                || service.description.to_lowercase().contains(&query)
        });
    }
    catalog.total = catalog.services.len();
    catalog.failed = catalog
        .services
        .iter()
        .filter(|service| service.active_state == "failed")
        .count();
    catalog.services.truncate(limit);
    catalog
}

#[cfg(test)]
mod tests {
    use super::{DaemonConfig, filter_packages, router};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use nixos_control_api_types::{
        API_VERSION, HealthResponse, HealthStatus, PackageCatalog, SystemPackage,
    };
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
    async fn dashboard_subpages_are_routable() {
        for (path, expected) in [
            ("/configuration", "page-configuration"),
            ("/packages", "page-packages"),
            ("/services", "page-services"),
            ("/runtime", "page-runtime"),
            ("/generations", "page-generations"),
        ] {
            let response = router(DaemonConfig {
                command_timeout: Duration::from_millis(10),
                ..DaemonConfig::default()
            })
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
            assert!(response.status().is_success(), "route {path} failed");
            let bytes = axum::body::to_bytes(response.into_body(), 128 * 1024)
                .await
                .unwrap();
            let html = String::from_utf8(bytes.to_vec()).unwrap();
            assert!(
                html.contains(&format!("class=\"shell {expected}\"")),
                "route {path} rendered wrong page"
            );
        }
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

    #[tokio::test]
    async fn managed_configuration_endpoint_is_available() {
        let response = router(DaemonConfig::default())
            .oneshot(
                Request::get("/api/v1/configuration/managed")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_success());
    }

    #[tokio::test]
    async fn service_detail_rejects_non_service_namespaces() {
        let response = router(DaemonConfig::default())
            .oneshot(
                Request::get("/api/v1/services/invalid!")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn option_detail_rejects_invalid_names() {
        let response = router(DaemonConfig::default())
            .oneshot(
                Request::get("/api/v1/options/services..invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn missing_generation_returns_not_found() {
        let response = router(DaemonConfig::default())
            .oneshot(
                Request::get("/api/v1/generations/18446744073709551615")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn packages_can_be_filtered_by_source_and_name() {
        let package = |name: &str, source: Option<&str>| SystemPackage {
            name: name.to_owned(),
            version: None,
            description: None,
            store_path: format!("/nix/store/example-{name}"),
            source: source.map(str::to_owned),
        };
        let catalog = PackageCatalog {
            host: "example".to_owned(),
            total: 3,
            packages: vec![
                package("stable-tool", Some("nixpkgs")),
                package("unstable-tool", Some("nixpkgs-unstable")),
                package("local-tool", None),
            ],
            sources: vec!["nixpkgs".to_owned(), "nixpkgs-unstable".to_owned()],
        };

        let filtered = filter_packages(catalog, "tool", Some("nixpkgs-unstable"), 100);

        assert_eq!(filtered.total, 1);
        assert_eq!(filtered.packages[0].name, "unstable-tool");
        assert_eq!(filtered.sources, ["nixpkgs", "nixpkgs-unstable"]);
    }
}
