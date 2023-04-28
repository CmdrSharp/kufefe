use crate::CLIENT;
use k8s_openapi::api::rbac::v1::ClusterRole;
use kube::Api;

pub struct Role {
    api: Api<ClusterRole>,
}

impl Role {
    /// Instantiate a Role struct
    pub fn new() -> Self {
        let client = CLIENT.get().unwrap().clone();
        let api: Api<ClusterRole> = Api::all(client);

        Self { api }
    }

    /// Find a role by name and verify it has the annotation kufefe.io/role
    pub async fn get(&self, name: &str) -> Result<ClusterRole, String> {
        match self.api.get(name).await {
            Ok(o) => {
                if o.metadata.annotations.is_none() {
                    return Err(
                        "Role does not have the annotation kufefe.io/role".to_string()
                    );
                }

                let annotations = o.clone().metadata.annotations.unwrap();
                if annotations.get("kufefe.io/role").is_none() {
                    return Err(
                        "Role does not have the annotation kufefe.io/role".to_string()
                    );
                }

                Ok(o)
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
