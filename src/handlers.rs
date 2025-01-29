use std::collections::HashMap;

use axum::{
    extract::{Form, Path, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::Host;
use eyre::WrapErr;

use crate::{error::AppError, send_mail::send_mail, state::AppState};

fn build_forward_to_url(
    config_forward_to: Option<url::Url>,
    referrer_url: url::Url,
    form: &HashMap<String, String>,
) -> url::Url {
    let mut forward_to = config_forward_to.unwrap_or(referrer_url);
    let mut query = HashMap::new();

    for (key, value) in forward_to.query_pairs() {
        query.insert(key, value);
    }

    if let Some(name) = form.get("name") {
        query.insert("name".into(), name.into());
    }

    if let Some(email) = form.get("email") {
        query.insert("email".into(), email.into());
    }

    query.insert("success".into(), "true".into());

    let query_string = query
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    forward_to.set_query(Some(&query_string));
    forward_to
}

#[axum::debug_handler]
#[tracing::instrument(skip(app_state, headers))]
pub async fn handle_form(
    State(app_state): State<AppState>,
    Path(template_name): Path<String>,
    Host(host): Host,
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
        .and_then(|r| r.to_str().ok())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| format!("https://{}/", host));

    let referrer_url = url::Url::parse(&referrer).wrap_err("Could not parse referrer")?;

    let forward_to = build_forward_to_url(
        app_state.config().forward_to().to_owned(),
        referrer_url,
        &form,
    );

    Ok(Redirect::to(forward_to.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_forward_to_url() {
        let config_forward_to = Some(url::Url::parse("https://config.com/").unwrap());
        let referrer_url = url::Url::parse("https://referrer.com/").unwrap();
        let form = HashMap::from([("name".to_string(), "John".to_string())]);

        let forward_to = build_forward_to_url(config_forward_to, referrer_url, &form);
        assert_eq!(forward_to.host_str(), Some("config.com"));

        let expected_pairs = HashMap::from_iter([
            ("name".into(), "John".into()),
            ("success".into(), "true".into()),
        ]);
        assert_eq!(
            forward_to.query_pairs().collect::<HashMap<_, _>>(),
            expected_pairs
        );
    }

    #[test]
    fn test_build_forward_to_url_no_forward_to() {
        let config_forward_to = None;
        let referrer_url = url::Url::parse("https://referrer.com/").unwrap();
        let form = HashMap::from([("name".to_string(), "John".to_string())]);

        let forward_to = build_forward_to_url(config_forward_to, referrer_url, &form);
        assert_eq!(forward_to.host_str(), Some("referrer.com"));
    }

    #[test]
    fn test_build_forward_to_url_with_existing_query_params() {
        let config_forward_to =
            Some(url::Url::parse("https://config.com/?existing=param").unwrap());
        let referrer_url = url::Url::parse("https://referrer.com/").unwrap();
        let form = HashMap::from([
            ("name".to_string(), "John".to_string()),
            ("email".to_string(), "john@example.com".to_string()),
        ]);

        let forward_to = build_forward_to_url(config_forward_to, referrer_url, &form);

        let expected_pairs = HashMap::from_iter([
            ("existing".into(), "param".into()),
            ("name".into(), "John".into()),
            ("email".into(), "john@example.com".into()),
            ("success".into(), "true".into()),
        ]);
        assert_eq!(
            forward_to.query_pairs().collect::<HashMap<_, _>>(),
            expected_pairs
        );
    }

    #[test]
    fn test_build_forward_to_url_with_empty_form() {
        let config_forward_to = Some(url::Url::parse("https://config.com/path").unwrap());
        let referrer_url = url::Url::parse("https://referrer.com/").unwrap();
        let form = HashMap::new();

        let forward_to = build_forward_to_url(config_forward_to, referrer_url, &form);

        assert_eq!(forward_to.path(), "/path");
        let expected_pairs = HashMap::from_iter([("success".into(), "true".into())]);
        assert_eq!(
            forward_to.query_pairs().collect::<HashMap<_, _>>(),
            expected_pairs
        );
    }
}
