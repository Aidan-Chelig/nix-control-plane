//! Frontend-independent application services.

use nixos_control_api_types::{ConfigurationKind, ConfigurationSummary, SystemSummary};
use nixos_control_api_types::{
    OptionCatalog, OptionSummary, PackageCatalog, ServiceCatalog, ServiceSummary, SystemPackage,
};
use nixos_control_nix::{ConfigurationTarget, InspectError, NixEvaluator};
use std::{fmt::Write, sync::Arc, time::Duration};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ControlPlane {
    command_timeout: Duration,
    configuration_target: ConfigurationTarget,
    option_cache: Arc<RwLock<Option<CachedCatalog>>>,
    package_cache: Arc<RwLock<Option<CachedPackages>>>,
    service_cache: Arc<RwLock<Option<CachedServices>>>,
}

#[derive(Debug, Clone)]
struct CachedCatalog {
    fingerprint: String,
    catalog: OptionCatalog,
}

#[derive(Debug, Clone)]
struct CachedPackages {
    fingerprint: String,
    catalog: PackageCatalog,
}

#[derive(Debug, Clone)]
struct CachedServices {
    fingerprint: String,
    catalog: ServiceCatalog,
}

impl ControlPlane {
    #[must_use]
    pub fn new(command_timeout: Duration, configuration_target: ConfigurationTarget) -> Self {
        Self {
            command_timeout,
            configuration_target,
            option_cache: Arc::new(RwLock::new(None)),
            package_cache: Arc::new(RwLock::new(None)),
            service_cache: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn system_summary(&self) -> SystemSummary {
        let inspected = nixos_control_nix::inspect_local_system(self.command_timeout).await;
        SystemSummary {
            hostname: inspected.hostname,
            nixos_version: inspected.nixos_version,
            current_system: inspected.current_system,
        }
    }

    #[must_use]
    pub fn configuration_summary(&self) -> ConfigurationSummary {
        let inspected = nixos_control_nix::inspect_configuration(&self.configuration_target);
        ConfigurationSummary {
            kind: match inspected.kind {
                nixos_control_nix::ConfigurationKind::Flake => ConfigurationKind::Flake,
                nixos_control_nix::ConfigurationKind::Traditional => ConfigurationKind::Traditional,
                nixos_control_nix::ConfigurationKind::Unknown => ConfigurationKind::Unknown,
            },
            source: inspected
                .source
                .map(|path| path.to_string_lossy().into_owned()),
            flake_uri: inspected.flake_uri,
            host: inspected.host,
            has_lock_file: inspected.has_lock_file,
        }
    }

    /// Evaluates and returns the selected host's NixOS option catalog.
    ///
    /// # Errors
    /// Returns an inspection error when the target cannot be evaluated.
    pub async fn option_catalog(&self) -> Result<OptionCatalog, InspectError> {
        let fingerprint = source_fingerprint(&self.configuration_target);
        if let Some(fingerprint) = &fingerprint
            && let Some(cached) = self.option_cache.read().await.as_ref()
            && &cached.fingerprint == fingerprint
        {
            return Ok(cached.catalog.clone());
        }
        let system = nixos_control_nix::inspect_local_system(self.command_timeout).await;
        let evaluated = NixEvaluator::new(self.command_timeout)
            .evaluate_options(&self.configuration_target, &system.hostname)
            .await?;
        let options: Vec<_> = evaluated
            .options
            .into_iter()
            .map(|option| OptionSummary {
                name: option.name,
                description: option.description,
                option_type: option.option_type,
                read_only: option.read_only,
                declarations: option.declarations,
            })
            .collect();
        let catalog = OptionCatalog {
            host: evaluated.host,
            total: options.len(),
            options,
        };
        if let Some(fingerprint) = fingerprint {
            *self.option_cache.write().await = Some(CachedCatalog {
                fingerprint,
                catalog: catalog.clone(),
            });
        }
        Ok(catalog)
    }

    /// Evaluates the effective merged `environment.systemPackages` value.
    ///
    /// # Errors
    /// Returns an inspection error when the target cannot be evaluated.
    pub async fn package_catalog(&self) -> Result<PackageCatalog, InspectError> {
        let fingerprint = source_fingerprint(&self.configuration_target);
        if let Some(fingerprint) = &fingerprint
            && let Some(cached) = self.package_cache.read().await.as_ref()
            && &cached.fingerprint == fingerprint
        {
            return Ok(cached.catalog.clone());
        }
        let system = nixos_control_nix::inspect_local_system(self.command_timeout).await;
        let evaluated = NixEvaluator::new(self.command_timeout)
            .evaluate_packages(&self.configuration_target, &system.hostname)
            .await?;
        let packages: Vec<_> = evaluated
            .packages
            .into_iter()
            .map(|package| SystemPackage {
                name: package.name,
                version: package.version,
                description: package.description,
                store_path: package.store_path,
            })
            .collect();
        let catalog = PackageCatalog {
            host: evaluated.host,
            total: packages.len(),
            packages,
        };
        if let Some(fingerprint) = fingerprint {
            *self.package_cache.write().await = Some(CachedPackages {
                fingerprint,
                catalog: catalog.clone(),
            });
        }
        Ok(catalog)
    }

    /// Derives NixOS services with boolean `services.<name>.enable` options.
    ///
    /// # Errors
    /// Returns an inspection error when the option graph cannot be evaluated.
    pub async fn service_catalog(&self) -> Result<ServiceCatalog, InspectError> {
        let fingerprint = source_fingerprint(&self.configuration_target);
        if let Some(fingerprint) = &fingerprint
            && let Some(cached) = self.service_cache.read().await.as_ref()
            && &cached.fingerprint == fingerprint
        {
            return Ok(cached.catalog.clone());
        }
        let system = nixos_control_nix::inspect_local_system(self.command_timeout).await;
        let evaluated = NixEvaluator::new(self.command_timeout)
            .evaluate_services(&self.configuration_target, &system.hostname)
            .await?;
        let services: Vec<_> = evaluated
            .services
            .into_iter()
            .map(|service| ServiceSummary {
                name: service
                    .name
                    .strip_suffix(".enable")
                    .unwrap_or(&service.name)
                    .to_owned(),
                enabled: service.enabled,
                description: service.description,
                declarations: service.declarations,
            })
            .collect();
        let catalog = ServiceCatalog {
            host: evaluated.host,
            total: services.len(),
            enabled: services
                .iter()
                .filter(|service| service.enabled == Some(true))
                .count(),
            services,
        };
        if let Some(fingerprint) = fingerprint {
            *self.service_cache.write().await = Some(CachedServices {
                fingerprint,
                catalog: catalog.clone(),
            });
        }
        Ok(catalog)
    }
}

fn source_fingerprint(target: &ConfigurationTarget) -> Option<String> {
    let path = target.path.as_deref()?;
    let root = if path.is_file() { path.parent()? } else { path };
    let mut fingerprint = String::new();
    for candidate in [root.join("flake.nix"), root.join("flake.lock")] {
        let metadata = std::fs::metadata(&candidate).ok()?;
        let modified = metadata
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_nanos();
        write!(
            fingerprint,
            "{}:{modified}:{};",
            candidate.display(),
            metadata.len()
        )
        .expect("writing to a String cannot fail");
    }
    Some(fingerprint)
}
