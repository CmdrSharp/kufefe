use crate::{crd::Request, meta::meta, CLIENT, NAMESPACE};
use k8s_openapi::api::core::v1::ServiceAccount as KubeServiceAccount;
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
        owner: &Request,
    ) -> Result<KubeServiceAccount, kube::Error> {
        let meta = meta(Some(name.clone()), Some(self.namespace.clone()), owner);

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
}