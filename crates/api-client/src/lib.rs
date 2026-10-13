use nixos_control_api_types::{
    API_VERSION, ApiError, CapabilityResponse, ConfigurationSummary, GenerationCatalog,
    GenerationDetail, HealthResponse, ManagedConfigurationSummary, OptionCatalog, OptionDetail,
    PackageCatalog, RuntimeServiceCatalog, ServiceCatalog, ServiceDetail, SystemSummary,
};
use reqwest::StatusCode;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("invalid daemon URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("daemon request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("daemon returned HTTP {status}: {error}", error = .error.message)]
    Api { status: StatusCode, error: ApiError },
    #[error("daemon returned HTTP {0} without a structured error")]
    Status(StatusCode),
}

#[derive(Debug, Clone)]
pub struct Client {
    base_url: Url,
    http: reqwest::Client,
}

impl Client {
    /// Creates a client for an HTTP daemon base URL.
    ///
    /// # Errors
    /// Returns [`ClientError::Url`] when `base_url` is not a valid URL.
    pub fn new(base_url: &str) -> Result<Self, ClientError> {
        Ok(Self {
            base_url: Url::parse(base_url)?,
            http: reqwest::Client::new(),
        })
    }

    /// Fetches daemon health and version information.
    ///
    /// # Errors
    /// Returns an error when the request fails or the response is invalid.
    pub async fn health(&self) -> Result<HealthResponse, ClientError> {
        self.get("health").await
    }

    /// Fetches supported API versions and capabilities.
    ///
    /// # Errors
    /// Returns an error when the request fails or the response is invalid.
    pub async fn capabilities(&self) -> Result<CapabilityResponse, ClientError> {
        self.get("api").await
    }

    /// Fetches the local system summary.
    ///
    /// # Errors
    /// Returns an error when the request fails or the response is invalid.
    pub async fn system_summary(&self) -> Result<SystemSummary, ClientError> {
        self.get(&format!("api/{API_VERSION}/system/summary")).await
    }

    /// Fetches installed NixOS system profile generations.
    ///
    /// # Errors
    /// Returns an error when the request fails or the response is invalid.
    pub async fn generations(&self) -> Result<GenerationCatalog, ClientError> {
        self.get(&format!("api/{API_VERSION}/generations")).await
    }

    /// Fetches closure metadata for one installed NixOS generation.
    ///
    /// # Errors
    /// Returns an error when the generation is absent or the response is invalid.
    pub async fn generation(&self, number: u64) -> Result<GenerationDetail, ClientError> {
        self.get(&format!("api/{API_VERSION}/generations/{number}"))
            .await
    }

    /// Fetches the configured NixOS source and detected configuration kind.
    ///
    /// # Errors
    /// Returns an error when the request fails or the response is invalid.
    pub async fn configuration(&self) -> Result<ConfigurationSummary, ClientError> {
        self.get(&format!("api/{API_VERSION}/configuration")).await
    }

    /// Fetches the selected repository's control-plane ownership boundary.
    ///
    /// # Errors
    /// Returns an error when the request fails or the response is invalid.
    pub async fn managed_configuration(&self) -> Result<ManagedConfigurationSummary, ClientError> {
        self.get(&format!("api/{API_VERSION}/configuration/managed"))
            .await
    }

    /// Fetches evaluated NixOS option metadata for the selected host.
    ///
    /// # Errors
    /// Returns an error when evaluation fails or the response is invalid.
    pub async fn options(&self) -> Result<OptionCatalog, ClientError> {
        self.get(&format!("api/{API_VERSION}/options")).await
    }

    /// Fetches evaluated metadata and values for one NixOS option.
    ///
    /// # Errors
    /// Returns an error when evaluation fails or the response is invalid.
    pub async fn option(&self, option: &str) -> Result<OptionDetail, ClientError> {
        let option: String = url::form_urlencoded::byte_serialize(option.as_bytes()).collect();
        self.get(&format!("api/{API_VERSION}/options/{option}"))
            .await
    }

    /// Fetches evaluated packages from `environment.systemPackages`.
    ///
    /// # Errors
    /// Returns an error when evaluation fails or the response is invalid.
    pub async fn packages(&self) -> Result<PackageCatalog, ClientError> {
        self.get(&format!("api/{API_VERSION}/packages")).await
    }

    /// Fetches services represented by boolean NixOS enable options.
    ///
    /// # Errors
    /// Returns an error when evaluation fails or the response is invalid.
    pub async fn services(&self) -> Result<ServiceCatalog, ClientError> {
        self.get(&format!("api/{API_VERSION}/services")).await
    }

    /// Fetches option metadata for a configured service namespace.
    ///
    /// # Errors
    /// Returns an error when evaluation fails or the response is invalid.
    pub async fn service(&self, service: &str) -> Result<ServiceDetail, ClientError> {
        let service: String = url::form_urlencoded::byte_serialize(service.as_bytes()).collect();
        self.get(&format!("api/{API_VERSION}/services/{service}"))
            .await
    }

    /// Fetches runtime systemd service states.
    ///
    /// # Errors
    /// Returns an error when systemd inspection fails or the response is invalid.
    pub async fn runtime_services(&self) -> Result<RuntimeServiceCatalog, ClientError> {
        self.get(&format!("api/{API_VERSION}/runtime/services"))
            .await
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        let response = self.http.get(self.base_url.join(path)?).send().await?;
        let status = response.status();
        if !status.is_success() {
            return match response.json::<ApiError>().await {
                Ok(error) => Err(ClientError::Api { status, error }),
                Err(_) => Err(ClientError::Status(status)),
            };
        }
        Ok(response.json().await?)
    }
}
