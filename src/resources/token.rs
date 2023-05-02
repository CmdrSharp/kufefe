use crate::traits::{api::ApiResource, meta::Meta};
use crate::CONFIG;
use anyhow::{bail, Result};
use k8s_openapi::api::core::v1::{Secret, ServiceAccount};
use k8s_openapi::ByteString;
use kube::api::PostParams;
use kube::Api;

pub struct Token {
    namespace: String,
    api: Api<Secret>,
}

impl Token {
    /// Instantiate Token Struct
    pub fn new() -> Self {
        let client = CONFIG.get().unwrap().client();
        let namespace = CONFIG.get().unwrap().namespace();
        let api: Api<Secret> = Api::namespaced(client, &namespace);

        Self { namespace, api }
    }

    /// Create a new Service Account Token Secret
    pub async fn create(&self, name: String, owner: &ServiceAccount) -> Result<Secret> {
        // Convert the k8sapi ServiceAccount to a kube ServiceACcount
        let mut metadata = self
            .generate_meta(Some(name.clone()), Some(self.namespace.clone()), owner)
            .await;

        let mut annotations = match metadata.annotations {
            Some(annotations) => annotations,
            None => std::collections::BTreeMap::new(),
        };

        annotations.insert(
            "kubernetes.io/service-account.name".to_string(),
            owner
                .metadata
                .name
                .clone()
                .expect("Service Account has no name"),
        );

        metadata.annotations = Some(annotations);

        // Create the Secret
        let secret = Secret {
            metadata,
            type_: Some("kubernetes.io/service-account-token".to_string()),
            ..Secret::default()
        };

        match self.api.create(&PostParams::default(), &secret).await {
            Ok(o) => {
                tracing::info!("Created Secret (SA Token) {}", name);
                Ok(o)
            }
            Err(e) => bail!(e),
        }
    }

    /// Get data from a secret idiomatically
    pub async fn data(&self, secret_name: String, key: &str) -> Result<ByteString> {
        let secret = self.get_api().get(&secret_name).await?;

        let data = if let Some(data) = secret.data {
            data
        } else {
            bail!("Secret has no data");
        };

        if let Some(v) = data.get(key) {
            Ok(v.clone())
        } else {
            bail!("Secret has no property {}", key);
        }
    }
}

impl ApiResource for Token {
    type ApiType = Secret;

    fn get_api(&self) -> Api<Self::ApiType> {
        self.api.clone()
    }
}

impl Meta for Token {}
