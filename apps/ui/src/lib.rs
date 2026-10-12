use dioxus::prelude::*;
use nixos_control_api_types::{
    ConfigurationKind, ConfigurationSummary, OptionCatalog, PackageCatalog, ServiceCatalog,
    SystemSummary,
};

const STYLES: &str = r"
:root { color-scheme: dark; font-family: Inter, ui-sans-serif, system-ui, sans-serif; background: #09100e; color: #edf7f2; }
* { box-sizing: border-box; }
body { margin: 0; min-height: 100vh; background: radial-gradient(circle at 80% 0%, #173a2e 0, transparent 38rem), #09100e; }
.shell { width: min(1120px, calc(100% - 2rem)); margin: 0 auto; padding: 2rem 0 4rem; }
.topbar { display: flex; align-items: center; justify-content: space-between; gap: 1rem; margin-bottom: 4.5rem; }
.brand { display: flex; align-items: center; gap: .8rem; font-weight: 700; letter-spacing: -.02em; }
.mark { display: grid; place-items: center; width: 2.35rem; aspect-ratio: 1; border: 1px solid #3c715f; border-radius: .7rem; background: #11251e; color: #7ce2b8; }
.status { display: flex; align-items: center; gap: .55rem; color: #9cb2a9; font-size: .875rem; }
.dot { width: .5rem; height: .5rem; border-radius: 50%; background: #55dc9e; box-shadow: 0 0 0 .25rem #55dc9e1d; }
.eyebrow { color: #70d8ad; text-transform: uppercase; letter-spacing: .16em; font-size: .72rem; font-weight: 700; }
h1 { max-width: 780px; margin: .8rem 0 1rem; font-size: clamp(2.5rem, 6vw, 5.5rem); line-height: .98; letter-spacing: -.065em; font-weight: 650; }
.lede { color: #a8bbb3; max-width: 630px; font-size: 1.08rem; line-height: 1.65; margin-bottom: 3rem; }
.grid { display: grid; grid-template-columns: repeat(12, 1fr); gap: 1rem; }
.card { grid-column: span 6; min-height: 15rem; border: 1px solid #28453a; border-radius: 1.15rem; padding: 1.5rem; background: linear-gradient(145deg, #102019e6, #0c1713d9); box-shadow: 0 1.5rem 4rem #0005; }
.card.wide { grid-column: span 12; min-height: auto; }
.options { grid-column: span 12; }
.card-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem; }
.card h2 { font-size: .78rem; text-transform: uppercase; letter-spacing: .12em; color: #8ba198; margin: 0; }
.pill { border: 1px solid #35604f; border-radius: 999px; color: #77d8b0; padding: .28rem .6rem; font-size: .72rem; }
.primary { font-size: 1.8rem; letter-spacing: -.035em; margin-bottom: .35rem; overflow-wrap: anywhere; }
.muted { color: #82968e; font-size: .88rem; }
.facts { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem; margin: 0; }
.facts div { border-left: 1px solid #315044; padding-left: 1rem; min-width: 0; }
dt { color: #82968e; font-size: .76rem; margin-bottom: .45rem; }
dd { margin: 0; overflow-wrap: anywhere; }
code { font-family: ui-monospace, SFMono-Regular, monospace; font-size: .82rem; color: #b7d8ca; }
.search { width: 100%; margin: 0 0 1rem; padding: .85rem 1rem; border: 1px solid #315044; border-radius: .7rem; background: #09120f; color: #edf7f2; font: inherit; outline: none; }
.search:focus { border-color: #63cda2; box-shadow: 0 0 0 .2rem #63cda21f; }
.option-list { display: grid; max-height: 29rem; overflow: auto; border-top: 1px solid #29453a; }
.option { display: grid; grid-template-columns: minmax(12rem, 1fr) minmax(8rem, 2fr); gap: 1.5rem; padding: .9rem .25rem; border-bottom: 1px solid #1f362d; }
.option-name { color: #9ce3c5; }
.option-description { color: #8fa49b; font-size: .84rem; line-height: 1.45; }
.error { padding: 1rem; border: 1px solid #653d3d; border-radius: .75rem; color: #e8b2b2; background: #291616; overflow-wrap: anywhere; }
@media (max-width: 700px) { .shell { padding-top: 1rem; } .topbar { margin-bottom: 3rem; } .card { grid-column: span 12; } .facts { grid-template-columns: 1fr; } }
";

const FILTER_SCRIPT: &str = r"(() => {
  const input = document.getElementById('option-search');
  const list = document.getElementById('option-list');
  const count = document.getElementById('option-count');
  let timer;
  input?.addEventListener('input', () => {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      const response = await fetch('/api/v1/options?q=' + encodeURIComponent(input.value) + '&limit=100');
      if (!response.ok) return;
      const catalog = await response.json();
      count.textContent = catalog.total + ' matches';
      list.replaceChildren(...catalog.options.map(option => {
        const row = document.createElement('div');
        row.className = 'option';
        const name = document.createElement('code');
        name.className = 'option-name';
        name.textContent = option.name;
        const description = document.createElement('div');
        description.className = 'option-description';
        description.textContent = option.description || 'No description available';
        row.append(name, description);
        return row;
      }));
    }, 250);
  });
})();";

const PACKAGE_FILTER_SCRIPT: &str = r"(() => {
  const input = document.getElementById('package-search');
  const list = document.getElementById('package-list');
  const count = document.getElementById('package-count');
  let timer;
  input?.addEventListener('input', () => {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      const response = await fetch('/api/v1/packages?q=' + encodeURIComponent(input.value) + '&limit=100');
      if (!response.ok) return;
      const catalog = await response.json();
      count.textContent = catalog.total + ' matches';
      list.replaceChildren(...catalog.packages.map(packageInfo => {
        const row = document.createElement('div');
        row.className = 'option';
        const name = document.createElement('code');
        name.className = 'option-name';
        name.textContent = packageInfo.name + (packageInfo.version ? ' ' + packageInfo.version : '');
        const description = document.createElement('div');
        description.className = 'option-description';
        description.textContent = packageInfo.description || packageInfo.store_path;
        row.append(name, description);
        return row;
      }));
    }, 250);
  });
})();";

const SERVICE_FILTER_SCRIPT: &str = r"(() => {
  const input = document.getElementById('service-search');
  const list = document.getElementById('service-list');
  const count = document.getElementById('service-count');
  let timer;
  input?.addEventListener('input', () => {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      const response = await fetch('/api/v1/services?q=' + encodeURIComponent(input.value) + '&limit=100');
      if (!response.ok) return;
      const catalog = await response.json();
      count.textContent = catalog.enabled + ' enabled / ' + catalog.total;
      list.replaceChildren(...catalog.services.map(service => {
        const row = document.createElement('div');
        row.className = 'option';
        const name = document.createElement('code');
        name.className = 'option-name';
        name.textContent = service.name;
        const state = document.createElement('div');
        state.className = 'option-description';
        state.textContent = service.enabled === true ? 'Enabled' : service.enabled === false ? 'Disabled' : 'State unavailable';
        row.append(name, state);
        return row;
      }));
    }, 250);
  });
})();";

#[must_use]
pub fn render_dashboard(
    system: SystemSummary,
    configuration: ConfigurationSummary,
    option_catalog: Result<OptionCatalog, String>,
    package_catalog: Result<PackageCatalog, String>,
    service_catalog: Result<ServiceCatalog, String>,
) -> String {
    dioxus_ssr::render_element(rsx! {
        style { dangerous_inner_html: STYLES }
        Dashboard { system, configuration, option_catalog, package_catalog, service_catalog }
    })
}

#[component]
fn Dashboard(
    system: SystemSummary,
    configuration: ConfigurationSummary,
    option_catalog: Result<OptionCatalog, String>,
    package_catalog: Result<PackageCatalog, String>,
    service_catalog: Result<ServiceCatalog, String>,
) -> Element {
    let kind = match configuration.kind {
        ConfigurationKind::Flake => "Flake",
        ConfigurationKind::Traditional => "Traditional",
        ConfigurationKind::Unknown => "Not detected",
    };
    let source = configuration
        .source
        .as_deref()
        .unwrap_or("No local source selected");
    let version = system
        .nixos_version
        .as_deref()
        .unwrap_or("Not a NixOS host");
    let current_system = system.current_system.as_deref().unwrap_or("Unavailable");
    let host = configuration.host.as_deref().unwrap_or("Automatic");
    let flake_uri = configuration.flake_uri.as_deref().unwrap_or("None");
    rsx! {
        main { class: "shell",
            header { class: "topbar",
                div { class: "brand", span { class: "mark", "N" } "NixOS Control Plane" }
                div { class: "status", span { class: "dot" } "Daemon connected" }
            }
            section {
                div { class: "eyebrow", "Local host overview" }
                h1 { "Declarative systems, clearly understood." }
                p { class: "lede", "A read-only view of the active system and its configuration source. Every future change will remain reviewable, explicit, and reversible." }
            }
            section { class: "grid",
                article { class: "card",
                    div { class: "card-head", h2 { "System" } span { class: "pill", "Active" } }
                    div { class: "primary", "{system.hostname}" }
                    div { class: "muted", "NixOS {version}" }
                }
                article { class: "card",
                    div { class: "card-head", h2 { "Configuration" } span { class: "pill", "{kind}" } }
                    div { class: "primary", "{source}" }
                    div { class: "muted", if configuration.has_lock_file { "Lock file detected" } else { "No lock file detected" } }
                }
                article { class: "card wide",
                    dl { class: "facts",
                        div { dt { "Flake URI" } dd { code { "{flake_uri}" } } }
                        div { dt { "Selected host" } dd { "{host}" } }
                        div { dt { "Current system" } dd { code { "{current_system}" } } }
                    }
                }
                article { class: "card options",
                    div { class: "card-head", h2 { "NixOS options" }
                        if let Ok(catalog) = &option_catalog { span { class: "pill", id: "option-count", "{catalog.total} evaluated" } }
                    }
                    match option_catalog {
                        Ok(catalog) => rsx! {
                            input { class: "search", id: "option-search", r#type: "search", placeholder: "Filter options by name…", aria_label: "Filter NixOS options" }
                            div { class: "option-list", id: "option-list",
                                for option in catalog.options {
                                    {
                                        let search_name = option.name.to_lowercase();
                                        let description = option.description.as_deref().unwrap_or("No description available");
                                        rsx! {
                                            div { class: "option", "data-option-name": "{search_name}",
                                                code { class: "option-name", "{option.name}" }
                                                div { class: "option-description", "{description}" }
                                            }
                                        }
                                    }
                                }
                            }
                            script { dangerous_inner_html: FILTER_SCRIPT }
                        },
                        Err(message) => rsx! { div { class: "error", "Option evaluation unavailable: {message}" } },
                    }
                }
                article { class: "card options",
                    div { class: "card-head", h2 { "System packages" }
                        if let Ok(catalog) = &package_catalog { span { class: "pill", id: "package-count", "{catalog.total} effective" } }
                    }
                    match package_catalog {
                        Ok(catalog) => rsx! {
                            input { class: "search", id: "package-search", r#type: "search", placeholder: "Filter environment.systemPackages…", aria_label: "Filter system packages" }
                            div { class: "option-list", id: "package-list",
                                for package in catalog.packages {
                                    {
                                        let label = package.version.as_ref().map_or_else(|| package.name.clone(), |version| format!("{} {version}", package.name));
                                        let description = package.description.as_deref().unwrap_or(&package.store_path);
                                        rsx! {
                                            div { class: "option",
                                                code { class: "option-name", "{label}" }
                                                div { class: "option-description", "{description}" }
                                            }
                                        }
                                    }
                                }
                            }
                            script { dangerous_inner_html: PACKAGE_FILTER_SCRIPT }
                        },
                        Err(message) => rsx! { div { class: "error", "Package evaluation unavailable: {message}" } },
                    }
                    p { class: "muted", "Effective merged environment.systemPackages; NixOS defaults may be included, while dependency closure entries are excluded." }
                }
                article { class: "card options",
                    div { class: "card-head", h2 { "Configured services" }
                        if let Ok(catalog) = &service_catalog { span { class: "pill", id: "service-count", "{catalog.enabled} enabled / {catalog.total}" } }
                    }
                    match service_catalog {
                        Ok(catalog) => rsx! {
                            input { class: "search", id: "service-search", r#type: "search", placeholder: "Filter service enable options…", aria_label: "Filter configured services" }
                            div { class: "option-list", id: "service-list",
                                for service in catalog.services {
                                    div { class: "option",
                                        code { class: "option-name", "{service.name}" }
                                        div { class: "option-description",
                                            if service.enabled == Some(true) { "Enabled" } else if service.enabled == Some(false) { "Disabled" } else { "State unavailable" }
                                        }
                                    }
                                }
                            }
                            script { dangerous_inner_html: SERVICE_FILTER_SCRIPT }
                        },
                        Err(message) => rsx! { div { class: "error", "Service discovery unavailable: {message}" } },
                    }
                    p { class: "muted", "Configuration state only; live systemd unit status will be shown separately." }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::render_dashboard;
    use nixos_control_api_types::{ConfigurationKind, ConfigurationSummary, SystemSummary};

    #[test]
    fn renders_system_and_flake_details() {
        let html = render_dashboard(
            SystemSummary {
                hostname: "snowflake".to_owned(),
                nixos_version: Some("26.11".to_owned()),
                current_system: Some("/nix/store/example-system".to_owned()),
            },
            ConfigurationSummary {
                kind: ConfigurationKind::Flake,
                source: Some("/etc/nixos/flake.nix".to_owned()),
                flake_uri: Some("/etc/nixos".to_owned()),
                host: Some("snowflake".to_owned()),
                has_lock_file: true,
            },
            Ok(nixos_control_api_types::OptionCatalog {
                host: "snowflake".to_owned(),
                total: 1,
                options: vec![nixos_control_api_types::OptionSummary {
                    name: "services.openssh.enable".to_owned(),
                    description: Some("Enable OpenSSH".to_owned()),
                    option_type: Some("boolean".to_owned()),
                    read_only: false,
                    declarations: Vec::new(),
                }],
            }),
            Ok(nixos_control_api_types::PackageCatalog {
                host: "snowflake".to_owned(),
                total: 1,
                packages: vec![nixos_control_api_types::SystemPackage {
                    name: "curl".to_owned(),
                    version: Some("8.0".to_owned()),
                    description: Some("Command line HTTP client".to_owned()),
                    store_path: "/nix/store/example-curl".to_owned(),
                }],
            }),
            Ok(nixos_control_api_types::ServiceCatalog {
                host: "snowflake".to_owned(),
                total: 1,
                enabled: 1,
                services: vec![nixos_control_api_types::ServiceSummary {
                    name: "openssh".to_owned(),
                    enabled: Some(true),
                    description: Some("OpenSSH daemon".to_owned()),
                    declarations: Vec::new(),
                }],
            }),
        );
        assert!(html.contains("snowflake"));
        assert!(html.contains("/etc/nixos/flake.nix"));
        assert!(html.contains("Lock file detected"));
        assert!(html.contains("services.openssh.enable"));
        assert!(html.contains("curl 8.0"));
        assert!(html.contains("openssh"));
    }
}
