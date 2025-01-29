use std::{ops::Deref, path::PathBuf, sync::Arc};

use derive_getters::Getters;
use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize, Getters)]
pub struct ConfigInner {
    bind_to: String,
    template_dir: PathBuf,
    smtp_connection_string: String,
    smtp_from: String,
    smtp_to: String,
    forward_to: Option<Url>,
}

#[derive(Clone)]
pub struct Config(Arc<ConfigInner>);

impl Deref for Config {
    type Target = ConfigInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let config: ConfigInner = envy::from_env().wrap_err("Failed to parse config from env")?;

        Ok(Self(Arc::new(config)))
    }
}
