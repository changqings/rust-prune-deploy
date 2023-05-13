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

    let list_params: ListParams = ListParams::default().labels("cicd_env=canary");
    let dp_list = deployments.list(&list_params).await?;

    for deploy in dp_list {
        if check_deploy_status(&deploy) != "True".to_string() {
            delete_deploy(kc.clone(), deploy).unwrap_or_else(|f| println!("delete deploy error {}",f)).await;
        }
    }

    Ok(())
}

fn check_deploy_status(d: &Deployment) -> String {
    if let Some(k) = &d.status {
        if let Some(v) = &k.conditions {
            for m in v {
                if m.type_ == "Available" {
                    return m.status.clone();
                }
            }
        }
    }

    return "Notok".to_string();
}

async fn delete_deploy(kc:Client , d: Deployment) -> Result<(), kube::Error> {
    let name: &str = d.metadata().name.as_ref().unwrap();
    let ns: &str = d.metadata().namespace.as_ref().unwrap();

    let dc: Api<Deployment> = Api::namespaced(kc.clone(),ns);
    let dp: DeleteParams = DeleteParams::default();

    dc.delete(name, &dp).await?
        .map_left(|o| println!("deleting {}.{} {:?} ", name, ns, o))
        .map_right(|s|  println!("deleted {},{} {:?}", name, ns,s.status.unwrap()));

    Ok(())
}
