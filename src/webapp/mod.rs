use std::{env, sync::Arc};

use axum::{
    Router, http::StatusCode, response::IntoResponse, routing::IntoMakeService, routing::get,
};
use axum_extra::extract::cookie::Key;
use minijinja::{Environment, context};
use rand::distr::{Alphanumeric, SampleString};
use state::{AppState, InnerState};
use tokio::net::TcpListener;
use tracing::info;

use axum::{extract::State, response::Html};

use crate::get_config;

mod handlers;
pub mod state;

#[derive(Debug, thiserror::Error)]
pub enum WebappError {
    #[error(transparent)]
    MinijinjaError(#[from] minijinja::Error),

    #[error("no id_token in token_response")]
    MissingIdToken,
}

impl IntoResponse for WebappError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("WebappError: {:#?}", self);
        println!("WebappError: {:#?}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
    }
}

pub async fn run_server() {
    tracing_subscriber::fmt::init();

    get_config();

    let env = add_templates();
    // let ms_oauth_client = oauth_client().await.unwrap();
    let secret = env::var("SECRET").unwrap_or_else(|_| {
        info!("no secret in env, generating...");
        Alphanumeric.sample_string(&mut rand::rng(), 64)
    });
    let key = Key::from(secret.as_bytes());

    let app_state = AppState(Arc::new(InnerState {
        env,
        // ms_oauth_client,
        key,
    }));

    let app = Router::new()
        .route("/", get(handlers::get_index))
        .with_state(app_state);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

fn add_templates<'a>() -> Environment<'a> {
    let mut env = Environment::new();

    env.add_template("layout", include_str!("./templates/layout.html"))
        .unwrap();
    env.add_template("home", include_str!("./templates/home.html"))
        .unwrap();

    env
}
