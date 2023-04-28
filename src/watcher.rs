use crate::meta::generate_expiry;
use crate::resources::{rolebinding, serviceaccount, token};
use crate::{
    crd::Request, crd::RequestStatus, kubeconfig::Kubeconfig, meta::generate_name, CLIENT,
};
use futures::TryStreamExt;
use kube::api::ListParams;
use kube::runtime::watcher::Event::*;
use kube::{api::Api, runtime::watcher, ResourceExt};
use std::error::Error;

/// Starts the controller which watches for CRD Creation/Modification
pub async fn watch() {
    tracing::info!("Starting watcher for resource creation/deletion");

    let client = CLIENT.get().unwrap().clone();
    let api: Api<Request> = Api::all(client);

    // Do an inital scan for previously created & unready CRD's
    if let Ok(list) = api.list(&ListParams::default()).await {
        for item in list {
            if let Some(status) = &item.status {
                if status.ready {
                    continue;
                }
            }

            tracing::info!("Processing Addition: {}", item.name_any());

            if let Err(e) = added(item).await {
                tracing::error!("{}", e);
            }
        }
    }

    // Start the watcher
    tokio::spawn({
        let api: Api<Request> = api.clone();

        async move {
            _watch(api).await;
        }
    });
}

/// Start the watcher for CRD Creation/Deletion
async fn _watch(api: Api<Request>) {
    if let Err(e) = watcher(api.clone(), watcher::Config::default())
        .try_for_each(|r| {
            tracing::debug!("Event: {:?}", r);

            async move {
                match r {
                    Applied(a) => {
                        if a.status.is_none() {
                            tracing::info!("Processing Addition: {}", a.name_any());

                            if let Err(e) = added(a).await {
                                tracing::error!("{}", e);
                            }
                        }

                        Ok(())
                    }
                    Deleted(d) => {
                        tracing::debug!("Resource deleted: {}", d.name_any());
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
        })
        .await
    {
        tracing::error!("Error during watch: {}", e);
    };
}

/// Handle new resource creation
async fn added(resource: Request) -> Result<(), Box<dyn Error>> {
    let sa = serviceaccount::ServiceAccount::new();
    let rb = rolebinding::RoleBinding::new();

    // Generate the object name, expiry time and update CRD status
    let name = generate_name();
    let expire_at = generate_expiry();
    RequestStatus {
        generated_name: name.clone(),
        kubeconfig: None,
        ready: false,
        expires_at: Some(expire_at),
    }
    .update(&resource, &name)
    .await?;

    // Create the Service Account
    sa.create(name.clone(), &resource).await?;

    // Create the SA Token
    token::Token::new().create(name.clone(), &resource).await?;

    // Create the RoleBinding
    rb.create(name.clone(), resource.spec.role.clone(), &resource)
        .await?;

    // Create the Kubeconfig and update the CRD Status
    let kubeconfig = Kubeconfig::new(name.clone()).await?.to_yaml()?;
    RequestStatus {
        generated_name: name.clone(),
        kubeconfig: Some(kubeconfig),
        ready: true,
        expires_at: Some(expire_at),
    }
    .update(&resource, &name)
    .await?;

    Ok(())
}
