use crate::{resources::token::Token, CLUSTER_URL};
use anyhow::{bail, Result};
use base64::{engine::general_purpose, Engine as _};
use k8s_openapi::api::core::v1::{Secret, ServiceAccount};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::Retry;

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Kubeconfig {
    api_version: String,
    clusters: Vec<Cluster>,
    contexts: Vec<Context>,
    #[serde(rename = "current-context")]
    current_context: String,
    kind: String,
    preferences: Preferences,
    users: Vec<User>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Cluster {
    cluster: ClusterDetails,
    name: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct ClusterDetails {
    #[serde(rename = "certificate-authority-data")]
    certificate_authority_data: String,
    server: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Context {
    context: ContextDetails,
    name: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct ContextDetails {
    cluster: String,
    user: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Preferences {}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct User {
    name: String,
    user: UserDetails,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct UserDetails {
    token: String,
}

impl Kubeconfig {
    /// Generetes a new Kubeconfig Struct
    pub async fn new(sa: ServiceAccount, secret: Secret) -> Result<Self> {
        // Get name of resources
        let sa_name = if let Some(name) = sa.metadata.name.as_ref() {
            name
        } else {
            bail!("ServiceAccount name is missing");
        };

        let secret_name = if let Some(name) = secret.metadata.name.as_ref() {
            name
        } else {
            bail!("Secret name is missing");
        };

        // Create an exponential backoff strategy
        let retry_strategy = ExponentialBackoff::from_millis(5)
            .factor(1000)
            .max_delay(Duration::from_secs(60))
            .take(30);

        // Get the CA
        let ca = Retry::spawn(retry_strategy.clone(), || {
            tracing::debug!(
                "Attempting to get CA for SA {}, secret {}",
                sa_name,
                secret_name
            );

            Self::get_ca(&secret)
        })
        .await?;

        // Get the Token
        let token = Retry::spawn(retry_strategy, || {
            tracing::debug!(
                "Attempting to get token for SA {}, secret {}",
                sa_name,
                secret_name
            );

            Self::get_token(&secret)
        })
        .await?;

        Ok(Self {
            api_version: "v1".to_string(),
            clusters: vec![Cluster {
                cluster: ClusterDetails {
                    certificate_authority_data: general_purpose::STANDARD.encode(ca),
                    server: CLUSTER_URL.get().unwrap().clone(),
                },
                name: "kubernetes".to_string(),
            }],
            contexts: vec![Context {
                context: ContextDetails {
                    cluster: "kubernetes".to_string(),
                    user: sa_name.clone(),
                },
                name: "kubernetes".to_string(),
            }],
            current_context: "kubernetes".to_string(),
            kind: "Config".to_string(),
            preferences: Preferences {},
            users: vec![User {
                name: sa_name.clone(),
                user: UserDetails { token },
            }],
        })
    }

    /// Converts the Kubeconfig Struct to YAML
    pub fn to_yaml(&self) -> Result<String> {
        Ok(serde_yaml::to_string(&self)?)
    }

    /// Gets the CA from the Secret
    async fn get_ca(secret: &Secret) -> Result<String> {
        let ca = Token::new()
            .data(secret.metadata.name.clone().unwrap(), "ca.crt")
            .await?;

        match String::from_utf8(ca.0) {
            Ok(ca) => Ok(ca),
            Err(_) => bail!("Could not parse ca.crt"),
        }
    }

    /// Gets the Token from the Secret
    async fn get_token(secret: &Secret) -> Result<String> {
        let token = Token::new()
            .data(secret.metadata.name.clone().unwrap(), "token")
            .await?;

        match String::from_utf8(token.0) {
            Ok(token) => Ok(token),
            Err(_) => bail!("Could not parse token"),
        }
    }
}
