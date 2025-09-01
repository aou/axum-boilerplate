use axum_boilerplate::webapp;

#[tokio::main]
async fn main() {
    webapp::run_server().await;
}
