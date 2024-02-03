mod local_k8s;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    local_k8s::deploy::exec_delete_unhealthy_deploy()
}
