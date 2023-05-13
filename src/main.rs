use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::Metadata;
use kube::api::{Api, DeleteParams, ListParams};
use kube::Client;

use reqwest::Client as reqClient;

use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kc: Client = Client::try_default().await?;
    let ns_all: &str = "";
    let addr: String = String::from("some_addr");

    let deployments: Api<Deployment> = Api::namespaced(kc.clone(), ns_all);

    let list_params: ListParams = ListParams::default().labels("cicd_env=canary");
    let dp_list = deployments.list(&list_params).await?;

    for deploy in dp_list {
        if check_deploy_status(&deploy) != "True".to_string() {
            // delete_deploy(kc.clone(), deploy).unwrap_or_else(|f| println!("delete deploy error {}",f)).await;
            let err = delete_deploy(kc.clone(), &deploy).await;
            send_wechat_msg(&deploy, addr.clone(), err).await?;
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

async fn send_wechat_msg(
    d: &Deployment,
    addr: String,
    err: Result<(), kube::Error>,
) -> Result<(), reqwest::Error> {
    let name = d.metadata.name.as_ref().unwrap();
    let ns = d.metadata.namespace.as_ref().unwrap();

    let (post_body, err_json) = match err {
        Ok(_) => (
            format!(
                r#"{{
            "msgtype": "text",
            "text": {{
                "content": format!("生产集群-灰度部署-启动异常定时清理 deployment={}/{},已成功删除"),
            }}
        }}"#,
                name, ns
            ),
            None,
        ),
        Err(e) => (
            format!(
                r#"{{
            "msgtype": "text",
            "text": {{
                "content": format!("生产集群-灰度部署-启动异常定时清理 deployment={}/{},删除失败，请手动处理"),
            }}
        }}"#,
                name, ns
            ),
            Some(e),
        ),
    };

    if let Some(e) = err_json {
        eprintln!("Error: prune deploy {}.{} error:{:?}", name, ns, e);
    };

    let reqc = reqClient::new();

    let resp = reqc
        .post(addr)
        .header("Content-Type", "aplication/json")
        .body(post_body)
        .send()
        .await?;

    if resp.status().is_success() {
        println!("Request succeeded! Status: {}", resp.status());
        let json_data: Value = resp.json().await?;
        println!("Response JSON data: {}", json_data);
    } else {
        println!("Request failed! Status: {}", resp.status());
    }

    Ok(())
}

async fn delete_deploy(kc: Client, d: &Deployment) -> Result<(), kube::Error> {
    let name: &str = d.metadata().name.as_ref().unwrap();
    let ns: &str = d.metadata().namespace.as_ref().unwrap();

    let dc: Api<Deployment> = Api::namespaced(kc.clone(), ns);
    let dp: DeleteParams = DeleteParams::default();

    dc.delete(name, &dp)
        .await?
        .map_left(|o| println!("deleting {}.{} {:?} ", name, ns, o))
        .map_right(|s| println!("deleted {},{} {:?}", name, ns, s.status.unwrap()));

    Ok(())
}
