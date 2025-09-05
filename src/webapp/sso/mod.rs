use axum::response::{IntoResponse, Redirect};
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
use serde::Deserialize;
use tracing::info;

use super::WebappError;

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

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CallbackParams {
    code: String,
    state: String,
}

fn always_verify_nonce(_nonce: Option<&Nonce>) -> Result<(), String> {
    Ok(())
}

async fn process_callback(
    params: CallbackParams,
    jar: PrivateCookieJar,
    client: &OauthClient,
) -> Result<(PrivateCookieJar, axum::http::Response<axum::body::Body>), WebappError> {
    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("HTTP Client should build");

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

    let mut updated_jar = jar.add(Cookie::build(("user", email.to_string())).path("/"));

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
