use crate::{crd::Request, meta::meta};
use crate::{CLIENT, NAMESPACE};
use k8s_openapi::api::core::v1::Secret;
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
        let client = CLIENT.get().unwrap().clone();
        let namespace = NAMESPACE.get().unwrap().clone();
        let api: Api<Secret> = Api::namespaced(client, &namespace);

        Self { namespace, api }
    }

    /// Create a new Service Account Token Secret
    pub async fn create(
        &self,
        name: String,
        owner: &Request,
    ) -> Result<Secret, kube::Error> {
        let mut metadata = meta(Some(name.clone()), Some(self.namespace.clone()), owner);

        let mut annotations = match metadata.annotations {
            Some(annotations) => annotations,
            None => std::collections::BTreeMap::new(),
        };

        annotations.insert(
            "kubernetes.io/service-account.name".to_string(),
            name.clone(),
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
            Err(e) => Err(e),
        }
    }

    /// Get a secret
    pub async fn get(&self, name: String) -> Result<Secret, kube::Error> {
        self.api.get(&name).await
    }

    /// Get data from a secret idiomatically
    pub fn data(secret: Secret, key: &str) -> Result<ByteString, String> {
        let data = if let Some(data) = secret.data {
            data
        } else {
            return Err("Secret has no data".to_string());
        };

        if let Some(v) = data.get(key) {
            Ok(v.clone())
        } else {
            return Err(format!("Secret has no property {}", key));
        }
    }
}