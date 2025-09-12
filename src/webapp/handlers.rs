use axum::{
    Form,
    extract::{Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::{PrivateCookieJar, cookie::Cookie};
use minijinja::{Environment, context};
use serde::Deserialize;
use tracing::info;

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

    // couldn't get query params to work through the oidc flow...
    // let context = match &params.next_url {
    //     Some(next_url) => context! { params => "?next_url=".to_string() + next_url },
    //     None => minijinja::Value::UNDEFINED,
    // };

    let updated_jar = match &params.next_url {
        Some(next_url) => jar.add(Cookie::build(("next_url", next_url.to_string())).path("/")),
        None => jar.remove(Cookie::from("next_url")),
    };

    Ok((
        updated_jar,
        render_login_with_context(state, minijinja::Value::UNDEFINED)?,
    ))
}

#[derive(Deserialize, Debug)]
pub struct LoginPayload {
    username: String,
    password: String,
}

pub async fn post_login(Form(login_payload): Form<LoginPayload>) -> Result<Response, WebappError> {
    info!("{login_payload:#?}");
    Ok("login".into_response())
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
