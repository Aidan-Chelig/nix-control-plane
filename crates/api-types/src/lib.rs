//! Stable wire types shared by the daemon and its clients.

use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub api_version: String,
    pub daemon_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Ok,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityResponse {
    pub api_versions: Vec<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemSummary {
    pub hostname: String,
    pub nixos_version: Option<String>,
    pub current_system: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigurationKind {
    Flake,
    Traditional,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigurationSummary {
    pub kind: ConfigurationKind,
    pub source: Option<String>,
    pub flake_uri: Option<String>,
    pub host: Option<String>,
    pub has_lock_file: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionSummary {
    pub name: String,
    pub description: Option<String>,
    pub option_type: Option<String>,
    pub read_only: bool,
    pub declarations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceSummary {
    pub name: String,
    pub enabled: Option<bool>,
    pub description: Option<String>,
    pub declarations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceCatalog {
    pub host: String,
    pub total: usize,
    pub enabled: usize,
    pub services: Vec<ServiceSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionCatalog {
    pub host: String,
    pub total: usize,
    pub options: Vec<OptionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemPackage {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub store_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageCatalog {
    pub host: String,
    pub total: usize,
    pub packages: Vec<SystemPackage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub operation_id: Option<String>,
}
