//! Narrow, typed wrappers around Nix and NixOS inspection.

use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Output,
    time::Duration,
};
use thiserror::Error;
use tokio::{process::Command, time::timeout};

#[derive(Debug, Error)]
pub enum InspectError {
    #[error("command timed out after {0:?}")]
    Timeout(Duration),
    #[error("could not start command: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("command failed: {0}")]
    Command(String),
    #[error("command returned invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("no NixOS configuration host was selected; available hosts: {0}")]
    HostRequired(String),
    #[error("option evaluation currently requires a flake configuration")]
    UnsupportedConfiguration,
    #[error("NixOS system generation {0} was not found")]
    GenerationNotFound(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemInspection {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemGenerationInspection {
    pub generation: u64,
    pub system_path: PathBuf,
    pub nixos_version: Option<String>,
    pub created_unix_seconds: Option<u64>,
    pub current: bool,
    pub active: bool,
    pub bootable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationDetailInspection {
    pub generation: SystemGenerationInspection,
    pub closure_size_bytes: u64,
    pub store_path_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NixPathInfo {
    closure_size: u64,
    nar_size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorePathDeltaInspection {
    pub path: String,
    pub name: String,
    pub nar_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationComparisonInspection {
    pub from_generation: u64,
    pub to_generation: u64,
    pub added_size_bytes: u64,
    pub removed_size_bytes: u64,
    pub added: Vec<StorePathDeltaInspection>,
    pub removed: Vec<StorePathDeltaInspection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationKind {
    Flake,
    Traditional,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationTarget {
    pub path: Option<PathBuf>,
    pub flake_uri: Option<String>,
    pub host: Option<String>,
}

impl Default for ConfigurationTarget {
    fn default() -> Self {
        Self {
            path: Some(PathBuf::from("/etc/nixos")),
            flake_uri: None,
            host: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationInspection {
    pub kind: ConfigurationKind,
    pub source: Option<PathBuf>,
    pub flake_uri: Option<String>,
    pub host: Option<String>,
    pub has_lock_file: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedConfigurationMode {
    Unmanaged,
    ManualWiring,
    ControlPlaneManaged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedConfigurationInspection {
    pub repository_root: Option<PathBuf>,
    pub managed_directory: Option<PathBuf>,
    pub host: Option<String>,
    pub host_module: Option<PathBuf>,
    pub host_module_exists: bool,
    pub metadata_exists: bool,
    pub mode: ManagedConfigurationMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EvaluatedOption {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub option_type: Option<String>,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub declarations: Vec<String>,
    #[serde(default)]
    pub current_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatedOptions {
    pub host: String,
    pub options: Vec<EvaluatedOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EvaluatedOptionDetail {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub option_type: Option<String>,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub declarations: Vec<String>,
    pub current_value: Option<String>,
    pub default_value: Option<String>,
    pub example: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EvaluatedServiceDetail {
    pub options: Vec<EvaluatedOption>,
    pub systemd_units: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EvaluatedPackage {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub store_path: String,
    pub source_position: Option<String>,
    #[serde(skip)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatedPackages {
    pub host: String,
    pub packages: Vec<EvaluatedPackage>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EvaluatedService {
    pub name: String,
    pub enabled: Option<bool>,
    pub description: Option<String>,
    #[serde(default)]
    pub declarations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatedServices {
    pub host: String,
    pub services: Vec<EvaluatedService>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RuntimeServiceInspection {
    pub unit: String,
    pub description: String,
    #[serde(rename = "load")]
    pub load_state: String,
    #[serde(rename = "active")]
    pub active_state: String,
    #[serde(rename = "sub")]
    pub sub_state: String,
}

/// Reads runtime service state from systemd's structured JSON output.
///
/// # Errors
/// Returns an error if systemd is unavailable, times out, or emits invalid JSON.
pub async fn inspect_runtime_services(
    command_timeout: Duration,
) -> Result<Vec<RuntimeServiceInspection>, InspectError> {
    let mut command = Command::new("systemctl");
    command.args([
        "list-units",
        "--type=service",
        "--all",
        "--no-pager",
        "--output=json",
    ]);
    let output = run_command(command_timeout, command).await?;
    serde_json::from_slice(&output.stdout).map_err(InspectError::from)
}

#[derive(Debug, Clone)]
pub struct NixEvaluator {
    timeout: Duration,
}

impl NixEvaluator {
    #[must_use]
    pub const fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Evaluates option metadata for a flake-backed NixOS configuration.
    ///
    /// # Errors
    /// Returns an error when no host can be selected, Nix fails or times out,
    /// or Nix emits a response that does not match the structured JSON schema.
    pub async fn evaluate_options(
        &self,
        target: &ConfigurationTarget,
        system_hostname: &str,
    ) -> Result<EvaluatedOptions, InspectError> {
        let (flake_uri, host) = self.resolve_host(target, system_hostname).await?;
        let options: Vec<EvaluatedOption> = self
            .run_json(&[
                "eval",
                "--json",
                "--no-write-lock-file",
                &format!("{flake_uri}#nixosConfigurations.{host}.options"),
                "--apply",
                OPTION_SUMMARY_EXPRESSION,
            ])
            .await?;
        Ok(EvaluatedOptions { host, options })
    }

    /// Evaluates option metadata and JSON-rendered values for one service subtree.
    ///
    /// # Errors
    /// Returns an error when the host, service subtree, or Nix evaluation fails.
    pub async fn evaluate_service_options(
        &self,
        target: &ConfigurationTarget,
        system_hostname: &str,
        service: &str,
    ) -> Result<EvaluatedServiceDetail, InspectError> {
        let (flake_uri, host) = self.resolve_host(target, system_hostname).await?;
        let path = service
            .split('.')
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()?
            .join(" ");
        let expression = SERVICE_OPTION_DETAIL_EXPRESSION.replace("@SERVICE_PATH@", &path);
        let detail: EvaluatedServiceDetail = self
            .run_json(&[
                "eval",
                "--json",
                "--no-write-lock-file",
                &format!("{flake_uri}#nixosConfigurations.{host}.options"),
                "--apply",
                &expression,
            ])
            .await?;
        Ok(detail)
    }

    /// Evaluates metadata and safely JSON-rendered values for one NixOS option.
    ///
    /// # Errors
    /// Returns an error when the host, option path, or Nix evaluation fails.
    pub async fn evaluate_option_detail(
        &self,
        target: &ConfigurationTarget,
        system_hostname: &str,
        option: &str,
    ) -> Result<EvaluatedOptionDetail, InspectError> {
        let (flake_uri, host) = self.resolve_host(target, system_hostname).await?;
        let path = option
            .split('.')
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()?
            .join(" ");
        let expression = OPTION_DETAIL_EXPRESSION.replace("@OPTION_PATH@", &path);
        self.run_json(&[
            "eval",
            "--json",
            "--no-write-lock-file",
            &format!("{flake_uri}#nixosConfigurations.{host}.options"),
            "--apply",
            &expression,
        ])
        .await
    }

    /// Evaluates the effective `environment.systemPackages` list.
    ///
    /// # Errors
    /// Returns an error when host resolution or Nix evaluation fails.
    pub async fn evaluate_packages(
        &self,
        target: &ConfigurationTarget,
        system_hostname: &str,
    ) -> Result<EvaluatedPackages, InspectError> {
        let (flake_uri, host) = self.resolve_host(target, system_hostname).await?;
        let mut packages: Vec<EvaluatedPackage> = self
            .run_json(&[
                "eval",
                "--json",
                "--no-write-lock-file",
                &format!(
                    "{flake_uri}#nixosConfigurations.{host}.config.environment.systemPackages"
                ),
                "--apply",
                PACKAGE_SUMMARY_EXPRESSION,
            ])
            .await?;
        // `builtins.getFlake` requires local references to be absolute, while
        // installable syntax accepted by `nix eval` also permits relative paths.
        let input_flake_uri = std::fs::canonicalize(&flake_uri).map_or_else(
            |_| flake_uri.clone(),
            |path| path.to_string_lossy().into_owned(),
        );
        let quoted_uri = serde_json::to_string(&input_flake_uri)?;
        let input_paths: BTreeMap<String, String> = self
            .run_json(&[
                "eval",
                "--json",
                "--impure",
                "--expr",
                &format!(
                    "builtins.mapAttrs (_: input: toString input.outPath) (builtins.getFlake {quoted_uri}).inputs"
                ),
            ])
            .await?;
        for package in &mut packages {
            package.source = package.source_position.as_deref().and_then(|position| {
                input_paths
                    .iter()
                    .filter(|(_, path)| position.starts_with(path.as_str()))
                    .max_by_key(|(_, path)| path.len())
                    .map(|(name, _)| name.clone())
            });
        }
        Ok(EvaluatedPackages {
            host,
            packages,
            sources: input_paths.into_keys().collect(),
        })
    }

    /// Evaluates boolean `services.*.enable` options only.
    ///
    /// # Errors
    /// Returns an error when host resolution or Nix evaluation fails.
    pub async fn evaluate_services(
        &self,
        target: &ConfigurationTarget,
        system_hostname: &str,
    ) -> Result<EvaluatedServices, InspectError> {
        let (flake_uri, host) = self.resolve_host(target, system_hostname).await?;
        let services: Vec<EvaluatedService> = self
            .run_json(&[
                "eval",
                "--json",
                "--no-write-lock-file",
                &format!("{flake_uri}#nixosConfigurations.{host}.options.services"),
                "--apply",
                SERVICE_SUMMARY_EXPRESSION,
            ])
            .await?;
        Ok(EvaluatedServices { host, services })
    }

    async fn resolve_host(
        &self,
        target: &ConfigurationTarget,
        system_hostname: &str,
    ) -> Result<(String, String), InspectError> {
        let inspection = inspect_configuration(target);
        if inspection.kind != ConfigurationKind::Flake {
            return Err(InspectError::UnsupportedConfiguration);
        }
        let flake_uri = inspection
            .flake_uri
            .ok_or(InspectError::UnsupportedConfiguration)?;
        let hosts: Vec<String> = self
            .run_json(&[
                "eval",
                "--json",
                "--no-write-lock-file",
                &format!("{flake_uri}#nixosConfigurations"),
                "--apply",
                "builtins.attrNames",
            ])
            .await?;
        let host = select_host(inspection.host.as_deref(), system_hostname, &hosts)?;
        Ok((flake_uri, host))
    }

    async fn run_json<T: serde::de::DeserializeOwned>(
        &self,
        arguments: &[&str],
    ) -> Result<T, InspectError> {
        let mut command = Command::new("nix");
        command.args(arguments).kill_on_drop(true);
        let output = run_command(self.timeout, command).await?;
        serde_json::from_slice(&output.stdout).map_err(InspectError::from)
    }
}

const OPTION_SUMMARY_EXPRESSION: &str = r#"options:
let
  walk = path: value:
    if builtins.isAttrs value && value ? _type && value._type == "option" then
      let
        description = value.description or null;
        typeDescription = if value ? type then value.type.description or null else null;
      in [{
        name = builtins.concatStringsSep "." path;
        description = if builtins.isString description then description else null;
        type = if builtins.isString typeDescription then typeDescription else null;
        read_only = value.readOnly or false;
        declarations = builtins.map toString (value.declarations or []);
        current_value = null;
      }]
    else if builtins.isAttrs value then
      builtins.concatLists (builtins.map (name: walk (path ++ [name]) value.${name}) (builtins.attrNames value))
    else [];
in walk [] options"#;

const OPTION_DETAIL_EXPRESSION: &str = r#"options:
let
  optionPath = [ @OPTION_PATH@ ];
  selected = builtins.foldl' (value: name: builtins.getAttr name value) options optionPath;
  render = candidate:
    let attempted = builtins.tryEval (builtins.toJSON candidate);
    in if attempted.success then attempted.value else null;
  renderDocumentation = candidate:
    if builtins.isAttrs candidate && candidate ? text then candidate.text
    else render candidate;
  description = selected.description or null;
  typeDescription = if selected ? type then selected.type.description or null else null;
in {
  name = builtins.concatStringsSep "." optionPath;
  description = if builtins.isString description then description else null;
  type = if builtins.isString typeDescription then typeDescription else null;
  read_only = selected.readOnly or false;
  declarations = builtins.map toString (selected.declarations or []);
  current_value = render (selected.value or null);
  default_value =
    if selected ? default then render selected.default
    else if selected ? defaultText then renderDocumentation selected.defaultText
    else null;
  example = if selected ? example then renderDocumentation selected.example else null;
}"#;

const SERVICE_OPTION_DETAIL_EXPRESSION: &str = r#"options:
let
  servicePath = [ @SERVICE_PATH@ ];
  selected = builtins.foldl' (value: name: builtins.getAttr name value) options.services servicePath;
  walk = path: value:
    if builtins.isAttrs value && value ? _type && value._type == "option" then
      let
        description = value.description or null;
        typeDescription = if value ? type then value.type.description or null else null;
        rendered = builtins.tryEval (builtins.toJSON (value.value or null));
      in [{
        name = builtins.concatStringsSep "." ([ "services" ] ++ servicePath ++ path);
        description = if builtins.isString description then description else null;
        type = if builtins.isString typeDescription then typeDescription else null;
        read_only = value.readOnly or false;
        declarations = builtins.map toString (value.declarations or []);
        current_value = if rendered.success then rendered.value else null;
      }]
    else if builtins.isAttrs value then
      builtins.concatLists (builtins.map (name: walk (path ++ [name]) value.${name}) (builtins.attrNames value))
    else [];
  declarationFiles = builtins.concatLists (builtins.map (option: option.declarations) (walk [] selected));
  unitDefinitions = options.systemd.services.definitionsWithLocations;
  matchingDefinitions = builtins.filter
    (definition: builtins.elem (toString definition.file) declarationFiles)
    unitDefinitions;
in {
  options = walk [] selected;
  systemd_units = builtins.attrNames (builtins.foldl'
    (units: definition: units // definition.value)
    {}
    matchingDefinitions);
}"#;

const PACKAGE_SUMMARY_EXPRESSION: &str = r#"packages:
builtins.map (package:
  let
    isPackage = builtins.isAttrs package;
    description = if isPackage then package.meta.description or null else null;
  in {
    name = if isPackage then package.pname or package.name or "unknown" else "unknown";
    version = if isPackage then package.version or null else null;
    description = if builtins.isString description then description else null;
    store_path = toString package;
    source_position = if isPackage then package.meta.position or null else null;
  }
) packages"#;

const SERVICE_SUMMARY_EXPRESSION: &str = r#"services:
let
  walk = path: value:
    if builtins.isAttrs value && value ? _type && value._type == "option" then
      if (value.visible or true) != false && builtins.elemAt path (builtins.length path - 1) == "enable" then
        let
          attempted = builtins.tryEval (builtins.deepSeq (value.value or null) (value.value or null));
          description = value.description or null;
        in [{
          name = builtins.concatStringsSep "." path;
          enabled = if attempted.success && builtins.isBool attempted.value then attempted.value else null;
          description = if builtins.isString description then description else null;
          declarations = builtins.map toString (value.declarations or []);
        }]
      else []
    else if builtins.isAttrs value then
      builtins.concatLists (builtins.map (name: walk (path ++ [name]) value.${name}) (builtins.attrNames value))
    else [];
in walk [] services"#;

fn select_host(
    requested: Option<&str>,
    system_hostname: &str,
    available: &[String],
) -> Result<String, InspectError> {
    if let Some(host) = requested {
        if available.iter().any(|candidate| candidate == host) {
            return Ok(host.to_owned());
        }
        return Err(InspectError::HostRequired(available.join(", ")));
    }
    if available.iter().any(|host| host == system_hostname) {
        return Ok(system_hostname.to_owned());
    }
    if let [host] = available {
        return Ok(host.clone());
    }
    Err(InspectError::HostRequired(available.join(", ")))
}

#[must_use]
pub fn inspect_configuration(target: &ConfigurationTarget) -> ConfigurationInspection {
    if let Some(uri) = &target.flake_uri {
        return ConfigurationInspection {
            kind: ConfigurationKind::Flake,
            source: None,
            flake_uri: Some(uri.clone()),
            host: target.host.clone(),
            has_lock_file: false,
        };
    }

    let Some(path) = target.path.as_deref() else {
        return ConfigurationInspection {
            kind: ConfigurationKind::Unknown,
            source: None,
            flake_uri: None,
            host: target.host.clone(),
            has_lock_file: false,
        };
    };
    let root = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    let flake = if path.file_name().is_some_and(|name| name == "flake.nix") {
        path.to_path_buf()
    } else {
        root.join("flake.nix")
    };
    let configuration = if path
        .file_name()
        .is_some_and(|name| name == "configuration.nix")
    {
        path.to_path_buf()
    } else {
        root.join("configuration.nix")
    };

    if flake.is_file() {
        let canonical_root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
        ConfigurationInspection {
            kind: ConfigurationKind::Flake,
            source: Some(canonical_root.join("flake.nix")),
            flake_uri: Some(canonical_root.to_string_lossy().into_owned()),
            host: target.host.clone(),
            has_lock_file: root.join("flake.lock").is_file(),
        }
    } else if configuration.is_file() {
        ConfigurationInspection {
            kind: ConfigurationKind::Traditional,
            source: Some(configuration),
            flake_uri: None,
            host: target.host.clone(),
            has_lock_file: false,
        }
    } else {
        ConfigurationInspection {
            kind: ConfigurationKind::Unknown,
            source: Some(path.to_path_buf()),
            flake_uri: None,
            host: target.host.clone(),
            has_lock_file: false,
        }
    }
}

/// Inspects the reserved control-plane area without modifying the repository.
#[must_use]
pub fn inspect_managed_configuration(
    target: &ConfigurationTarget,
) -> ManagedConfigurationInspection {
    let inspection = inspect_configuration(target);
    let repository_root = target
        .flake_uri
        .as_deref()
        .map(Path::new)
        .filter(|path| path.is_dir())
        .and_then(|path| std::fs::canonicalize(path).ok())
        .or_else(|| {
            inspection
                .source
                .as_deref()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
        });
    let managed_directory = repository_root.as_ref().map(|root| root.join("ncp"));
    let metadata = managed_directory
        .as_ref()
        .map(|directory| directory.join("metadata.json"));
    let metadata_exists = metadata.as_ref().is_some_and(|path| path.is_file());
    let ownership_metadata = metadata.as_ref().and_then(|path| {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|contents| serde_json::from_str::<serde_json::Value>(&contents).ok())
            .and_then(|value| {
                (value.get("owner").and_then(serde_json::Value::as_str)
                    == Some("nixos-control-plane"))
                .then(|| {
                    value
                        .get("repository_managed")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false)
                })
            })
    });
    let host_module = inspection
        .host
        .as_ref()
        .zip(managed_directory.as_ref())
        .map(|(host, directory)| directory.join("hosts").join(format!("{host}.nix")));
    let host_module_exists = host_module.as_ref().is_some_and(|path| path.is_file());
    let mode = match ownership_metadata {
        Some(true) => ManagedConfigurationMode::ControlPlaneManaged,
        Some(false) => ManagedConfigurationMode::ManualWiring,
        None => ManagedConfigurationMode::Unmanaged,
    };
    ManagedConfigurationInspection {
        repository_root,
        managed_directory,
        host: inspection.host,
        host_module,
        host_module_exists,
        metadata_exists,
        mode,
    }
}

/// Returns true only for definitions contained by the reserved managed directory.
#[must_use]
pub fn definition_is_control_plane_owned(
    managed: &ManagedConfigurationInspection,
    declaration: &Path,
) -> bool {
    if managed.mode == ManagedConfigurationMode::Unmanaged {
        return false;
    }
    managed.managed_directory.as_ref().is_some_and(|directory| {
        let canonical_directory = std::fs::canonicalize(directory).ok();
        let canonical_declaration = std::fs::canonicalize(declaration).ok();
        canonical_directory
            .zip(canonical_declaration)
            .is_some_and(|(directory, declaration)| declaration.starts_with(directory))
    })
}

pub async fn inspect_local_system(command_timeout: Duration) -> SystemInspection {
    let hostname =
        read_trimmed("/proc/sys/kernel/hostname").unwrap_or_else(|| "unknown".to_owned());
    let nixos_version = read_trimmed("/etc/os-release").and_then(|contents| {
        contents.lines().find_map(|line| {
            line.strip_prefix("VERSION_ID=")
                .map(|value| value.trim_matches('"').to_owned())
        })
    });
    let current_system = run_readlink(command_timeout, "/run/current-system")
        .await
        .ok();
    let active_generation = std::fs::read_link("/nix/var/nix/profiles/system")
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .and_then(|name| {
            name.strip_prefix("system-")
                .and_then(|value| value.strip_suffix("-link"))
                .and_then(|value| value.parse().ok())
        });
    let kernel_version = read_trimmed("/proc/sys/kernel/osrelease");
    let uptime_seconds = read_trimmed("/proc/uptime").and_then(|value| {
        value
            .split_whitespace()
            .next()?
            .split('.')
            .next()?
            .parse::<u64>()
            .ok()
    });
    let cpu_count = std::thread::available_parallelism().ok().map(usize::from);
    let load_average = read_trimmed("/proc/loadavg").map(|value| {
        value
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    });
    let memory = read_trimmed("/proc/meminfo");
    let memory_total_bytes = memory
        .as_deref()
        .and_then(|value| meminfo_bytes(value, "MemTotal"));
    let memory_available_bytes = memory
        .as_deref()
        .and_then(|value| meminfo_bytes(value, "MemAvailable"));
    let (root_disk_total_bytes, root_disk_available_bytes) = inspect_root_disk(command_timeout)
        .await
        .unwrap_or((None, None));

    SystemInspection {
        hostname,
        nixos_version,
        current_system,
        active_generation,
        kernel_version,
        uptime_seconds,
        cpu_count,
        load_average,
        memory_total_bytes,
        memory_available_bytes,
        root_disk_total_bytes,
        root_disk_available_bytes,
    }
}

/// Lists valid NixOS system profile generations without mutating the profile.
#[must_use]
pub fn inspect_system_generations() -> Vec<SystemGenerationInspection> {
    inspect_generations_at(
        Path::new("/nix/var/nix/profiles"),
        Path::new("/run/current-system"),
    )
}

/// Inspects immutable closure metadata for one system profile generation.
///
/// # Errors
/// Returns an error when the generation is absent or Nix cannot query its closure.
pub async fn inspect_generation_detail(
    command_timeout: Duration,
    number: u64,
) -> Result<GenerationDetailInspection, InspectError> {
    let generation = inspect_system_generations()
        .into_iter()
        .find(|generation| generation.generation == number)
        .ok_or(InspectError::GenerationNotFound(number))?;
    let system_path = generation.system_path.to_string_lossy().into_owned();
    let paths = query_closure(command_timeout, &system_path).await?;
    let closure_size_bytes = paths
        .get(&system_path)
        .map(|info| info.closure_size)
        .ok_or_else(|| InspectError::Command("Nix omitted the generation root path".to_owned()))?;
    Ok(GenerationDetailInspection {
        generation,
        closure_size_bytes,
        store_path_count: paths.len(),
    })
}

/// Compares the exact store closures of two installed generations.
///
/// # Errors
/// Returns an error when either generation is absent or Nix cannot query it.
pub async fn compare_generations(
    command_timeout: Duration,
    from: u64,
    to: u64,
) -> Result<GenerationComparisonInspection, InspectError> {
    let generations = inspect_system_generations();
    let find = |number| {
        generations
            .iter()
            .find(|generation| generation.generation == number)
            .ok_or(InspectError::GenerationNotFound(number))
    };
    let from_path = find(from)?.system_path.to_string_lossy().into_owned();
    let to_path = find(to)?.system_path.to_string_lossy().into_owned();
    let (from_paths, to_paths) = tokio::try_join!(
        query_closure(command_timeout, &from_path),
        query_closure(command_timeout, &to_path)
    )?;
    let collect = |paths: &BTreeMap<String, NixPathInfo>, other: &BTreeMap<String, NixPathInfo>| {
        paths
            .iter()
            .filter(|(path, _)| !other.contains_key(*path))
            .map(|(path, info)| StorePathDeltaInspection {
                path: path.clone(),
                name: store_path_name(path),
                nar_size_bytes: info.nar_size,
            })
            .collect::<Vec<_>>()
    };
    let added = collect(&to_paths, &from_paths);
    let removed = collect(&from_paths, &to_paths);
    Ok(GenerationComparisonInspection {
        from_generation: from,
        to_generation: to,
        added_size_bytes: added.iter().map(|path| path.nar_size_bytes).sum(),
        removed_size_bytes: removed.iter().map(|path| path.nar_size_bytes).sum(),
        added,
        removed,
    })
}

async fn query_closure(
    command_timeout: Duration,
    system_path: &str,
) -> Result<BTreeMap<String, NixPathInfo>, InspectError> {
    let mut command = Command::new("nix");
    command.args([
        "path-info",
        "--json",
        "--json-format",
        "1",
        "--closure-size",
        "--recursive",
        system_path,
    ]);
    let output = run_command(command_timeout, command).await?;
    serde_json::from_slice(&output.stdout).map_err(InspectError::from)
}

fn store_path_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.split_once('-').map(|(_, name)| name))
        .unwrap_or(path)
        .to_owned()
}

fn inspect_generations_at(
    profile_directory: &Path,
    active_system: &Path,
) -> Vec<SystemGenerationInspection> {
    let active_path = std::fs::canonicalize(active_system).ok();
    let current_path = std::fs::canonicalize(profile_directory.join("system")).ok();
    let Ok(entries) = std::fs::read_dir(profile_directory) else {
        return Vec::new();
    };
    let mut generations = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let file_name = entry.file_name();
            let file_name = file_name.to_str()?;
            let generation = file_name
                .strip_prefix("system-")?
                .strip_suffix("-link")?
                .parse::<u64>()
                .ok()?;
            let profile_path = entry.path();
            let system_path = std::fs::canonicalize(&profile_path).ok()?;
            let created_unix_seconds = std::fs::symlink_metadata(&profile_path)
                .ok()?
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs());
            let nixos_version = read_trimmed(system_path.join("nixos-version"));
            let bootable = system_path.join("init").exists() && system_path.join("kernel").exists();
            Some(SystemGenerationInspection {
                generation,
                current: current_path.as_ref() == Some(&system_path),
                active: active_path.as_ref() == Some(&system_path),
                bootable,
                system_path,
                nixos_version,
                created_unix_seconds,
            })
        })
        .collect::<Vec<_>>();
    generations.sort_by_key(|generation| std::cmp::Reverse(generation.generation));
    generations
}

fn meminfo_bytes(contents: &str, field: &str) -> Option<u64> {
    contents.lines().find_map(|line| {
        let value = line.strip_prefix(field)?.trim_start_matches(':').trim();
        value
            .split_whitespace()
            .next()?
            .parse::<u64>()
            .ok()
            .map(|kilobytes| kilobytes * 1024)
    })
}

async fn inspect_root_disk(duration: Duration) -> Result<(Option<u64>, Option<u64>), InspectError> {
    let mut command = Command::new("df");
    command.args(["-B1", "--output=size,avail", "/"]);
    let output = run_command(duration, command).await?;
    let line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .nth(1)
        .map(str::to_owned);
    let mut fields = line.as_deref().unwrap_or_default().split_whitespace();
    Ok((
        fields.next().and_then(|value| value.parse().ok()),
        fields.next().and_then(|value| value.parse().ok()),
    ))
}

fn read_trimmed(path: impl AsRef<Path>) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_owned())
}

async fn run_readlink(duration: Duration, path: &str) -> Result<String, InspectError> {
    let mut command = Command::new("readlink");
    command.args(["-f", path]);
    let output = run_command(duration, command).await?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

async fn run_command(duration: Duration, mut command: Command) -> Result<Output, InspectError> {
    command.kill_on_drop(true);
    let output = timeout(duration, command.output())
        .await
        .map_err(|_| InspectError::Timeout(duration))??;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(InspectError::Command(stderr));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{
        ConfigurationKind, ConfigurationTarget, InspectError, ManagedConfigurationMode,
        definition_is_control_plane_owned, inspect_configuration, inspect_generations_at,
        inspect_managed_configuration, select_host, store_path_name,
    };

    #[test]
    fn detects_flake_and_lock_file() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("flake.nix"), "{}").unwrap();
        std::fs::write(directory.path().join("flake.lock"), "{}").unwrap();
        let result = inspect_configuration(&ConfigurationTarget {
            path: Some(directory.path().to_path_buf()),
            flake_uri: None,
            host: Some("test-host".to_owned()),
        });
        assert_eq!(result.kind, ConfigurationKind::Flake);
        assert!(result.has_lock_file);
        assert_eq!(result.host.as_deref(), Some("test-host"));
    }

    #[test]
    fn detects_traditional_configuration() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("configuration.nix"), "{}").unwrap();
        let result = inspect_configuration(&ConfigurationTarget {
            path: Some(directory.path().to_path_buf()),
            flake_uri: None,
            host: None,
        });
        assert_eq!(result.kind, ConfigurationKind::Traditional);
    }

    #[test]
    fn explicit_flake_uri_does_not_require_local_files() {
        let result = inspect_configuration(&ConfigurationTarget {
            path: None,
            flake_uri: Some("github:example/config".to_owned()),
            host: None,
        });
        assert_eq!(result.kind, ConfigurationKind::Flake);
        assert_eq!(result.flake_uri.as_deref(), Some("github:example/config"));
    }

    #[test]
    fn selects_requested_hostname_or_only_host() {
        let hosts = vec!["alpha".to_owned(), "beta".to_owned()];
        assert_eq!(select_host(Some("beta"), "alpha", &hosts).unwrap(), "beta");
        assert_eq!(select_host(None, "alpha", &hosts).unwrap(), "alpha");
        assert_eq!(
            select_host(None, "other", &["only".to_owned()]).unwrap(),
            "only"
        );
        assert!(matches!(
            select_host(None, "other", &hosts),
            Err(InspectError::HostRequired(_))
        ));
    }

    #[test]
    fn recognizes_managed_host_modules_and_rejects_outside_definitions() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("flake.nix"), "{}").unwrap();
        let hosts = directory.path().join("ncp/hosts");
        std::fs::create_dir_all(&hosts).unwrap();
        let host_module = hosts.join("alpha.nix");
        std::fs::write(&host_module, "{ ... }: {}").unwrap();
        std::fs::write(
            directory.path().join("ncp/metadata.json"),
            r#"{"owner":"nixos-control-plane","repository_managed":true}"#,
        )
        .unwrap();
        let outside = directory.path().join("configuration.nix");
        std::fs::write(&outside, "{}").unwrap();

        let inspected = inspect_managed_configuration(&ConfigurationTarget {
            path: Some(directory.path().to_path_buf()),
            flake_uri: None,
            host: Some("alpha".to_owned()),
        });

        assert_eq!(
            inspected.mode,
            ManagedConfigurationMode::ControlPlaneManaged
        );
        assert!(inspected.host_module_exists);
        assert!(definition_is_control_plane_owned(&inspected, &host_module));
        assert!(!definition_is_control_plane_owned(&inspected, &outside));
    }

    #[cfg(unix)]
    #[test]
    fn discovers_and_classifies_system_generations() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().unwrap();
        let profiles = temporary.path().join("profiles");
        let system = temporary.path().join("system-store-path");
        std::fs::create_dir_all(&profiles).unwrap();
        std::fs::create_dir_all(&system).unwrap();
        std::fs::write(system.join("nixos-version"), "26.11\n").unwrap();
        std::fs::write(system.join("init"), "").unwrap();
        std::fs::write(system.join("kernel"), "").unwrap();
        symlink(&system, profiles.join("system-7-link")).unwrap();
        symlink("system-7-link", profiles.join("system")).unwrap();
        let active = temporary.path().join("active");
        symlink(&system, &active).unwrap();

        let generations = inspect_generations_at(&profiles, &active);

        assert_eq!(generations.len(), 1);
        assert_eq!(generations[0].generation, 7);
        assert_eq!(generations[0].nixos_version.as_deref(), Some("26.11"));
        assert!(generations[0].current);
        assert!(generations[0].active);
        assert!(generations[0].bootable);
    }

    #[test]
    fn extracts_readable_store_path_names() {
        assert_eq!(
            store_path_name("/nix/store/0123456789abcdefghijklmnopqrstuv-hello-1.0"),
            "hello-1.0"
        );
    }

    #[test]
    fn directory_name_alone_does_not_claim_ownership() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("flake.nix"), "{}").unwrap();
        std::fs::create_dir(directory.path().join("ncp")).unwrap();

        let inspected = inspect_managed_configuration(&ConfigurationTarget {
            path: Some(directory.path().to_path_buf()),
            flake_uri: None,
            host: Some("alpha".to_owned()),
        });

        assert_eq!(inspected.mode, ManagedConfigurationMode::Unmanaged);
    }
}
