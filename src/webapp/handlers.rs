use std::{collections::HashMap, str::FromStr};

use axum::{
    Form,
    extract::{Query, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::{PrivateCookieJar, cookie::Cookie};
use bcrypt::bcrypt;
use minijinja::{Environment, context};
use serde::Deserialize;
use tracing::info;
use url::Url;
use validator::{Validate, ValidationErrorsKind};

use crate::db;

use super::{WebappError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct Params {
    next_url: Option<String>,
    alert: Option<bool>,
}

pub async fn get_login(
    params: Query<Params>,
    jar: PrivateCookieJar,
    State(state): State<AppState>,
) -> Result<(PrivateCookieJar, Response), WebappError> {
    // info!("{:#?}", jar);
    info!("{params:#?}");

    // you only get here if you manually go to url, so we don't worry about query params / next
    if let Some(_user) = jar.get("user") {
        return Ok((jar, Redirect::to("/").into_response()));
    }

    Ok((
        jar,
        render_login_with_context(state, minijinja::Value::UNDEFINED)?,
    ))
}

#[derive(Deserialize, Debug, Validate)]
pub struct LoginPayload {
    #[validate(length(min = 1, message = "Username cannot be blank."))]
    username: String,

    #[validate(length(min = 1, message = "Password cannot be blank"))]
    password: String,
}

pub async fn post_login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    headers: HeaderMap,
    Form(login_payload): Form<LoginPayload>,
) -> Result<(PrivateCookieJar, Response), WebappError> {
    info!("{login_payload:#?}");

    let validation = login_payload.validate();

    // if validation errors, render login with messages in alert
    if let Err(validation_errors) = validation {
        let errors = validation_errors.errors();
        let validation_messages: Vec<_> = errors
            .values()
            .filter_map(|x| match x {
                ValidationErrorsKind::Field(validation_errors) => Some(validation_errors),
                _ => None,
            })
            .flatten()
            .filter_map(|x| x.message.clone())
            .collect();
        let message = validation_messages.join("<br>");
        return Ok((
            jar,
            render_login_with_context(
                state,
                context! {
                    alert => message,
                },
            )?,
        ));
    }

    if let Some(user) = db::get_user_by_username(&login_payload.username) {
        // empty password means no password login
        if let Some(hashed_password) = user.hashed_password {
            if bcrypt::verify(login_payload.password, &hashed_password)
                .ok()
                .unwrap_or_else(|| false)
            {
                let updated_jar = jar.add(Cookie::build(("user", user.username)).path("/"));

                // get next_url from REFERER header
                let next_url = get_next_url_from_headers(headers);

                return Ok((updated_jar, Redirect::to(next_url.as_str()).into_response()));
            }
        }
    };

    Ok((jar, "login".into_response()))
}

pub fn old_get_next_url_from_headers(headers: HeaderMap) -> String {
    let next_url = if let Some(referer_url) = headers
        .get("REFERER")
        .map(|x| x.to_str().ok())
        .unwrap_or_else(|| None)
        .map(|x| Url::from_str(x).ok())
        .unwrap_or_else(|| None)
    {
        let query_map: HashMap<String, String> = referer_url.query_pairs().into_owned().collect();

        match query_map.get("next_url") {
            Some(next_url) => next_url.clone(),
            None => "/".to_string(),
        }
    } else {
        "/".to_string()
    };
    next_url
}

pub fn get_next_url_from_headers(headers: HeaderMap) -> String {
    let next_url = headers
        .get("REFERER")
        .and_then(|x| x.to_str().ok())
        .and_then(|x| Url::from_str(x).ok())
        .and_then(|referer_url| {
            referer_url
                .query_pairs()
                .find_map(|(k, v)| (k == "next_url").then(|| v.into_owned()))
        })
        .unwrap_or_else(|| "/".to_string());
    next_url
}

pub fn render_login_with_context(
    state: AppState,
    context: minijinja::Value,
) -> Result<Response, minijinja::Error> {
    let template = state.env.get_template("login")?;

    let rendered = template.render(context)?;

    Ok(Html(rendered).into_response())
}

pub async fn get_logout(
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Response), WebappError> {
    let updated_jar = jar.remove(Cookie::from("user"));
    Ok((updated_jar, Redirect::to("/").into_response()))
}

pub async fn get_index(
    State(env): State<Environment<'static>>,
) -> Result<Html<String>, WebappError> {
    let template = env.get_template("home")?;

    let rendered = template.render(context! {
        title => "Home",
        content => "STUFF GOES HERE",
        // welcome_text => "Congrats! Hypermedia!",
        // df_values => DataFrameValues::from_df(&df),
        // chart_url => "/chart",
    })?;

    Ok(Html(rendered))
}

// to be used as middleware
pub async fn check_auth(
    jar: PrivateCookieJar,
    request: Request,
    next: Next,
) -> Result<Response, WebappError> {
    if let Some(user) = jar.get("user") {
        info!("logged in user: {}", user);
    } else {
        let redirect_url = "/login?next_url=".to_string() + request.uri().to_string().as_str();
        return Ok((StatusCode::FOUND, Redirect::to(redirect_url.as_str())).into_response());
    }
    let response = next.run(request).await;

    Ok(response)
}
