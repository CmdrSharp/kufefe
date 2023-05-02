use crate::traits::api::ApiResource;
use crate::{resources::role::Role, traits::meta::Meta, Request, CONFIG};
use anyhow::{bail, Result};
use k8s_openapi::api::core::v1::ServiceAccount;
use k8s_openapi::api::rbac::v1::{ClusterRoleBinding, RoleRef, Subject};
use kube::api::PostParams;
use kube::Api;

pub struct RoleBinding {
    api: Api<ClusterRoleBinding>,
}

impl RoleBinding {
    /// Instantiate a RoleBinding struct
    pub fn new() -> Self {
        let client = CONFIG.get().unwrap().client();
        let api: Api<ClusterRoleBinding> = Api::all(client);

        Self { api }
    }

    /// Create the RoleBinding in Kubernetes
    pub async fn create(
        &self,
        name: String,
        role: String,
        sa: &ServiceAccount,
        owner: &Request,
    ) -> Result<ClusterRoleBinding> {
        let namespace = CONFIG.get().unwrap().namespace();
        let meta = self.generate_meta(Some(name.clone()), None, owner).await;
        let role_api = Role::new();

        // Get the owner name
        let sa_name = if let Some(name) = sa.metadata.name.clone() {
            name
        } else {
            bail!("ServiceAccount has no name");
        };

        // Check if the specified role has the annotation kufefe.io/role.
        role_api.get(&role).await?;

        // Construct a subject
        let subject = Subject {
            kind: "ServiceAccount".to_string(),
            name: sa_name,
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
