use axum::{
    extract::{Query, State},
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
}

pub async fn get_login(
    params: Query<Params>,
    jar: PrivateCookieJar,
    State(state): State<AppState>,
) -> Result<(PrivateCookieJar, Response), WebappError> {
    // info!("{:#?}", jar);
    info!("{params:#?}");

    // you only get here if you manually go to url, so we don't worry about query params / next
    if let Some(user) = jar.get("user") {
        return Ok((jar, Redirect::to("/").into_response()));
    }

    let template = state.env.get_template("login")?;

    // couldn't get query params to work through the oidc flow...
    // let context = match &params.next_url {
    //     Some(next_url) => context! { params => "?next_url=".to_string() + next_url },
    //     None => minijinja::Value::UNDEFINED,
    // };

    let updated_jar = match &params.next_url {
        Some(next_url) => jar.add(Cookie::build(("next_url", next_url.to_string())).path("/")),
        None => jar.remove(Cookie::from("next_url")),
    };

    let rendered = template.render({})?;

    Ok((updated_jar, Html(rendered).into_response()))
}

pub async fn get_index(
    State(env): State<Environment<'static>>,
) -> Result<Html<String>, WebappError> {
    let template = env.get_template("home")?;

    // let df = get_df_from_db_table("vmware")?;

    // let df = df
    //     .lazy()
    //     .with_columns(vec![
    //         (col("notification")
    //             .dt()
    //             .quarter()
    //             .alias("notification_quarter")),
    //     ])
    //     .collect()?;

    let rendered = template.render(context! {
        title => "Home",
        content => "STUFF GOES HERE",
        // welcome_text => "Congrats! Hypermedia!",
        // df_values => DataFrameValues::from_df(&df),
        // chart_url => "/chart",
    })?;

    Ok(Html(rendered))
}
