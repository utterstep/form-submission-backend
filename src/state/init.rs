use eyre::{OptionExt, Result, WrapErr};
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use minijinja::{AutoEscape, Environment};
use tokio::fs;

use crate::config::Config;

pub async fn read_templates(config: &Config) -> Result<Environment<'static>> {
    let mut templates = Environment::new();
    let mut entries = fs::read_dir(config.template_dir())
        .await
        .wrap_err_with(|| {
            format!(
                "Failed to read template directory: {}",
                config.template_dir().display()
            )
        })?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .wrap_err("Failed to read entry from the template directory")?
    {
        let path = entry.path();
        let name = path
            .file_stem()
            .ok_or_eyre("Failed to get file stem from the template directory")?
            .to_str()
            .ok_or_eyre("Failed to convert file stem to string")?;
        let file_content = fs::read_to_string(&path)
            .await
            .wrap_err("Failed to read file content")?;

        templates.add_template_owned(name.to_owned(), file_content)?;
    }

    templates.set_auto_escape_callback(|_| AutoEscape::Html);

    Ok(templates)
}

pub async fn init_mailer(config: &Config) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::from_url(config.smtp_connection_string())
            .wrap_err("Error parsing smtp connection string")?
            .build();

    Ok(mailer)
}
