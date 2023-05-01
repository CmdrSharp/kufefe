use crate::traits::api::ApiResource;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::{core::ObjectMeta, Resource, ResourceExt};
use rand::distributions::{Alphanumeric, DistString};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[async_trait::async_trait]
pub trait Meta {
    /// Generates a unique resource name
    async fn generate_name(&self) -> String
    where
        Self: ApiResource,
    {
        // Get API
        let api = self.get_api();

        let name = format!(
            "kufefe-generated-{}",
            Alphanumeric.sample_string(&mut rand::thread_rng(), 6)
        )
        .to_lowercase();

        // Verify that the name does not already exist for the resource
        if api.get(&name).await.is_ok() {
            tracing::info!("Name {} already exists, generating a new one", name);
            return self.generate_name().await;
        }

        name
    }

    /// Gets ownership labels
    fn labels() -> std::collections::BTreeMap<String, String> {
        let mut m = std::collections::BTreeMap::new();

        m.insert(
            "app.kubernetes.io/managed-by".to_string(),
            "kufefe".to_string(),
        );

        m.insert("kufefe.io/owned-by".to_string(), "debug".to_string());

        m
    }

    /// Creates metadata for Kubernetes resources
    async fn generate_meta<T>(
        &self,
        mut name: Option<String>,
        namespace: Option<String>,
        owner: &T,
    ) -> ObjectMeta
    where
        Self: ApiResource,
        T: Resource<DynamicType = ()> + DeserializeOwned + Debug + Clone + Send + Sync,
    {
        if name.is_none() {
            name = Some(self.generate_name().await);
        }

        let mut meta = ObjectMeta {
            name,
            namespace,
            labels: Some(Self::labels()),
            ..ObjectMeta::default()
        };

        let api_version = <T as Resource>::api_version(&()).to_string();
        let kind = <T as Resource>::kind(&()).to_string();

        if owner.uid().is_some() {
            meta.owner_references = Some(vec![OwnerReference {
                api_version,
                kind,
                name: owner.name_any(),
                uid: owner.uid().unwrap(),
                controller: None,
                block_owner_deletion: None,
            }]);
        }

        meta
    }
}
