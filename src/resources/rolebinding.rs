use crate::traits::api::ApiResource;
use crate::{crd::Request, resources::role::Role, traits::meta::Meta, CLIENT, NAMESPACE};
use anyhow::{bail, Result};
use k8s_openapi::api::rbac::v1::{ClusterRoleBinding, RoleRef, Subject};
use kube::api::PostParams;
use kube::Api;

pub struct RoleBinding {
    api: Api<ClusterRoleBinding>,
}

impl RoleBinding {
    /// Instantiate a RoleBinding struct
    pub fn new() -> Self {
        let client = CLIENT.get().unwrap().clone();
        let api: Api<ClusterRoleBinding> = Api::all(client);

        Self { api }
    }

    /// Create the RoleBinding in Kubernetes
    pub async fn create(
        &self,
        name: String,
        role: String,
        owner: &Request,
    ) -> Result<ClusterRoleBinding> {
        let namespace = NAMESPACE.get().unwrap().clone();
        let meta = self.generate_meta(Some(name.clone()), None, owner).await;
        let role_api = Role::new();

        // Check if the specified role has the annotation kufefe.io/role.
        role_api.get(&role).await?;

        // Construct a subject
        let subject = Subject {
            kind: "ServiceAccount".to_string(),
            name: name.clone(),
            namespace: Some(namespace),
            ..Subject::default()
        };

        let binding = ClusterRoleBinding {
            metadata: meta,
            subjects: Some(vec![subject]),
            role_ref: RoleRef {
                api_group: "rbac.authorization.k8s.io".to_string(),
                kind: "ClusterRole".to_string(),
                name: role,
            },
        };

        match self.api.create(&PostParams::default(), &binding).await {
            Ok(o) => {
                tracing::info!("Created RoleBinding {}", &name);
                Ok(o)
            }
            Err(e) => bail!(e),
        }
    }
}

impl ApiResource for RoleBinding {
    type ApiType = ClusterRoleBinding;

    fn get_api(&self) -> Api<Self::ApiType> {
        self.api.clone()
    }
}

impl Meta for RoleBinding {}
