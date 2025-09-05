use std::collections::HashMap;
use std::sync::Arc;

use std::{env, process::exit};

use axum::extract::Query;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::Cookie;
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
    CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreRevocableToken, CoreTokenType,
};
use openidconnect::{
    AuthDisplay, AuthPrompt, AuthorizationCode, AuthorizationRequest, Client,
    EmptyAdditionalClaims, EmptyExtraTokenFields, EndpointMaybeSet, EndpointNotSet, EndpointSet,
    IdTokenFields, NonceVerifier, ResponseType, RevocationErrorResponseType, StandardErrorResponse,
    StandardTokenIntrospectionResponse, StandardTokenResponse, TokenResponse,
};
use openidconnect::{
    AuthenticationFlow, ClientId, ClientSecret, CsrfToken, DiscoveryError, HttpClientError,
    IssuerUrl, Nonce, RedirectUrl, Scope,
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    reqwest,
    url::ParseError,
};

use axum::{
    Router,
    extract::State,
    response::{Html, Redirect},
    routing::get,
};
use serde::Deserialize;
use tracing::{debug, info};

use crate::webapp::WebappError;
use crate::webapp::state::AppState;

use super::{CallbackParams, OauthClient, get_sso_callback};

pub async fn oauth_client() -> Result<OauthClient, WebappError> {
    let client_id = ClientId::new(
        env::var("MICROSOFT_CLIENT_ID")
            .expect("Missing the MICROSOFT_CLIENT_ID environment variable."),
    );
    let client_secret = ClientSecret::new(
        env::var("MICROSOFT_CLIENT_SECRET")
            .expect("Missing the MICROSOFT_CLIENT_SECRET environment variable."),
    );
    let tenant_id = env::var("MICROSOFT_TENANT_ID").expect("Missing MICROSOFT_TENANT_ID");
    let redirect_url = env::var("MICROSOFT_REDIRECT_URL").expect("Missing MICROSOFT_REDIRECT_URL");
    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("HTTP Client should build");
    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(format!(
            "https://login.microsoftonline.com/{tenant_id}/v2.0"
        ))?,
        &http_client,
    )
    .await?;
    let client =
        CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret))
            .set_redirect_uri(RedirectUrl::new(redirect_url)?);

    Ok(client)
}
