use std::{ops::Deref, sync::Arc};

use derive_getters::Getters;
use eyre::{Result, WrapErr};
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use minijinja::Environment;

mod init;

use crate::config::Config;

#[derive(Clone, Getters)]
pub struct AppStateInner {
    config: Config,
    templates: Environment<'static>,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl AppStateInner {
    async fn new(config: Config) -> Result<Self> {
        let templates = init::read_templates(&config)
            .await
            .wrap_err("Failed to init templates")?;

        let mailer = init::init_mailer(&config).await?;

        Ok(Self {
            config,
            templates,
            mailer,
        })
    }
}

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        let inner = AppStateInner::new(config).await?;
        Ok(Self(Arc::new(inner)))
    }
}
