use axum::{extract::State, response::Html};
use minijinja::{Environment, context};

use super::WebappError;

#[axum::debug_handler]
pub async fn get_index<'a>(
    State(env): State<Environment<'a>>,
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
        // welcome_text => "Congrats! Hypermedia!",
        // df_values => DataFrameValues::from_df(&df),
        // chart_url => "/chart",
    })?;

    Ok(Html(rendered))
}
