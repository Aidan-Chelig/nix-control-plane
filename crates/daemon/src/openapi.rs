//! Documentation generated from handler annotations and shared wire types.

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "NixOS Control Plane API",
        description = "Read-only inspection of the selected NixOS configuration and running system."
    ),
    paths(
        crate::health,
        crate::capabilities,
        crate::system_summary,
        crate::generations,
        crate::generation_detail,
        crate::generation_comparison,
        crate::configuration_summary,
        crate::managed_configuration_summary,
        crate::flake_inputs,
        crate::repository_status,
        crate::option_catalog,
        crate::option_detail,
        crate::package_catalog,
        crate::service_catalog,
        crate::service_detail,
        crate::runtime_services,
    )
)]
pub(crate) struct ApiDoc;
