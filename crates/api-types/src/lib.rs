//! Stable wire types shared by the daemon and its clients.

use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub api_version: String,
    pub daemon_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Ok,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CapabilityResponse {
    pub api_versions: Vec<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SystemSummary {
    pub hostname: String,
    pub nixos_version: Option<String>,
    pub current_system: Option<String>,
    pub active_generation: Option<u64>,
    pub kernel_version: Option<String>,
    pub uptime_seconds: Option<u64>,
    pub cpu_count: Option<usize>,
    pub load_average: Option<String>,
    pub memory_total_bytes: Option<u64>,
    pub memory_available_bytes: Option<u64>,
    pub root_disk_total_bytes: Option<u64>,
    pub root_disk_available_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SystemGeneration {
    pub generation: u64,
    pub system_path: String,
    pub nixos_version: Option<String>,
    pub created_unix_seconds: Option<u64>,
    pub current: bool,
    pub active: bool,
    pub bootable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GenerationCatalog {
    pub total: usize,
    pub generations: Vec<SystemGeneration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GenerationDetail {
    pub generation: SystemGeneration,
    pub closure_size_bytes: u64,
    pub store_path_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StorePathDelta {
    pub path: String,
    pub name: String,
    pub nar_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GenerationComparison {
    pub from_generation: u64,
    pub to_generation: u64,
    pub added_size_bytes: u64,
    pub removed_size_bytes: u64,
    pub added: Vec<StorePathDelta>,
    pub removed: Vec<StorePathDelta>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConfigurationKind {
    Flake,
    Traditional,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ConfigurationSummary {
    pub kind: ConfigurationKind,
    pub source: Option<String>,
    pub flake_uri: Option<String>,
    pub host: Option<String>,
    pub has_lock_file: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FlakeInput {
    pub node: String,
    pub direct: bool,
    pub input_type: Option<String>,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub url: Option<String>,
    pub reference: Option<String>,
    pub revision: Option<String>,
    pub nar_hash: Option<String>,
    pub last_modified: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FlakeInputCatalog {
    pub lock_file: String,
    pub version: Option<u64>,
    pub total: usize,
    pub direct: usize,
    pub inputs: Vec<FlakeInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RepositoryStatus {
    pub root: String,
    pub branch: Option<String>,
    pub head: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u64,
    pub behind: u64,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub conflicted: usize,
    pub dirty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ManagedConfigurationMode {
    Unmanaged,
    ManualWiring,
    ControlPlaneManaged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ManagedConfigurationSummary {
    pub repository_root: Option<String>,
    pub managed_directory: Option<String>,
    pub host: Option<String>,
    pub host_module: Option<String>,
    pub host_module_exists: bool,
    pub metadata_exists: bool,
    pub mode: ManagedConfigurationMode,
    pub definitions_editable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct OptionSummary {
    pub name: String,
    pub description: Option<String>,
    pub option_type: Option<String>,
    pub read_only: bool,
    pub declarations: Vec<String>,
    #[serde(default)]
    pub current_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct OptionDetail {
    pub name: String,
    pub description: Option<String>,
    pub option_type: Option<String>,
    pub read_only: bool,
    pub declarations: Vec<String>,
    pub current_value: Option<String>,
    pub default_value: Option<String>,
    pub example: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ServiceSummary {
    pub name: String,
    pub enabled: Option<bool>,
    pub description: Option<String>,
    pub declarations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ServiceCatalog {
    pub host: String,
    pub total: usize,
    pub enabled: usize,
    pub services: Vec<ServiceSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ServiceDetail {
    pub name: String,
    pub options: Vec<OptionSummary>,
    pub systemd_units: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RuntimeService {
    pub unit: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RuntimeServiceCatalog {
    pub total: usize,
    pub failed: usize,
    pub services: Vec<RuntimeService>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct OptionCatalog {
    pub host: String,
    pub total: usize,
    pub options: Vec<OptionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SystemPackage {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub store_path: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PackageCatalog {
    pub host: String,
    pub total: usize,
    pub packages: Vec<SystemPackage>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub operation_id: Option<String>,
}
