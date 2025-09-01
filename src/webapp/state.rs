use std::{ops::Deref, sync::Arc};

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use minijinja::Environment;

// AppState shenanigans, because CookieJar
#[derive(Clone)]
pub struct AppState(pub Arc<InnerState>);

pub struct InnerState {
    pub env: Environment<'static>,
    // pub ms_oauth_client: MsOauthClient,
    pub key: Key,
}

// automatically get to InnerState
impl Deref for AppState {
    type Target = InnerState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.0.key.clone()
    }
}

// impl FromRef<AppState> for MsOauthClient {
//     fn from_ref(state: &AppState) -> Self {
//         state.0.ms_oauth_client.clone()
//     }
// }

impl FromRef<AppState> for Environment<'_> {
    fn from_ref(state: &AppState) -> Self {
        state.0.env.clone()
    }
}
