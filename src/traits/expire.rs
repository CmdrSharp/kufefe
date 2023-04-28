use crate::traits::delete::DeleteOpt;
use async_trait::async_trait;
use k8s_openapi::serde::de::DeserializeOwned;
use kube::api::DeleteParams;
use kube::api::ListParams;
use kube::Api;
use kube::Resource;
use std::any::type_name;
use std::fmt::Debug;
use std::marker::{Send, Sync};

#[async_trait]
pub trait Expire {
    /// Checks if the object is expired
    fn is_expired(expire_by: Option<i64>) -> bool {
        if let Some(expire_by) = expire_by {
            if expire_by < chrono::Utc::now().timestamp() {
                return true;
            }
        }

        false
    }

    /// Gets the expiry of a generic resource
    async fn get_expiry<T>(&self, resource: T) -> Option<i64>
    where
        T: Resource<DynamicType = ()> + Clone + Send + Sync,
    {
        if let Some(annotations) = resource.meta().clone().annotations {
            if let Some(expire_by) = annotations.get("kufefe.io/expire-by") {
                return Some(
                    expire_by.parse::<i64>().expect("Failed to parse expire-by"),
                );
            }
        }

        None
    }

    /// Scan for expired objects
    async fn scan<T>(&self, api: Api<T>)
    where
        T: Resource<DynamicType = ()> + DeserializeOwned + Debug + Clone + Send + Sync,
    {
        let r#type = type_name::<T>();
        tracing::info!("Scanning {}", r#type);

        let list_param = if let true = Self::require_managed_by_label() {
            ListParams::default().labels("app.kubernetes.io/managed-by=kufefe")
        } else {
            ListParams::default()
        };

        match api.list(&list_param).await {
            Ok(resources) => {
                for resource in &resources.items {
                    let expire_by = self.get_expiry(resource.clone());

                    let name = if let Some(name) = resource.meta().name.as_ref() {
                        name
                    } else {
                        tracing::debug!("{} has no name", r#type);
                        continue;
                    };

                    if Self::is_expired(expire_by.await) {
                        tracing::info!("{} {} expired", r#type, name);

                        if let Err(e) =
                            api.delete_opt(name, &DeleteParams::default()).await
                        {
                            tracing::error!("Error deleting {}: {}", r#type, e);
                            continue;
                        }
                    }
                }
            }
            Err(e) => tracing::error!("Error listing {}: {}", r#type, e),
        }
    }

    /// Whether to require a managed by label to delete resources
    fn require_managed_by_label() -> bool {
        true
    }
}
