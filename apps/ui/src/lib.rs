use dioxus::prelude::*;
use nixos_control_api_types::{
    ConfigurationKind, ConfigurationSummary, FlakeInputCatalog, GenerationCatalog,
    GenerationComparison, GenerationDetail, ManagedConfigurationMode, ManagedConfigurationSummary,
    OptionCatalog, OptionDetail, PackageCatalog, RuntimeServiceCatalog, ServiceCatalog,
    ServiceDetail, SystemSummary,
};

const STYLES: &str = r"
:root { color-scheme: dark; font-family: Inter, ui-sans-serif, system-ui, sans-serif; background: #09100e; color: #edf7f2; }
* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body { margin: 0; min-height: 100vh; overflow-x: hidden; background: #09100e; }
button, input { touch-action: manipulation; -webkit-tap-highlight-color: transparent; }
.skip-link { position: fixed; left: 1rem; top: 1rem; z-index: 10; transform: translateY(-5rem); padding: .65rem .85rem; border-radius: .5rem; background: #edf7f2; color: #09100e; }
.skip-link:focus { transform: translateY(0); }
.shell { width: min(1180px, calc(100% - 2rem)); margin: 0 auto; padding: 1.25rem 0 4rem; }
.topbar { display: flex; align-items: center; gap: 1.25rem; padding-bottom: 1rem; border-bottom: 1px solid #20372e; }
.brand { display: flex; align-items: center; gap: .8rem; color: inherit; font-weight: 700; letter-spacing: -.02em; text-decoration: none; }
.mark { display: grid; place-items: center; width: 2.35rem; aspect-ratio: 1; border: 1px solid #3c715f; border-radius: .7rem; background: #11251e; color: #7ce2b8; }
.nav { display: flex; align-items: center; gap: .2rem; margin-right: auto; }
.nav a { padding: .5rem .65rem; border-radius: .45rem; color: #9fb3aa; font-size: .84rem; text-decoration: none; }
.nav a:hover, .nav a.active { color: #edf7f2; background: #13231d; }
.status { display: flex; align-items: center; gap: .55rem; color: #9cb2a9; font-size: .82rem; }
.dot { width: .5rem; height: .5rem; border-radius: 50%; background: #55dc9e; box-shadow: 0 0 0 .25rem #55dc9e1d; }
.page-head { display: flex; align-items: end; justify-content: space-between; gap: 2rem; padding: 2.25rem 0 1.25rem; }
h1 { margin: 0; font-size: clamp(1.8rem, 4vw, 2.5rem); line-height: 1.1; letter-spacing: -.045em; font-weight: 650; text-wrap: balance; }
.lede { color: #8fa49b; margin: .55rem 0 0; font-size: .9rem; overflow-wrap: anywhere; }
.grid { display: grid; grid-template-columns: repeat(12, 1fr); gap: 1rem; }
.card { grid-column: span 6; min-width: 0; min-height: 12rem; border: 1px solid #28453a; border-radius: .8rem; padding: 1.25rem; background: #0d1914; }
.page-section { display: none; }
.page-overview .overview-section, .page-configuration .configuration-section, .page-packages .packages-section, .page-services .services-section, .page-runtime .runtime-section, .page-generations .generations-section { display: block; }
.card.wide { grid-column: span 12; min-height: auto; }
.options { grid-column: span 12; }
.card-head { display: flex; justify-content: space-between; align-items: center; gap: 1rem; margin-bottom: 1.25rem; }
.card h2 { font-size: .95rem; color: #d8e7e0; margin: 0; scroll-margin-top: 1rem; }
.pill { border: 1px solid #35604f; border-radius: 999px; color: #77d8b0; padding: .28rem .6rem; font-size: .72rem; }
.primary { font-size: 1.45rem; letter-spacing: -.025em; margin-bottom: .35rem; overflow-wrap: anywhere; }
.muted { color: #82968e; font-size: .88rem; }
.facts { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem; margin: 0; }
.facts div { border-left: 1px solid #315044; padding-left: 1rem; min-width: 0; }
dt { color: #82968e; font-size: .76rem; margin-bottom: .45rem; }
dd { margin: 0; overflow-wrap: anywhere; }
code { font-family: ui-monospace, SFMono-Regular, monospace; font-size: .82rem; color: #b7d8ca; }
.visually-hidden { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0; }
.search { width: 100%; margin: 0 0 1rem; padding: .75rem .85rem; border: 1px solid #315044; border-radius: .55rem; background: #09120f; color: #edf7f2; font: inherit; }
.search:focus-visible, .tag:focus-visible, .nav a:focus-visible { outline: 2px solid #78ddb5; outline-offset: 2px; }
.option-list { display: grid; gap: .5rem; max-height: 36rem; overflow: auto; }
.option { border: 1px solid #263f35; border-radius: .55rem; background: #0a1511; content-visibility: auto; contain-intrinsic-size: auto 3.5rem; }
.option[open] { border-color: #3a6755; background: #0d1b16; }
.entry-summary { position: relative; display: grid; grid-template-columns: minmax(12rem, 1fr) minmax(8rem, 2fr); gap: 1.5rem; align-items: center; padding: .85rem 2.5rem .85rem .85rem; cursor: pointer; list-style: none; }
.entry-summary::-webkit-details-marker { display: none; }
.entry-summary::after { content: '+'; position: absolute; right: 1rem; color: #79c9a7; font-size: 1.1rem; }
.option[open] > .entry-summary::after { content: '−'; }
.entry-summary:focus-visible { outline: 2px solid #78ddb5; outline-offset: -2px; border-radius: .5rem; }
.entry-body { padding: .85rem; border-top: 1px solid #263f35; color: #9db0a8; font-size: .86rem; line-height: 1.5; overflow-wrap: anywhere; }
.option-name { color: #9ce3c5; }
.option-description { color: #8fa49b; font-size: .84rem; line-height: 1.45; }
.detail-link { display: inline-block; margin-top: .7rem; color: #8be3bd; }
.package-row { grid-template-columns: minmax(10rem, 1fr) minmax(7rem, .55fr); }
.tags { display: flex; flex-wrap: wrap; gap: .5rem; margin-bottom: 1rem; }
.tag { min-height: 2.25rem; border: 1px solid #315044; border-radius: 999px; padding: .38rem .7rem; background: #0b1712; color: #a8bbb3; cursor: pointer; }
.tag.active, .tag:hover { border-color: #63cda2; color: #8be3bd; background: #13271f; }
.error { padding: 1rem; border: 1px solid #653d3d; border-radius: .75rem; color: #e8b2b2; background: #291616; overflow-wrap: anywhere; }
@media (max-width: 760px) { .shell { padding-top: .75rem; } .topbar { flex-wrap: wrap; } .nav { order: 3; width: 100%; overflow-x: auto; } .status { margin-left: auto; } .page-head { padding-top: 1.5rem; } .card { grid-column: span 12; } .facts { grid-template-columns: 1fr; } .entry-summary, .package-row { grid-template-columns: 1fr; gap: .35rem; } }
@media (prefers-reduced-motion: reduce) { html { scroll-behavior: auto; } }
@media (pointer: fine) { html { overscroll-behavior: none; } }
";

const FILTER_SCRIPT: &str = r"(() => {
  const input = document.getElementById('option-search');
  const list = document.getElementById('option-list');
  const count = document.getElementById('option-count');
  let timer;
  input.value = new URL(window.location).searchParams.get('option_q') || '';
  input?.addEventListener('input', () => {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      const response = await fetch('/api/v1/options?q=' + encodeURIComponent(input.value) + '&limit=100');
      if (!response.ok) return;
      const catalog = await response.json();
      count.textContent = catalog.total + ' matches';
      const url = new URL(window.location);
      input.value ? url.searchParams.set('option_q', input.value) : url.searchParams.delete('option_q');
      history.replaceState(null, '', url);
      list.replaceChildren(...catalog.options.map(option => {
        const row = document.createElement('details');
        row.className = 'option';
        const summary = document.createElement('summary');
        summary.className = 'entry-summary';
        const name = document.createElement('code');
        name.className = 'option-name';
        name.textContent = option.name;
        const type = document.createElement('span');
        type.className = 'option-description';
        type.textContent = option.option_type || 'Unknown type';
        const description = document.createElement('div');
        description.className = 'entry-body';
        description.textContent = option.description || 'No description available';
        const link = document.createElement('a');
        link.className = 'detail-link';
        link.href = '/configuration/options/' + encodeURIComponent(option.name);
        link.textContent = 'View Option Details';
        description.append(document.createElement('br'), link);
        summary.append(name, type);
        row.append(summary, description);
        return row;
      }));
    }, 250);
  });
  if (input.value) input.dispatchEvent(new Event('input'));
})();";

const PACKAGE_FILTER_SCRIPT: &str = r"(() => {
  const input = document.getElementById('package-search');
  const list = document.getElementById('package-list');
  const count = document.getElementById('package-count');
  const tags = document.querySelectorAll('[data-package-source]');
  const initialParams = new URL(window.location).searchParams;
  let selectedSource = initialParams.get('package_source') || '';
  let timer;
  input.value = initialParams.get('package_q') || '';
  const load = () => {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      const response = await fetch('/api/v1/packages?q=' + encodeURIComponent(input.value) + '&source=' + encodeURIComponent(selectedSource) + '&limit=100');
      if (!response.ok) return;
      const catalog = await response.json();
      count.textContent = catalog.total + ' matches';
      const url = new URL(window.location);
      input.value ? url.searchParams.set('package_q', input.value) : url.searchParams.delete('package_q');
      selectedSource ? url.searchParams.set('package_source', selectedSource) : url.searchParams.delete('package_source');
      history.replaceState(null, '', url);
      list.replaceChildren(...catalog.packages.map(packageInfo => {
        const row = document.createElement('details');
        row.className = 'option';
        const summary = document.createElement('summary');
        summary.className = 'entry-summary package-row';
        const name = document.createElement('code');
        name.className = 'option-name';
        name.textContent = packageInfo.name + (packageInfo.version ? ' ' + packageInfo.version : '');
        const source = document.createElement('code');
        source.className = 'option-description';
        source.textContent = packageInfo.source || 'unknown';
        const description = document.createElement('div');
        description.className = 'entry-body';
        description.textContent = packageInfo.description || packageInfo.store_path;
        summary.append(name, source);
        row.append(summary, description);
        return row;
      }));
    }, 250);
  };
  input?.addEventListener('input', load);
  tags.forEach(tag => tag.addEventListener('click', () => {
    selectedSource = tag.dataset.packageSource;
    tags.forEach(candidate => candidate.classList.toggle('active', candidate === tag));
    tags.forEach(candidate => candidate.setAttribute('aria-pressed', String(candidate === tag)));
    load();
  }));
  const selectedTag = Array.from(tags).find(tag => tag.dataset.packageSource === selectedSource) || tags[0];
  tags.forEach(tag => {
    tag.classList.toggle('active', tag === selectedTag);
    tag.setAttribute('aria-pressed', String(tag === selectedTag));
  });
  selectedSource = selectedTag?.dataset.packageSource || '';
  if (input.value || selectedSource) load();
})();";

const SERVICE_FILTER_SCRIPT: &str = r"(() => {
  const input = document.getElementById('service-search');
  const list = document.getElementById('service-list');
  const count = document.getElementById('service-count');
  let timer;
  input.value = new URL(window.location).searchParams.get('service_q') || '';
  input?.addEventListener('input', () => {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      const response = await fetch('/api/v1/services?q=' + encodeURIComponent(input.value) + '&limit=100');
      if (!response.ok) return;
      const catalog = await response.json();
      count.textContent = catalog.enabled + ' enabled / ' + catalog.total;
      const url = new URL(window.location);
      input.value ? url.searchParams.set('service_q', input.value) : url.searchParams.delete('service_q');
      history.replaceState(null, '', url);
      list.replaceChildren(...catalog.services.map(service => {
        const row = document.createElement('details');
        row.className = 'option';
        const summary = document.createElement('summary');
        summary.className = 'entry-summary';
        const name = document.createElement('code');
        name.className = 'option-name';
        name.textContent = service.name;
        const state = document.createElement('div');
        state.className = 'option-description';
        state.textContent = service.enabled === true ? 'Enabled' : service.enabled === false ? 'Disabled' : 'State unavailable';
        const body = document.createElement('div');
        body.className = 'entry-body';
        body.textContent = service.description || 'No description available';
        const link = document.createElement('a');
        link.className = 'detail-link';
        link.href = '/services/' + encodeURIComponent(service.name);
        link.textContent = 'View Service Details';
        body.append(document.createElement('br'), link);
        summary.append(name, state);
        row.append(summary, body);
        return row;
      }));
    }, 250);
  });
  if (input.value) input.dispatchEvent(new Event('input'));
})();";

const RUNTIME_FILTER_SCRIPT: &str = r"(() => {
  const input = document.getElementById('runtime-search');
  const list = document.getElementById('runtime-list');
  const count = document.getElementById('runtime-count');
  let timer;
  input.value = new URL(window.location).searchParams.get('runtime_q') || '';
  input?.addEventListener('input', () => {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      const response = await fetch('/api/v1/runtime/services?q=' + encodeURIComponent(input.value) + '&limit=100');
      if (!response.ok) return;
      const catalog = await response.json();
      count.textContent = catalog.failed + ' failed / ' + catalog.total;
      const url = new URL(window.location);
      input.value ? url.searchParams.set('runtime_q', input.value) : url.searchParams.delete('runtime_q');
      history.replaceState(null, '', url);
      list.replaceChildren(...catalog.services.map(service => {
        const row = document.createElement('details');
        row.className = 'option';
        const summary = document.createElement('summary');
        summary.className = 'entry-summary';
        const name = document.createElement('code');
        name.className = 'option-name';
        name.textContent = service.unit;
        const state = document.createElement('div');
        state.className = 'option-description';
        state.textContent = service.active_state + ' / ' + service.sub_state;
        const body = document.createElement('div');
        body.className = 'entry-body';
        body.textContent = service.description + ' · load state: ' + service.load_state;
        summary.append(name, state);
        row.append(summary, body);
        return row;
      }));
    }, 250);
  });
  if (input.value) input.dispatchEvent(new Event('input'));
})();";

const GENERATION_TIME_SCRIPT: &str = r"document.querySelectorAll('[data-unix-seconds]').forEach(element => {
  const seconds = Number(element.dataset.unixSeconds);
  if (Number.isFinite(seconds)) element.textContent = new Date(seconds * 1000).toLocaleString();
});";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardPage {
    Overview,
    Configuration,
    Packages,
    Services,
    Runtime,
    Generations,
}

impl DashboardPage {
    fn class_name(self) -> &'static str {
        match self {
            Self::Overview => "page-overview",
            Self::Configuration => "page-configuration",
            Self::Packages => "page-packages",
            Self::Services => "page-services",
            Self::Runtime => "page-runtime",
            Self::Generations => "page-generations",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Overview => "System Overview",
            Self::Configuration => "Configuration",
            Self::Packages => "System Packages",
            Self::Services => "Configured Services",
            Self::Runtime => "Runtime Services",
            Self::Generations => "System Generations",
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    let tenths = bytes.saturating_mul(10) / GIB;
    format!("{}.{:01} GiB", tenths / 10, tenths % 10)
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    if days > 0 {
        format!("{days}d {hours}h")
    } else {
        format!("{hours}h {}m", (seconds % 3_600) / 60)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardData {
    pub system: SystemSummary,
    pub configuration: ConfigurationSummary,
    pub managed_configuration: ManagedConfigurationSummary,
    pub option_catalog: Result<OptionCatalog, String>,
    pub package_catalog: Result<PackageCatalog, String>,
    pub service_catalog: Result<ServiceCatalog, String>,
    pub runtime_services: Result<RuntimeServiceCatalog, String>,
    pub generations: GenerationCatalog,
}

#[must_use]
pub fn render_dashboard(page: DashboardPage, data: DashboardData) -> String {
    dioxus_ssr::render_element(rsx! {
        style { dangerous_inner_html: STYLES }
        Dashboard { page, data }
    })
}

#[must_use]
pub fn render_flake_inputs(
    system: &SystemSummary,
    configuration: &ConfigurationSummary,
    catalog: Result<FlakeInputCatalog, String>,
) -> String {
    let flake_uri = configuration.flake_uri.as_deref().unwrap_or("None");
    dioxus_ssr::render_element(rsx! {
        style { dangerous_inner_html: STYLES }
        a { class: "skip-link", href: "#main-content", "Skip to Main Content" }
        main { class: "shell", id: "main-content",
            header { class: "topbar",
                a { class: "brand", href: "/", span { class: "mark", "N" } "NixOS Control Plane" }
                nav { class: "nav", aria_label: "Dashboard sections",
                    a { href: "/configuration", "Configuration" }
                    a { href: "/packages", "Packages" }
                    a { href: "/services", "Services" }
                    a { href: "/runtime", "Runtime" }
                    a { href: "/generations", "Generations" }
                    a { class: "active", "aria-current": "page", href: "/inputs", "Inputs" }
                }
                div { class: "status", span { class: "dot" } "{system.hostname}" }
            }
            section { class: "page-head", div { h1 { "Flake Inputs" } p { class: "lede", code { "{flake_uri}" } } } }
            section { class: "grid",
                article { class: "card options", style: "display: block",
                    match catalog {
                        Ok(catalog) => rsx! {
                            div { class: "card-head", h2 { "Pinned Inputs" } span { class: "pill", "{catalog.direct} direct / {catalog.total} nodes" } }
                            p { class: "muted", "Lock file: " code { "{catalog.lock_file}" } }
                            div { class: "option-list",
                                for input in catalog.inputs {
                                    {
                                        let origin = match (&input.owner, &input.repo, &input.url) {
                                            (Some(owner), Some(repo), _) => format!("{owner}/{repo}"),
                                            (_, _, Some(url)) => url.clone(),
                                            _ => "Origin unavailable".to_owned(),
                                        };
                                        let revision = input.revision.as_deref().or(input.reference.as_deref()).unwrap_or("Unpinned revision");
                                        let input_type = input.input_type.as_deref().unwrap_or("unknown");
                                        rsx! { details { class: "option",
                                            summary { class: "entry-summary", code { class: "option-name", "{input.node}" } div { class: "option-description", if input.direct { "Direct · {origin}" } else { "Transitive · {origin}" } } }
                                            div { class: "entry-body",
                                                dl { class: "facts",
                                                    div { dt { "Type" } dd { "{input_type}" } }
                                                    div { dt { "Revision" } dd { code { "{revision}" } } }
                                                    div { dt { "Last modified" } dd { if let Some(seconds) = input.last_modified { span { "data-unix-seconds": "{seconds}", "{seconds}" } } else { "Unavailable" } } }
                                                }
                                                if let Some(hash) = input.nar_hash { p { class: "muted", "Content hash" } code { "{hash}" } }
                                            }
                                        } }
                                    }
                                }
                            }
                            script { dangerous_inner_html: GENERATION_TIME_SCRIPT }
                        },
                        Err(message) => rsx! { div { class: "error", "Flake inputs unavailable: {message}" } },
                    }
                }
            }
        }
    })
}

#[must_use]
pub fn render_service_detail(
    system: SystemSummary,
    configuration: ConfigurationSummary,
    detail: Result<ServiceDetail, String>,
) -> String {
    dioxus_ssr::render_element(rsx! {
        style { dangerous_inner_html: STYLES }
        ServiceDetailPage { system, configuration, detail }
    })
}

#[must_use]
pub fn render_option_detail(
    system: SystemSummary,
    configuration: ConfigurationSummary,
    detail: Result<OptionDetail, String>,
) -> String {
    dioxus_ssr::render_element(rsx! {
        style { dangerous_inner_html: STYLES }
        OptionDetailPage { system, configuration, detail }
    })
}

#[must_use]
pub fn render_generation_detail(
    system: SystemSummary,
    detail: Result<GenerationDetail, String>,
) -> String {
    dioxus_ssr::render_element(rsx! {
        style { dangerous_inner_html: STYLES }
        GenerationDetailPage { system, detail }
    })
}

#[must_use]
pub fn render_generation_comparison(
    system: SystemSummary,
    comparison: Result<GenerationComparison, String>,
) -> String {
    dioxus_ssr::render_element(rsx! {
        style { dangerous_inner_html: STYLES }
        GenerationComparisonPage { system, comparison }
    })
}

#[component]
fn GenerationComparisonPage(
    system: SystemSummary,
    comparison: Result<GenerationComparison, String>,
) -> Element {
    rsx! {
        a { class: "skip-link", href: "#main-content", "Skip to Main Content" }
        main { class: "shell", id: "main-content",
            header { class: "topbar",
                a { class: "brand", href: "/", span { class: "mark", "N" } "NixOS Control Plane" }
                nav { class: "nav", aria_label: "Dashboard sections",
                    a { href: "/configuration", "Configuration" }
                    a { href: "/packages", "Packages" }
                    a { href: "/services", "Services" }
                    a { href: "/runtime", "Runtime" }
                    a { class: "active", "aria-current": "page", href: "/generations", "Generations" }
                }
                div { class: "status", span { class: "dot" } "{system.hostname}" }
            }
            section { class: "page-head", div {
                a { class: "muted", href: "/generations", "← System Generations" }
                h1 { "Generation Comparison" }
                p { class: "lede", "Exact recursive store closure delta" }
            } }
            match comparison {
                Ok(comparison) => {
                    let added_size = format_bytes(comparison.added_size_bytes);
                    let removed_size = format_bytes(comparison.removed_size_bytes);
                    rsx! { section { class: "grid",
                        article { class: "card wide",
                            div { class: "card-head", h2 { "Generation {comparison.from_generation} → {comparison.to_generation}" } span { class: "pill", "Closure diff" } }
                            dl { class: "facts",
                                div { dt { "Added" } dd { "{comparison.added.len()} paths · {added_size}" } }
                                div { dt { "Removed" } dd { "{comparison.removed.len()} paths · {removed_size}" } }
                                div { dt { "Net path change" } dd { "{comparison.added.len() as i64 - comparison.removed.len() as i64}" } }
                            }
                        }
                        article { class: "card options",
                            div { class: "card-head", h2 { "Added Store Paths" } span { class: "pill", "{comparison.added.len()}" } }
                            div { class: "option-list",
                                for path in comparison.added.iter().take(500) {
                                    details { class: "option", summary { class: "entry-summary", code { class: "option-name", "{path.name}" } div { class: "option-description", "{format_bytes(path.nar_size_bytes)}" } } div { class: "entry-body", code { "{path.path}" } } }
                                }
                            }
                        }
                        article { class: "card options",
                            div { class: "card-head", h2 { "Removed Store Paths" } span { class: "pill", "{comparison.removed.len()}" } }
                            div { class: "option-list",
                                for path in comparison.removed.iter().take(500) {
                                    details { class: "option", summary { class: "entry-summary", code { class: "option-name", "{path.name}" } div { class: "option-description", "{format_bytes(path.nar_size_bytes)}" } } div { class: "entry-body", code { "{path.path}" } } }
                                }
                            }
                        }
                    } }
                },
                Err(message) => rsx! { section { class: "error", "Generation comparison unavailable: {message}" } },
            }
        }
    }
}

#[component]
fn GenerationDetailPage(
    system: SystemSummary,
    detail: Result<GenerationDetail, String>,
) -> Element {
    let title = detail.as_ref().map_or_else(
        |_| "Generation Details".to_owned(),
        |detail| format!("Generation {}", detail.generation.generation),
    );
    rsx! {
        a { class: "skip-link", href: "#main-content", "Skip to Main Content" }
        main { class: "shell", id: "main-content",
            header { class: "topbar",
                a { class: "brand", href: "/", span { class: "mark", "N" } "NixOS Control Plane" }
                nav { class: "nav", aria_label: "Dashboard sections",
                    a { href: "/configuration", "Configuration" }
                    a { href: "/packages", "Packages" }
                    a { href: "/services", "Services" }
                    a { href: "/runtime", "Runtime" }
                    a { class: "active", "aria-current": "page", href: "/generations", "Generations" }
                }
                div { class: "status", span { class: "dot" } "{system.hostname}" }
            }
            section { class: "page-head",
                div {
                    a { class: "muted", href: "/generations", "← System Generations" }
                    h1 { "{title}" }
                    p { class: "lede", "Immutable closure inspection" }
                }
            }
            section { class: "grid",
                article { class: "card wide",
                    match detail {
                        Ok(detail) => {
                            let version = detail.generation.nixos_version.as_deref().unwrap_or("Unknown NixOS version");
                            let size = format_bytes(detail.closure_size_bytes);
                            rsx! {
                                div { class: "card-head", h2 { "System Closure" } span { class: "pill", "{version}" } }
                                div { class: "tags", aria_label: "Generation status",
                                    if detail.generation.active { span { class: "tag active", "Active" } }
                                    if detail.generation.current { span { class: "tag active", "Current profile" } }
                                    if detail.generation.bootable { span { class: "tag", "Bootable" } }
                                }
                                dl { class: "facts",
                                    div { dt { "Closure size" } dd { "{size}" } }
                                    div { dt { "Store paths" } dd { "{detail.store_path_count}" } }
                                    div { dt { "Generation" } dd { "{detail.generation.generation}" } }
                                }
                                p { class: "muted", "Resolved system path" }
                                code { "{detail.generation.system_path}" }
                            }
                        },
                        Err(message) => rsx! { div { class: "error", "Generation details unavailable: {message}" } },
                    }
                }
            }
        }
    }
}

#[component]
fn OptionDetailPage(
    system: SystemSummary,
    configuration: ConfigurationSummary,
    detail: Result<OptionDetail, String>,
) -> Element {
    let host = configuration.host.as_deref().unwrap_or("Automatic");
    let flake_uri = configuration.flake_uri.as_deref().unwrap_or("None");
    let title = detail
        .as_ref()
        .map_or("Option Details", |option| option.name.as_str());
    rsx! {
        a { class: "skip-link", href: "#main-content", "Skip to Main Content" }
        main { class: "shell", id: "main-content",
            header { class: "topbar",
                a { class: "brand", href: "/", span { class: "mark", "N" } "NixOS Control Plane" }
                nav { class: "nav", aria_label: "Dashboard sections",
                    a { class: "active", "aria-current": "page", href: "/configuration", "Configuration" }
                    a { href: "/packages", "Packages" }
                    a { href: "/services", "Services" }
                    a { href: "/runtime", "Runtime" }
                    a { href: "/generations", "Generations" }
                }
                div { class: "status", span { class: "dot" } "{system.hostname}" }
            }
            section { class: "page-head",
                div {
                    a { class: "muted", href: "/configuration", "← NixOS Options" }
                    h1 { "{title}" }
                    p { class: "lede", code { "{host}" } " · " code { "{flake_uri}" } }
                }
            }
            section { class: "grid",
                article { class: "card wide",
                    match detail {
                        Ok(option) => {
                            let option_type = option.option_type.as_deref().unwrap_or("Unknown type");
                            let description = option.description.as_deref().unwrap_or("No description available");
                            let current = option.current_value.as_deref().unwrap_or("Value unavailable");
                            let default = option.default_value.as_deref().unwrap_or("No default documented");
                            let example = option.example.as_deref().unwrap_or("No example documented");
                            rsx! {
                                div { class: "card-head", h2 { "Evaluated Option" } span { class: "pill", if option.read_only { "Read only" } else { "Configurable" } } }
                                p { "{description}" }
                                dl { class: "facts",
                                    div { dt { "Type" } dd { code { "{option_type}" } } }
                                    div { dt { "Effective value" } dd { code { "{current}" } } }
                                    div { dt { "Default" } dd { code { "{default}" } } }
                                }
                                details { class: "option", style: "margin-top: 1rem",
                                    summary { class: "entry-summary", code { class: "option-name", "Example" } div { class: "option-description", "Module documentation" } }
                                    div { class: "entry-body", code { "{example}" } }
                                }
                                details { class: "option", style: "margin-top: .5rem",
                                    summary { class: "entry-summary", code { class: "option-name", "Declarations" } div { class: "option-description", "{option.declarations.len()} source files" } }
                                    div { class: "entry-body",
                                        if option.declarations.is_empty() {
                                            "Declaration locations unavailable"
                                        } else {
                                            for declaration in option.declarations { div { code { "{declaration}" } } }
                                        }
                                    }
                                }
                            }
                        },
                        Err(message) => rsx! { div { class: "error", "Option details unavailable: {message}" } },
                    }
                }
            }
        }
    }
}

#[component]
fn ServiceDetailPage(
    system: SystemSummary,
    configuration: ConfigurationSummary,
    detail: Result<ServiceDetail, String>,
) -> Element {
    let host = configuration.host.as_deref().unwrap_or("Automatic");
    let flake_uri = configuration.flake_uri.as_deref().unwrap_or("None");
    let title = detail
        .as_ref()
        .map_or("Service Details", |service| service.name.as_str());
    rsx! {
        a { class: "skip-link", href: "#main-content", "Skip to Main Content" }
        main { class: "shell", id: "main-content",
            header { class: "topbar",
                a { class: "brand", href: "/", span { class: "mark", "N" } "NixOS Control Plane" }
                nav { class: "nav", aria_label: "Dashboard sections",
                    a { href: "/configuration", "Configuration" }
                    a { href: "/packages", "Packages" }
                    a { class: "active", "aria-current": "page", href: "/services", "Services" }
                    a { href: "/runtime", "Runtime" }
                    a { href: "/generations", "Generations" }
                }
                div { class: "status", span { class: "dot" } "{system.hostname}" }
            }
            section { class: "page-head",
                div {
                    a { class: "muted", href: "/services", "← Configured Services" }
                    h1 { "{title}" }
                    p { class: "lede", code { "{host}" } " · " code { "{flake_uri}" } }
                }
            }
            section { class: "grid",
                article { class: "card options", style: "display: block",
                    match detail {
                        Ok(service) => rsx! {
                            div { class: "card-head", h2 { "Service Options" } span { class: "pill", "{service.options.len()} options" } }
                            if service.systemd_units.is_empty() {
                                p { class: "muted", "No systemd units were associated through Nix declaration provenance." }
                            } else {
                                div { class: "tags", aria_label: "Associated systemd units",
                                    for unit in &service.systemd_units {
                                        a { class: "tag", href: "/runtime?runtime_q={unit}", "{unit}.service" }
                                    }
                                }
                            }
                            div { class: "option-list",
                                for option in service.options {
                                    {
                                        let description = option.description.as_deref().unwrap_or("No description available");
                                        let option_type = option.option_type.as_deref().unwrap_or("Unknown type");
                                        let declarations = if option.declarations.is_empty() {
                                            "Declaration unavailable".to_owned()
                                        } else {
                                            option.declarations.join(", ")
                                        };
                                        let current_value = option.current_value.as_deref().unwrap_or("Value unavailable");
                                        rsx! {
                                            details { class: "option",
                                                summary { class: "entry-summary", code { class: "option-name", "{option.name}" } div { class: "muted", "{option_type} · {declarations}" } }
                                                div { class: "entry-body",
                                                    div { "{description}" }
                                                    div { class: "muted", "Current Value" }
                                                    code { "{current_value}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Err(message) => rsx! { div { class: "error", "Service details unavailable: {message}" } },
                    }
                }
            }
        }
    }
}

#[component]
fn Dashboard(page: DashboardPage, data: DashboardData) -> Element {
    let DashboardData {
        system,
        configuration,
        managed_configuration,
        option_catalog,
        package_catalog,
        service_catalog,
        runtime_services,
        generations,
    } = data;
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
    let generation = system
        .active_generation
        .map_or_else(|| "Unavailable".to_owned(), |value| value.to_string());
    let active_generation_number = system.active_generation;
    let kernel = system.kernel_version.as_deref().unwrap_or("Unavailable");
    let uptime = system
        .uptime_seconds
        .map_or_else(|| "Unavailable".to_owned(), format_uptime);
    let cpu = system.cpu_count.map_or_else(
        || "Unavailable".to_owned(),
        |count| format!("{count} logical CPUs"),
    );
    let load = system.load_average.as_deref().unwrap_or("Unavailable");
    let memory = match (system.memory_total_bytes, system.memory_available_bytes) {
        (Some(total), Some(available)) => {
            format!(
                "{} used / {} total",
                format_bytes(total - available),
                format_bytes(total)
            )
        }
        _ => "Unavailable".to_owned(),
    };
    let disk = match (
        system.root_disk_total_bytes,
        system.root_disk_available_bytes,
    ) {
        (Some(total), Some(available)) => {
            format!(
                "{} used / {} total",
                format_bytes(total - available),
                format_bytes(total)
            )
        }
        _ => "Unavailable".to_owned(),
    };
    let host = configuration.host.as_deref().unwrap_or("Automatic");
    let flake_uri = configuration.flake_uri.as_deref().unwrap_or("None");
    let managed_mode = match managed_configuration.mode {
        ManagedConfigurationMode::Unmanaged => "Unmanaged",
        ManagedConfigurationMode::ManualWiring => "Manual wiring",
        ManagedConfigurationMode::ControlPlaneManaged => "Control-plane managed",
    };
    let managed_directory = managed_configuration
        .managed_directory
        .as_deref()
        .unwrap_or("Unavailable for this configuration target");
    let host_module = managed_configuration
        .host_module
        .as_deref()
        .unwrap_or("Select a host to derive its managed module");
    let page_class = page.class_name();
    let page_title = page.title();
    rsx! {
        a { class: "skip-link", href: "#main-content", "Skip to Main Content" }
        main { class: "shell {page_class}", id: "main-content",
            header { class: "topbar",
                a { class: "brand", href: "/", span { class: "mark", "N" } "NixOS Control Plane" }
                nav { class: "nav", aria_label: "Dashboard sections",
                    a { class: if page == DashboardPage::Configuration { "active" } else { "" }, "aria-current": if page == DashboardPage::Configuration { "page" } else { "false" }, href: "/configuration", "Configuration" }
                    a { class: if page == DashboardPage::Packages { "active" } else { "" }, "aria-current": if page == DashboardPage::Packages { "page" } else { "false" }, href: "/packages", "Packages" }
                    a { class: if page == DashboardPage::Services { "active" } else { "" }, "aria-current": if page == DashboardPage::Services { "page" } else { "false" }, href: "/services", "Services" }
                    a { class: if page == DashboardPage::Runtime { "active" } else { "" }, "aria-current": if page == DashboardPage::Runtime { "page" } else { "false" }, href: "/runtime", "Runtime" }
                    a { class: if page == DashboardPage::Generations { "active" } else { "" }, "aria-current": if page == DashboardPage::Generations { "page" } else { "false" }, href: "/generations", "Generations" }
                    a { href: "/inputs", "Inputs" }
                }
                div { class: "status", span { class: "dot" } "Daemon connected" }
            }
            section { class: "page-head",
                div {
                    h1 { "{page_title}" }
                    p { class: "lede", code { "{host}" } " · " code { "{flake_uri}" } }
                }
            }
            section { class: "grid",
                article { class: "card page-section overview-section",
                    div { class: "card-head", h2 { "System" } span { class: "pill", "Active" } }
                    div { class: "primary", "{system.hostname}" }
                    div { class: "muted", "NixOS {version}" }
                }
                article { class: "card page-section overview-section",
                    div { class: "card-head", h2 { "Configuration" } span { class: "pill", "{kind}" } }
                    div { class: "primary", "{source}" }
                    div { class: "muted", if configuration.has_lock_file { "Lock file detected" } else { "No lock file detected" } }
                }
                article { class: "card wide page-section overview-section",
                    dl { class: "facts",
                        div { dt { "Flake URI" } dd { code { "{flake_uri}" } } }
                        div { dt { "Selected host" } dd { "{host}" } }
                        div { dt { "Active generation" } dd { "{generation}" } }
                    }
                }
                article { class: "card wide page-section overview-section",
                    div { class: "card-head", h2 { "Runtime Health" } span { class: "pill", "Live" } }
                    dl { class: "facts",
                        div { dt { "Kernel" } dd { code { "{kernel}" } } }
                        div { dt { "Uptime" } dd { "{uptime}" } }
                        div { dt { "CPU / load (1m 5m 15m)" } dd { "{cpu} · {load}" } }
                        div { dt { "Memory" } dd { "{memory}" } }
                        div { dt { "Root disk" } dd { "{disk}" } }
                        div { dt { "Current system" } dd { code { "{current_system}" } } }
                    }
                }
                article { class: "card wide page-section configuration-section",
                    div { class: "card-head", h2 { "Managed configuration" } span { class: "pill", "{managed_mode}" } }
                    dl { class: "facts",
                        div { dt { "Reserved directory" } dd { code { "{managed_directory}" } } }
                        div { dt { "Host module" } dd { code { "{host_module}" } } }
                        div { dt { "Edit boundary" } dd {
                            if managed_configuration.definitions_editable { "Owned definitions only" } else { "Read-only" }
                        } }
                    }
                    p { class: "muted",
                        if managed_configuration.mode == ManagedConfigurationMode::ControlPlaneManaged {
                            "Repository wiring may be maintained automatically because ownership metadata is present."
                        } else {
                            "Import the host module manually; unmanaged repository files will not be modified."
                        }
                    }
                }
                article { class: "card options page-section configuration-section",
                    div { class: "card-head", h2 { "NixOS Options" }
                        if let Ok(catalog) = &option_catalog { span { class: "pill", id: "option-count", "aria-live": "polite", "{catalog.total} evaluated" } }
                    }
                    match option_catalog {
                        Ok(catalog) => rsx! {
                            label { class: "visually-hidden", r#for: "option-search", "Filter NixOS Options" }
                            input { class: "search", id: "option-search", name: "option_q", autocomplete: "off", r#type: "search", placeholder: "Filter options by name…", aria_label: "Filter NixOS options" }
                            div { class: "option-list", id: "option-list",
                                for option in catalog.options {
                                    {
                                        let search_name = option.name.to_lowercase();
                                        let description = option.description.as_deref().unwrap_or("No description available");
                                        let option_type = option.option_type.as_deref().unwrap_or("Unknown type");
                                        rsx! {
                                            details { class: "option", "data-option-name": "{search_name}",
                                                summary { class: "entry-summary",
                                                    code { class: "option-name", "{option.name}" }
                                                    div { class: "option-description", "{option_type}" }
                                                }
                                                div { class: "entry-body",
                                                    "{description}"
                                                    br {}
                                                    a { class: "detail-link", href: "/configuration/options/{option.name}", "View Option Details" }
                                                }
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
                article { class: "card options page-section packages-section",
                    div { class: "card-head", h2 { "System Packages" }
                        if let Ok(catalog) = &package_catalog { span { class: "pill", id: "package-count", "aria-live": "polite", "{catalog.total} effective" } }
                    }
                    match package_catalog {
                        Ok(catalog) => rsx! {
                            div { class: "tags",
                                button { class: "tag active", r#type: "button", "aria-pressed": "true", "data-package-source": "", "All" }
                                for source in &catalog.sources {
                                    button { class: "tag", r#type: "button", "aria-pressed": "false", "data-package-source": "{source}", "{source}" }
                                }
                            }
                            label { class: "visually-hidden", r#for: "package-search", "Filter System Packages" }
                            input { class: "search", id: "package-search", name: "package_q", autocomplete: "off", r#type: "search", placeholder: "Filter environment.systemPackages…", aria_label: "Filter system packages" }
                            div { class: "option-list", id: "package-list",
                                for package in catalog.packages {
                                    {
                                        let label = package.version.as_ref().map_or_else(|| package.name.clone(), |version| format!("{} {version}", package.name));
                                        let description = package.description.as_deref().unwrap_or(&package.store_path);
                                        let source = package.source.as_deref().unwrap_or("unknown");
                                        rsx! {
                                            details { class: "option",
                                                summary { class: "entry-summary package-row",
                                                    code { class: "option-name", "{label}" }
                                                    code { class: "option-description", "{source}" }
                                                }
                                                div { class: "entry-body", "{description}" }
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
                article { class: "card options page-section services-section",
                    div { class: "card-head", h2 { "Enabled Services" }
                        if let Ok(catalog) = &service_catalog { span { class: "pill", id: "service-count", "aria-live": "polite", "{catalog.enabled} enabled / {catalog.total}" } }
                    }
                    match service_catalog {
                        Ok(catalog) => rsx! {
                            label { class: "visually-hidden", r#for: "service-search", "Filter Enabled Services" }
                            input { class: "search", id: "service-search", name: "service_q", autocomplete: "off", r#type: "search", placeholder: "Filter enabled services…", aria_label: "Filter enabled services" }
                            div { class: "option-list", id: "service-list",
                                for service in catalog.services {
                                    details { class: "option",
                                        summary { class: "entry-summary",
                                            code { class: "option-name", "{service.name}" }
                                            div { class: "option-description",
                                                if service.enabled == Some(true) { "Enabled" } else if service.enabled == Some(false) { "Disabled" } else { "State unavailable" }
                                            }
                                        }
                                        div { class: "entry-body",
                                            if let Some(description) = service.description { "{description}" } else { "No description available" }
                                            br {}
                                            a { class: "detail-link", href: "/services/{service.name}", "View Service Details" }
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
                article { class: "card options page-section runtime-section",
                    div { class: "card-head", h2 { "Runtime Services" }
                        if let Ok(catalog) = &runtime_services { span { class: "pill", id: "runtime-count", "aria-live": "polite", "{catalog.failed} failed / {catalog.total}" } }
                    }
                    match runtime_services {
                        Ok(catalog) => rsx! {
                            label { class: "visually-hidden", r#for: "runtime-search", "Filter Runtime Services" }
                            input { class: "search", id: "runtime-search", name: "runtime_q", autocomplete: "off", r#type: "search", placeholder: "Filter runtime units…", aria_label: "Filter runtime systemd services" }
                            div { class: "option-list", id: "runtime-list",
                                for service in catalog.services {
                                    details { class: "option",
                                        summary { class: "entry-summary",
                                            code { class: "option-name", "{service.unit}" }
                                            div { class: "option-description", "{service.active_state} / {service.sub_state}" }
                                        }
                                        div { class: "entry-body", "{service.description} · load state: {service.load_state}" }
                                    }
                                }
                            }
                            script { dangerous_inner_html: RUNTIME_FILTER_SCRIPT }
                        },
                        Err(message) => rsx! { div { class: "error", "Runtime service state unavailable: {message}" } },
                    }
                }
                article { class: "card options page-section generations-section",
                    div { class: "card-head", h2 { "Installed Generations" } span { class: "pill", "{generations.total} profiles" } }
                    if generations.generations.is_empty() {
                        p { class: "muted", "No NixOS system profile generations were found." }
                    } else {
                        div { class: "option-list",
                            for generation in generations.generations {
                                {
                                    let version = generation.nixos_version.as_deref().unwrap_or("Unknown NixOS version");
                                    let created = generation.created_unix_seconds.map_or_else(|| "Creation time unavailable".to_owned(), |seconds| seconds.to_string());
                                    rsx! {
                                        details { class: "option",
                                            summary { class: "entry-summary",
                                                code { class: "option-name", "Generation {generation.generation}" }
                                                div { class: "option-description", "{version}" }
                                            }
                                            div { class: "entry-body",
                                                div { class: "tags", aria_label: "Generation status",
                                                    if generation.active { span { class: "tag active", "Active" } }
                                                    if generation.current { span { class: "tag active", "Current profile" } }
                                                    if generation.bootable { span { class: "tag", "Bootable" } } else { span { class: "tag", "Not bootable" } }
                                                }
                                                dl { class: "facts",
                                                    div { dt { "Created" } dd { span { "data-unix-seconds": "{created}", "{created}" } } }
                                                    div { dt { "Generation" } dd { "{generation.generation}" } }
                                                    div { dt { "System path" } dd { code { "{generation.system_path}" } } }
                                                }
                                                a { class: "detail-link", href: "/generations/{generation.generation}", "View Generation Details" }
                                                if let Some(active) = active_generation_number.filter(|active| *active != generation.generation) {
                                                    " · "
                                                    a { class: "detail-link", href: "/generations/{generation.generation}/compare/{active}", "Compare with Active" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        script { dangerous_inner_html: GENERATION_TIME_SCRIPT }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DashboardData, DashboardPage, render_dashboard, render_option_detail, render_service_detail,
    };
    use nixos_control_api_types::{
        ConfigurationKind, ConfigurationSummary, ManagedConfigurationMode,
        ManagedConfigurationSummary, SystemSummary,
    };

    fn generation_fixture() -> nixos_control_api_types::GenerationCatalog {
        nixos_control_api_types::GenerationCatalog {
            total: 1,
            generations: vec![nixos_control_api_types::SystemGeneration {
                generation: 42,
                system_path: "/nix/store/example-system".to_owned(),
                nixos_version: Some("26.11".to_owned()),
                created_unix_seconds: Some(1_700_000_000),
                current: true,
                active: true,
                bootable: true,
            }],
        }
    }

    #[test]
    fn renders_system_and_flake_details() {
        let html = render_dashboard(
            DashboardPage::Overview,
            DashboardData {
                system: SystemSummary {
                    hostname: "snowflake".to_owned(),
                    nixos_version: Some("26.11".to_owned()),
                    current_system: Some("/nix/store/example-system".to_owned()),
                    active_generation: Some(42),
                    kernel_version: Some("6.12.0".to_owned()),
                    uptime_seconds: Some(90_000),
                    cpu_count: Some(8),
                    load_average: Some("0.10 0.20 0.30".to_owned()),
                    memory_total_bytes: Some(16 * 1024 * 1024 * 1024),
                    memory_available_bytes: Some(8 * 1024 * 1024 * 1024),
                    root_disk_total_bytes: Some(100 * 1024 * 1024 * 1024),
                    root_disk_available_bytes: Some(60 * 1024 * 1024 * 1024),
                },
                configuration: ConfigurationSummary {
                    kind: ConfigurationKind::Flake,
                    source: Some("/etc/nixos/flake.nix".to_owned()),
                    flake_uri: Some("/etc/nixos".to_owned()),
                    host: Some("snowflake".to_owned()),
                    has_lock_file: true,
                },
                managed_configuration: ManagedConfigurationSummary {
                    repository_root: Some("/etc/nixos".to_owned()),
                    managed_directory: Some("/etc/nixos/ncp".to_owned()),
                    host: Some("snowflake".to_owned()),
                    host_module: Some("/etc/nixos/ncp/hosts/snowflake.nix".to_owned()),
                    host_module_exists: true,
                    metadata_exists: false,
                    mode: ManagedConfigurationMode::ManualWiring,
                    definitions_editable: true,
                },
                option_catalog: Ok(nixos_control_api_types::OptionCatalog {
                    host: "snowflake".to_owned(),
                    total: 1,
                    options: vec![nixos_control_api_types::OptionSummary {
                        name: "services.openssh.enable".to_owned(),
                        description: Some("Enable OpenSSH".to_owned()),
                        option_type: Some("boolean".to_owned()),
                        read_only: false,
                        declarations: Vec::new(),
                        current_value: None,
                    }],
                }),
                package_catalog: Ok(nixos_control_api_types::PackageCatalog {
                    host: "snowflake".to_owned(),
                    total: 1,
                    packages: vec![nixos_control_api_types::SystemPackage {
                        name: "curl".to_owned(),
                        version: Some("8.0".to_owned()),
                        description: Some("Command line HTTP client".to_owned()),
                        store_path: "/nix/store/example-curl".to_owned(),
                        source: Some("nixpkgs".to_owned()),
                    }],
                    sources: vec!["nixpkgs".to_owned()],
                }),
                service_catalog: Ok(nixos_control_api_types::ServiceCatalog {
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
                runtime_services: Ok(nixos_control_api_types::RuntimeServiceCatalog {
                    total: 1,
                    failed: 0,
                    services: vec![nixos_control_api_types::RuntimeService {
                        unit: "sshd.service".to_owned(),
                        description: "OpenSSH server".to_owned(),
                        load_state: "loaded".to_owned(),
                        active_state: "active".to_owned(),
                        sub_state: "running".to_owned(),
                    }],
                }),
                generations: generation_fixture(),
            },
        );
        assert!(html.contains("snowflake"));
        assert!(html.contains("/etc/nixos/flake.nix"));
        assert!(html.contains("Lock file detected"));
        assert!(html.contains("Manual wiring"));
        assert!(html.contains("/etc/nixos/ncp/hosts/snowflake.nix"));
        assert!(html.contains("href=\"/packages\""));
        assert!(html.contains("href=\"/services/openssh\""));
        assert!(html.contains("<details class=\"option\""));
        assert!(html.contains("<summary class=\"entry-summary"));
        assert!(html.contains("}));\n  const selectedTag"));
        assert!(html.contains("Skip to Main Content"));
        assert!(html.contains("System Overview"));
        assert!(!html.contains("Declarative systems, clearly understood"));
        assert!(html.contains("page-overview"));
    }

    #[test]
    fn renders_service_current_values() {
        let html = render_service_detail(
            SystemSummary {
                hostname: "snowflake".to_owned(),
                nixos_version: None,
                current_system: None,
                active_generation: None,
                kernel_version: None,
                uptime_seconds: None,
                cpu_count: None,
                load_average: None,
                memory_total_bytes: None,
                memory_available_bytes: None,
                root_disk_total_bytes: None,
                root_disk_available_bytes: None,
            },
            ConfigurationSummary {
                kind: ConfigurationKind::Flake,
                source: None,
                flake_uri: Some("/etc/nixos".to_owned()),
                host: Some("snowflake".to_owned()),
                has_lock_file: true,
            },
            Ok(nixos_control_api_types::ServiceDetail {
                name: "openssh".to_owned(),
                systemd_units: vec!["sshd".to_owned()],
                options: vec![nixos_control_api_types::OptionSummary {
                    name: "services.openssh.enable".to_owned(),
                    description: Some("Enable OpenSSH".to_owned()),
                    option_type: Some("boolean".to_owned()),
                    read_only: false,
                    declarations: Vec::new(),
                    current_value: Some("true".to_owned()),
                }],
            }),
        );
        assert!(html.contains("Current Value"));
        assert!(html.contains(">true</code>"));
        assert!(html.contains("sshd.service"));
    }

    #[test]
    fn renders_option_metadata_and_values() {
        let system = SystemSummary {
            hostname: "snowflake".to_owned(),
            nixos_version: None,
            current_system: None,
            active_generation: None,
            kernel_version: None,
            uptime_seconds: None,
            cpu_count: None,
            load_average: None,
            memory_total_bytes: None,
            memory_available_bytes: None,
            root_disk_total_bytes: None,
            root_disk_available_bytes: None,
        };
        let configuration = ConfigurationSummary {
            kind: ConfigurationKind::Flake,
            source: None,
            flake_uri: Some("/etc/nixos".to_owned()),
            host: Some("snowflake".to_owned()),
            has_lock_file: true,
        };
        let html = render_option_detail(
            system,
            configuration,
            Ok(nixos_control_api_types::OptionDetail {
                name: "services.openssh.enable".to_owned(),
                description: Some("Enable OpenSSH".to_owned()),
                option_type: Some("boolean".to_owned()),
                read_only: false,
                declarations: vec!["/nix/store/source/sshd.nix".to_owned()],
                current_value: Some("true".to_owned()),
                default_value: Some("false".to_owned()),
                example: None,
            }),
        );
        assert!(html.contains("services.openssh.enable"));
        assert!(html.contains("Effective value"));
        assert!(html.contains(">true</code>"));
        assert!(html.contains(">false</code>"));
        assert!(html.contains("/nix/store/source/sshd.nix"));
    }
}
