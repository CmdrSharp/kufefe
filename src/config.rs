use crate::resources::gke::cluster::Cluster;
use anyhow::{bail, Result};
use kube::{api::ListParams, Api, Client};
use std::env;

pub struct KufefeConfig {
    url: String,
    namespace: String,
    client: Client,
}

impl KufefeConfig {
    /// Attempt to automatically fetch the cluster url from the environment
    pub async fn new() -> Result<Self> {
        let client = Client::try_default()
            .await
            .expect("Failed to create Kubernetes Client");

        let mut url: String = Self::from_env();
        let namespace = env::var("NAMESPACE").unwrap_or_else(|_| "default".to_string());

        // Handle fallback methods if URL isn't explicitly set
        if url.is_empty() {
            tracing::info!(
                "Cluster URL not explicitly set. Attempting to find it automatically.."
            );

            url = Self::anthos(client.clone()).await?
        }

        tracing::info!("Detected URL: {} and namespace: {}", url, namespace);

        Ok(Self {
            url,
            namespace,
            client: Client::try_default()
                .await
                .expect("Failed to generate Kubernetes Client"),
        })
    }

    /// Attempts to fetch the cluster url from GKE / Anthos
    async fn anthos(client: Client) -> Result<String> {
        tracing::info!("Attempting to find GKE/Anthos kind: Cluster");

        let api: Api<Cluster> = Api::namespaced(client, "default");

        match api.list(&ListParams::default()).await {
            Ok(list) => {
                if list.items.len() > 1 {
                    let cluster_name = env::var("CLUSTER_NAME");

                    if cluster_name.is_err() {
                        bail!("Found more than one cluster. Please specify the CLUSTER_NAME environment variable");
                    }

                    if let Some(cluster) = list.items.iter().find(|c| {
                        let cluster_name = cluster_name.clone().unwrap();
                        c.metadata.name.clone().unwrap() == cluster_name
                    }) {
                        if let Some(status) = &cluster.status {
                            return Ok(status.api_endpoints[0].host.clone());
                        }
                    }
                }

                if let Some(cluster) = list.items.first() {
                    if let Some(status) = &cluster.status {
                        return Ok(status.api_endpoints[0].host.clone());
                    }
                }

                bail!("Failed to find any cluster resources");
            }
            Err(e) => {
                bail!("Failed to find a cluster: {}", e);
            }
        }
    }

    /// Attempts to fetch the cluster url from the environment
    fn from_env() -> String {
        env::var("CLUSTER_URL").unwrap_or("".to_string())
    }

    /// Getter for URL
    pub fn url(&self) -> String {
        self.url.clone()
    }

    /// Getter for namespace
    pub fn namespace(&self) -> String {
        self.namespace.clone()
    }

    /// Getter for client
    pub fn client(&self) -> Client {
        self.client.clone()
    }
}
