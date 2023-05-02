use crate::crd::RequestStatus;
use crate::resources::{rolebinding, serviceaccount, token};
use crate::traits::{expire::Expire, meta::Meta};
use crate::{crd::Request, kubeconfig::Kubeconfig, CLIENT, NAMESPACE};
use anyhow::Result;
use futures::TryStreamExt;
use kube::api::ListParams;
use kube::runtime::watcher::Event::*;
use kube::{api::Api, runtime::watcher, ResourceExt};

/// Starts the controller which watches for CRD Creation/Modification
pub async fn watch() {
    tracing::info!(
        "Starting watcher for resource creation/deletion in Kubernetes namespace {}",
        NAMESPACE.get().unwrap().clone()
    );

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
                    Applied(mut a) => {
                        if a.status.is_none() {
                            tracing::info!("Processing Addition: {}", a.name_any());

                            if let Err(e) = added(a.clone()).await {
                                let status = RequestStatus::new(&a).await;
                                a.status = Some(status);

                                a.message(e.to_string())
                                    .failed(true)
                                    .update_status()
                                    .await
                                    .ok();

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
async fn added(mut resource: Request) -> Result<()> {
    let sa = serviceaccount::ServiceAccount::new();
    let rb = rolebinding::RoleBinding::new();
    let tk = token::Token::new();

    // Generate the object name, expiry time and update CRD status
    let expire_at = resource.generate_expiry();
    let sa_name = sa.generate_name().await;
    let rb_name = rb.generate_name().await;
    let tk_name = tk.generate_name().await;

    // Set status
    resource
        .account_names(sa_name.clone(), tk_name.clone(), rb_name.clone())
        .expires_at(expire_at)
        .ready(false)
        .failed(false)
        .message("Generated names for resources".to_string())
        .update_status()
        .await?;

    // Create the Service Account
    let service_account = sa.create(sa_name.clone(), &resource).await?;

    // Create the SA Token
    let token = token::Token::new()
        .create(tk_name.clone(), &service_account)
        .await?;

    // Create the RoleBinding
    rb.create(
        rb_name.clone(),
        resource.spec.role.clone(),
        &service_account,
        &resource,
    )
    .await?;

    // Create the Kubeconfig and update the CRD Status
    let kubeconfig = Kubeconfig::new(service_account, token).await?.to_yaml()?;

    resource
        .ready(true)
        .kubeconfig(&kubeconfig)
        .message("Completed".to_string())
        .update_status()
        .await?;

    Ok(())
}
