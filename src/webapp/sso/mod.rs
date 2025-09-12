use std::collections::HashMap;
use std::str::FromStr;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::Cookie;
use minijinja::context;
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
    CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreRevocableToken, CoreTokenType,
};
use openidconnect::{AuthenticationFlow, CsrfToken, Nonce, Scope, core::CoreResponseType, reqwest};
use openidconnect::{
    AuthorizationCode, Client, EmptyAdditionalClaims, EmptyExtraTokenFields, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IdTokenFields, RevocationErrorResponseType, StandardErrorResponse,
    StandardTokenIntrospectionResponse, StandardTokenResponse, TokenResponse,
};
use serde::Deserialize;
use tracing::info;
use url::Url;

use super::WebappError;
use super::handlers;
use super::state::AppState;

use crate::db;

pub mod google_sso;
pub mod microsoft_sso;

pub type OauthClient = Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    StandardTokenResponse<
        IdTokenFields<
            EmptyAdditionalClaims,
            EmptyExtraTokenFields,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
        >,
        CoreTokenType,
    >,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, CoreTokenType>,
    CoreRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

pub fn sso_router() -> Router<AppState> {
    let route = Router::new()
        .route("/{provider}/login", get(get_sso_login))
        .route("/{provider}/callback", get(get_sso_callback));

    route
}

async fn get_sso_login(
    Path(provider): Path<String>,
    State(client_map): State<HashMap<String, OauthClient>>,
    headers: HeaderMap,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, impl IntoResponse), WebappError> {
    let client = client_map
        .get(&provider)
        .ok_or_else(|| WebappError::MissingOauthClientError)?;

    let (authorize_url, _csrf_state, _nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .url();

    // persist next_url in cookie for sso flow
    let updated_jar = match headers
        .get("REFERER")
        .map(|x| x.to_str().ok())
        .unwrap_or_else(|| None)
        .map(|x| Url::from_str(x).ok())
        .unwrap_or_else(|| None)
    {
        Some(referer_url) => {
            let query_map: HashMap<String, String> =
                referer_url.query_pairs().into_owned().collect();

            match query_map.get("next_url") {
                Some(next_url) => jar.add(Cookie::build(("next_url", next_url.clone())).path("/")),
                None => jar,
            }
        }
        None => jar,
    };

    Ok((updated_jar, Redirect::to(authorize_url.as_str())))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CallbackParams {
    code: String,
    state: String,
}

fn always_verify_nonce(_nonce: Option<&Nonce>) -> Result<(), String> {
    Ok(())
}

async fn get_sso_callback(
    Query(params): Query<CallbackParams>,
    Path(provider): Path<String>,
    State(client_map): State<HashMap<String, OauthClient>>,
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, axum::http::Response<axum::body::Body>), WebappError> {
    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("HTTP Client should build");

    let client = client_map
        .get(&provider)
        .ok_or_else(|| WebappError::MissingOauthClientError)?;

    let token_response = client
        .exchange_code(AuthorizationCode::new(params.code.clone()))?
        .request_async(&http_client)
        .await?;

    let id_token = token_response
        .id_token()
        .ok_or(WebappError::MissingIdToken)?;

    let id_token_verifier = client.id_token_verifier();
    let claims = id_token.claims(&id_token_verifier, always_verify_nonce)?;

    let email = claims.email().ok_or(WebappError::MissingEmailError)?;
    info!("sso login email: {email:#?}");

    // println!("params: {:#?}", params);
    // println!("token_response: {:#?}", token_response);
    // println!("id_token: {:#?}", id_token);
    // println!("claims: {:#?}", claims);
    // println!("email: {}", email.as_str());

    let user = db::get_user_by_email(email);

    let Some(user) = user else {
        // return Err(WebappError::NoMatchingUserError);
        return Ok((
            jar,
            handlers::render_login_with_context(
                state,
                context! {
                    alert => "No registered user found."
                },
            )?,
        ));
    };

    let mut updated_jar = jar.add(Cookie::build(("user", user.username)).path("/"));

    if let Some(next_url) = updated_jar.get("next_url") {
        info!("next_url: {:#?}", next_url.value_trimmed());
        updated_jar = updated_jar.remove(Cookie::from("next_url"));
        return Ok((
            updated_jar,
            Redirect::to(next_url.value_trimmed()).into_response(),
        ));
    };

    Ok((
        updated_jar,
        Redirect::to("/").into_response().into_response(),
    ))
}
