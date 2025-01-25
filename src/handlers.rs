use std::collections::HashMap;

use axum::{
    extract::{Form, Path, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect},
};
use eyre::WrapErr;

use crate::{error::AppError, send_mail::send_mail, state::AppState};

#[axum::debug_handler]
#[tracing::instrument(skip(app_state, form))]
pub async fn handle_form(
    State(app_state): State<AppState>,
    Path(template_name): Path<String>,
    headers: HeaderMap,
    Form(form): Form<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let template = app_state
        .templates()
        .get_template(&template_name)
        .wrap_err_with(|| format!("Could not fetch template: {}", template_name))?;
    let rendered = template
        .render(&form)
        .wrap_err_with(|| format!("Could not render template: {}", template_name))?;

    send_mail(
        app_state.mailer(),
        app_state.config().smtp_to(),
        app_state.config().smtp_from(),
        "New Lead",
        &rendered,
    )
    .await
    .wrap_err("Failed to send email")?;

    let referrer = headers
        .get("Referer")
        .map(|r| r.to_str().unwrap_or("/"))
        .unwrap_or("/");

    Ok(Redirect::to(referrer))
}
