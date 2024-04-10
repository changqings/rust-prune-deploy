mod local_k8s;
use kube::Client;
use local_k8s::deploy::exec_delete_unhealthy_deploy;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let deploy_labels = "app=httpbin";
    // "some_sendmsg_url" is for debug, if not this value, will send http request
    let webhook_url = "some_sendmsg_url";

    exec_delete_unhealthy_deploy(client, deploy_labels, webhook_url).await?;
    Ok(())
}
