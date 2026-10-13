//! Frontend-independent application services.

use nixos_control_api_types::{
    ConfigurationKind, ConfigurationSummary, GenerationCatalog, GenerationDetail,
    ManagedConfigurationMode, ManagedConfigurationSummary, SystemGeneration, SystemSummary,
};
use nixos_control_api_types::{
    OptionCatalog, OptionDetail, OptionSummary, PackageCatalog, RuntimeService,
    RuntimeServiceCatalog, ServiceCatalog, ServiceDetail, ServiceSummary, SystemPackage,
};
use nixos_control_nix::{ConfigurationTarget, InspectError, NixEvaluator};
use std::{collections::HashMap, fmt::Write, path::Path, sync::Arc, time::Duration};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ControlPlane {
    command_timeout: Duration,
    configuration_target: ConfigurationTarget,
    option_cache: Arc<RwLock<Option<CachedCatalog>>>,
    option_detail_cache: Arc<RwLock<HashMap<String, CachedOptionDetail>>>,
    package_cache: Arc<RwLock<Option<CachedPackages>>>,
    service_cache: Arc<RwLock<Option<CachedServices>>>,
    service_detail_cache: Arc<RwLock<HashMap<String, CachedServiceDetail>>>,
    generation_detail_cache: Arc<RwLock<HashMap<u64, GenerationDetail>>>,
}

#[derive(Debug, Clone)]
struct CachedCatalog {
    fingerprint: String,
    catalog: OptionCatalog,
}

#[derive(Debug, Clone)]
struct CachedOptionDetail {
    fingerprint: String,
    detail: OptionDetail,
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

#[derive(Debug, Clone)]
struct CachedServiceDetail {
    fingerprint: String,
    detail: ServiceDetail,
}

impl ControlPlane {
    #[must_use]
    pub fn new(command_timeout: Duration, configuration_target: ConfigurationTarget) -> Self {
        Self {
            command_timeout,
            configuration_target,
            option_cache: Arc::new(RwLock::new(None)),
            option_detail_cache: Arc::new(RwLock::new(HashMap::new())),
            package_cache: Arc::new(RwLock::new(None)),
            service_cache: Arc::new(RwLock::new(None)),
            service_detail_cache: Arc::new(RwLock::new(HashMap::new())),
            generation_detail_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn system_summary(&self) -> SystemSummary {
        let inspected = nixos_control_nix::inspect_local_system(self.command_timeout).await;
        SystemSummary {
            hostname: inspected.hostname,
            nixos_version: inspected.nixos_version,
            current_system: inspected.current_system,
            active_generation: inspected.active_generation,
            kernel_version: inspected.kernel_version,
            uptime_seconds: inspected.uptime_seconds,
            cpu_count: inspected.cpu_count,
            load_average: inspected.load_average,
            memory_total_bytes: inspected.memory_total_bytes,
            memory_available_bytes: inspected.memory_available_bytes,
            root_disk_total_bytes: inspected.root_disk_total_bytes,
            root_disk_available_bytes: inspected.root_disk_available_bytes,
        }
    }

    #[must_use]
    pub fn generations(&self) -> GenerationCatalog {
        let generations = nixos_control_nix::inspect_system_generations()
            .into_iter()
            .map(|generation| SystemGeneration {
                generation: generation.generation,
                system_path: generation.system_path.to_string_lossy().into_owned(),
                nixos_version: generation.nixos_version,
                created_unix_seconds: generation.created_unix_seconds,
                current: generation.current,
                active: generation.active,
                bootable: generation.bootable,
            })
            .collect::<Vec<_>>();
        GenerationCatalog {
            total: generations.len(),
            generations,
        }
    }

    /// Returns closure metadata for one installed system generation.
    ///
    /// # Errors
    /// Returns an inspection error when the generation is absent or Nix fails.
    pub async fn generation_detail(&self, number: u64) -> Result<GenerationDetail, InspectError> {
        if let Some(detail) = self.generation_detail_cache.read().await.get(&number) {
            return Ok(detail.clone());
        }
        let detail =
            nixos_control_nix::inspect_generation_detail(self.command_timeout, number).await?;
        let detail = GenerationDetail {
            generation: SystemGeneration {
                generation: detail.generation.generation,
                system_path: detail.generation.system_path.to_string_lossy().into_owned(),
                nixos_version: detail.generation.nixos_version,
                created_unix_seconds: detail.generation.created_unix_seconds,
                current: detail.generation.current,
                active: detail.generation.active,
                bootable: detail.generation.bootable,
            },
            closure_size_bytes: detail.closure_size_bytes,
            store_path_count: detail.store_path_count,
        };
        self.generation_detail_cache
            .write()
            .await
            .insert(number, detail.clone());
        Ok(detail)
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

    #[must_use]
    pub fn managed_configuration_summary(&self) -> ManagedConfigurationSummary {
        let inspected =
            nixos_control_nix::inspect_managed_configuration(&self.configuration_target);
        let mode = match inspected.mode {
            nixos_control_nix::ManagedConfigurationMode::Unmanaged => {
                ManagedConfigurationMode::Unmanaged
            }
            nixos_control_nix::ManagedConfigurationMode::ManualWiring => {
                ManagedConfigurationMode::ManualWiring
            }
            nixos_control_nix::ManagedConfigurationMode::ControlPlaneManaged => {
                ManagedConfigurationMode::ControlPlaneManaged
            }
        };
        ManagedConfigurationSummary {
            repository_root: inspected
                .repository_root
                .map(|path| path.to_string_lossy().into_owned()),
            managed_directory: inspected
                .managed_directory
                .map(|path| path.to_string_lossy().into_owned()),
            host: inspected.host,
            host_module: inspected
                .host_module
                .map(|path| path.to_string_lossy().into_owned()),
            host_module_exists: inspected.host_module_exists,
            metadata_exists: inspected.metadata_exists,
            mode,
            definitions_editable: mode != ManagedConfigurationMode::Unmanaged,
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
                current_value: option.current_value,
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

    /// Evaluates metadata and values for one option without expanding every
    /// option value in the catalog.
    ///
    /// # Errors
    /// Returns an inspection error when the target or option cannot be evaluated.
    pub async fn option_detail(&self, option: &str) -> Result<OptionDetail, InspectError> {
        let fingerprint = source_fingerprint(&self.configuration_target);
        if let Some(fingerprint) = &fingerprint
            && let Some(cached) = self.option_detail_cache.read().await.get(option)
            && &cached.fingerprint == fingerprint
        {
            return Ok(cached.detail.clone());
        }
        let system = nixos_control_nix::inspect_local_system(self.command_timeout).await;
        let evaluated = NixEvaluator::new(self.command_timeout)
            .evaluate_option_detail(&self.configuration_target, &system.hostname, option)
            .await?;
        let detail = OptionDetail {
            name: evaluated.name,
            description: evaluated.description,
            option_type: evaluated.option_type,
            read_only: evaluated.read_only,
            declarations: evaluated.declarations,
            current_value: evaluated.current_value,
            default_value: evaluated.default_value,
            example: evaluated.example,
        };
        if let Some(fingerprint) = fingerprint {
            self.option_detail_cache.write().await.insert(
                option.to_owned(),
                CachedOptionDetail {
                    fingerprint,
                    detail: detail.clone(),
                },
            );
        }
        Ok(detail)
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
                source: package.source,
            })
            .collect();
        let catalog = PackageCatalog {
            host: evaluated.host,
            total: packages.len(),
            packages,
            sources: evaluated.sources,
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
            .filter(|service| service.enabled == Some(true))
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

    /// Returns option metadata belonging to one configured service namespace.
    ///
    /// # Errors
    /// Returns an inspection error when the option graph cannot be evaluated.
    pub async fn service_detail(&self, service: &str) -> Result<ServiceDetail, InspectError> {
        let fingerprint = source_fingerprint(&self.configuration_target);
        if let Some(fingerprint) = &fingerprint
            && let Some(cached) = self.service_detail_cache.read().await.get(service)
            && &cached.fingerprint == fingerprint
        {
            return Ok(cached.detail.clone());
        }
        let system = nixos_control_nix::inspect_local_system(self.command_timeout).await;
        let evaluated = NixEvaluator::new(self.command_timeout)
            .evaluate_service_options(&self.configuration_target, &system.hostname, service)
            .await?;
        let options = evaluated
            .options
            .into_iter()
            .map(|option| OptionSummary {
                name: option.name,
                description: option.description,
                option_type: option.option_type,
                read_only: option.read_only,
                declarations: option.declarations,
                current_value: option.current_value,
            })
            .collect();
        let detail = ServiceDetail {
            name: service.to_owned(),
            options,
            systemd_units: evaluated.systemd_units,
        };
        if let Some(fingerprint) = fingerprint {
            self.service_detail_cache.write().await.insert(
                service.to_owned(),
                CachedServiceDetail {
                    fingerprint,
                    detail: detail.clone(),
                },
            );
        }
        Ok(detail)
    }

    /// Returns runtime systemd service state independently of Nix configuration.
    ///
    /// # Errors
    /// Returns an inspection error when systemd cannot be queried.
    pub async fn runtime_services(&self) -> Result<RuntimeServiceCatalog, InspectError> {
        let services: Vec<_> = nixos_control_nix::inspect_runtime_services(self.command_timeout)
            .await?
            .into_iter()
            .map(|service| RuntimeService {
                unit: service.unit,
                description: service.description,
                load_state: service.load_state,
                active_state: service.active_state,
                sub_state: service.sub_state,
            })
            .collect();
        Ok(RuntimeServiceCatalog {
            total: services.len(),
            failed: services
                .iter()
                .filter(|service| service.active_state == "failed")
                .count(),
            services,
        })
    }
}

fn source_fingerprint(target: &ConfigurationTarget) -> Option<String> {
    let explicit_flake = target.flake_uri.as_deref().map(Path::new);
    let path = explicit_flake
        .filter(|path| path.exists())
        .or(target.path.as_deref());
    let Some(path) = path else {
        return target.flake_uri.as_ref().map(|uri| {
            format!(
                "remote:{uri}:{}",
                target.host.as_deref().unwrap_or_default()
            )
        });
    };
    let root = if path.is_file() { path.parent()? } else { path };
    let mut fingerprint = String::new();
    fingerprint_tree(root, root, &mut fingerprint).ok()?;
    write!(
        fingerprint,
        "host:{};",
        target.host.as_deref().unwrap_or_default()
    )
    .expect("writing to a String cannot fail");
    Some(fingerprint)
}

fn fingerprint_tree(
    root: &Path,
    directory: &Path,
    fingerprint: &mut String,
) -> std::io::Result<()> {
    let mut entries = std::fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let candidate = entry.path();
        let file_name = entry.file_name();
        if candidate.is_dir() {
            if matches!(
                file_name.to_str(),
                Some(".git" | ".direnv" | "target" | "node_modules")
            ) {
                continue;
            }
            fingerprint_tree(root, &candidate, fingerprint)?;
            continue;
        }
        let metadata = std::fs::symlink_metadata(&candidate)?;
        let modified = metadata
            .modified()
            .unwrap_or(std::time::UNIX_EPOCH)
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        write!(
            fingerprint,
            "{}:{modified}:{};",
            candidate.strip_prefix(root).unwrap_or(&candidate).display(),
            metadata.len()
        )
        .expect("writing to a String cannot fail");
    }
    Ok(())
}
