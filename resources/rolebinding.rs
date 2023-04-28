use crate::traits::{api::ApiResource, delete::DeleteOpt, expire::Expire};
use crate::{meta::meta, resources::role::Role, CLIENT, NAMESPACE};
use k8s_openapi::api::rbac::v1::{ClusterRoleBinding, RoleRef, Subject};
use kube::api::PostParams;
use kube::Api;
use std::error::Error;

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
        expire_at: i64,
    ) -> Result<ClusterRoleBinding, Box<dyn Error>> {
        let namespace = NAMESPACE.get().unwrap().clone();
        let meta = meta(Some(name.clone()), None, Some(expire_at));
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
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Delete Role Binding
    pub async fn delete(&self, name: String) -> Result<(), kube::Error> {
        self.api.delete_opt(&name, &Default::default()).await?;
        Ok(())
    }
}

impl Expire for RoleBinding {}

impl ApiResource for RoleBinding {
    type ApiType = ClusterRoleBinding;

    fn get_api(&self) -> Api<Self::ApiType> {
        self.api.clone()
    }
}
