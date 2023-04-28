use crate::{
    crd::Request,
    meta::meta,
    traits::{api::ApiResource, delete::DeleteOpt, expire::Expire},
    CLIENT, NAMESPACE,
};
use k8s_openapi::api::core::v1::ServiceAccount as KubeServiceAccount;
use kube::api::DeleteParams;
use kube::api::PostParams;
use kube::Api;

pub struct ServiceAccount {
    namespace: String,
    api: Api<KubeServiceAccount>,
}

impl ServiceAccount {
    /// Instantiate a ServiceAccount struct
    pub fn new() -> Self {
        let client = CLIENT.get().unwrap().clone();

        let namespace = NAMESPACE.get().unwrap();
        let api: Api<KubeServiceAccount> = Api::namespaced(client, namespace);

        Self {
            namespace: namespace.clone(),
            api,
        }
    }

    /// Create the Service Account in Kubernetes
    pub async fn create(
        &self,
        name: String,
        expire_at: i64,
        owner: &Request,
    ) -> Result<KubeServiceAccount, kube::Error> {
        let meta = meta(
            Some(name.clone()),
            Some(self.namespace.clone()),
            Some(expire_at),
            owner,
        );

        // Construct the API Object
        let sa = KubeServiceAccount {
            metadata: meta,
            automount_service_account_token: Some(true),
            ..KubeServiceAccount::default()
        };

        // Create the ServiceAccount
        match self.api.create(&PostParams::default(), &sa).await {
            Ok(o) => {
                tracing::info!("Created ServiceAccount {}", &name);
                Ok(o)
            }
            Err(e) => Err(e),
        }
    }

    /// Delete a ServiceAccount
    pub async fn _delete(&self, name: String) -> Result<(), kube::Error> {
        self.api.delete_opt(&name, &DeleteParams::default()).await?;
        Ok(())
    }
}

impl Expire for ServiceAccount {}

impl ApiResource for ServiceAccount {
    type ApiType = KubeServiceAccount;

    fn get_api(&self) -> Api<Self::ApiType> {
        self.api.clone()
    }
}
