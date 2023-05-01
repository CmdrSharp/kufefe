use crate::CLIENT;
use anyhow::{anyhow, bail, Result};
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
    pub async fn get(&self, name: &str) -> Result<ClusterRole> {
        match self.api.get(name).await {
            Ok(o) => {
                let annotations =
                    o.metadata.annotations.as_ref().ok_or_else(|| {
                        anyhow!("Role lacks the annotation kufefe.io/role")
                    })?;

                if annotations.get("kufefe.io/role") != Some(&"true".to_string()) {
                    bail!("Role lacks the annotation kufefe.io/role");
                }

                Ok(o)
            }

            Err(e) => bail!(e),
        }
    }
}
