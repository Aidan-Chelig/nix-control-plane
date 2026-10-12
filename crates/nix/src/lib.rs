//! Narrow, typed wrappers around Nix and NixOS inspection.

use serde::Deserialize;
use std::{
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemInspection {
    pub hostname: String,
    pub nixos_version: Option<String>,
    pub current_system: Option<String>,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatedOptions {
    pub host: String,
    pub options: Vec<EvaluatedOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EvaluatedPackage {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub store_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatedPackages {
    pub host: String,
    pub packages: Vec<EvaluatedPackage>,
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
        let packages: Vec<EvaluatedPackage> = self
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
        Ok(EvaluatedPackages { host, packages })
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
      }]
    else if builtins.isAttrs value then
      builtins.concatLists (builtins.map (name: walk (path ++ [name]) value.${name}) (builtins.attrNames value))
    else [];
in walk [] options"#;

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

    SystemInspection {
        hostname,
        nixos_version,
        current_system,
    }
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
        ConfigurationKind, ConfigurationTarget, InspectError, inspect_configuration, select_host,
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
}
