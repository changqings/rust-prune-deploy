use futures::TryFutureExt;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::Metadata;
use kube::api::{Api, DeleteParams, ListParams};
use kube::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kc: Client = Client::try_default().await?;
    let ns_all: &str = "";

    let deployments: Api<Deployment> = Api::namespaced(kc.clone(), ns_all);

    let list_params: ListParams = ListParams::default().labels("app=gpu-tmp");
    let dp_list = deployments.list(&list_params).await?;

    for deploy in dp_list {
            println!("handler {},{}", deploy.metadata().name.as_ref().unwrap(),deploy.metadata().namespace.as_ref().unwrap());
        if check_deploy_status(&deploy) != "True".to_string() {
            let dc = Api::namespaced(kc.clone(), deploy.metadata().namespace.as_ref().unwrap());
            delete_deploy(dc, deploy).unwrap_or_else(|f| println!("delete deploy error {}",f)).await;
        }
    }

    Ok(())
}

fn check_deploy_status(d: &Deployment) -> String {
    if let Some(k) = &d.status {
        if let Some(v) = &k.conditions {
            for m in v {
                if m.type_ != "Available" {
                    return m.status.clone();
                }
            }
        }
    }

    return "Notok".to_string();
}

async fn delete_deploy(dc:Api<Deployment>, d: Deployment) -> Result<(), Box<dyn std::error::Error>> {
    let dp: DeleteParams = DeleteParams::default();
    let name: &str = d.metadata().name.as_ref().unwrap();
    let ns: &str = d.metadata().namespace.as_ref().unwrap();

    dc.delete(name, &dp)
        .map_ok(|_o| println!("delete {}.{} ok", name, ns))
        .map_err(|e| { eprintln!("delete {},{} err {}", name, ns, e);e})
        .await?;

    Ok(())
}
