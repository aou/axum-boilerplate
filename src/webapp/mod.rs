use std::{env, sync::Arc};

use axum_extra::extract::cookie::Key;
use minijinja::Environment;
use rand::distr::{Alphanumeric, SampleString};
use state::{AppState, InnerState};
use tracing::info;

use crate::get_config;

pub mod state;

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

    println!("running...");
}

fn add_templates<'a>() -> Environment<'a> {
    let mut env = Environment::new();

    env.add_template("home", include_str!("./templates/home.html"))
        .unwrap();

    env
}
