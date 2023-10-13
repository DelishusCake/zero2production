use std::env;
use std::path::Path;

use anyhow::Context;

use config::{Config, Environment, File};

use secrecy::Secret;

use serde::Deserialize;

use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(Debug)]
pub enum Runtime {
    Dev,
    Prod,
}

impl Runtime {
    pub fn as_str(&self) -> &str {
        match self {
            Runtime::Dev => "dev",
            Runtime::Prod => "prod",
        }
    }
}

impl TryFrom<String> for Runtime {
    type Error = anyhow::Error;

    fn try_from(s: String) -> anyhow::Result<Self> {
        match s.to_lowercase().as_str() {
            "dev" => Ok(Self::Dev),
            "prod" => Ok(Self::Prod),
            other => anyhow::bail!("{} is not a valid runtime environment", other),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub app: ApplicationSettings,
    pub database: DatabaseSettings,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        let path = env::current_dir()?.join("settings");

        let runtime: Runtime = env::var("APP_ENV")
            .unwrap_or_else(|_| "dev".into())
            .try_into()?;

        Self::load_from(runtime, &path)
    }

    pub fn load_from(runtime: Runtime, base_path: &Path) -> anyhow::Result<Self> {
        Config::builder()
            .add_source(File::from(base_path.join("base")).required(true))
            .add_source(File::from(base_path.join(runtime.as_str())).required(true))
            .add_source(
                Environment::with_prefix("app")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?
            .try_deserialize()
            .context("Failed to load/deserialize settings")
    }
}

#[derive(Debug, Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
    pub secret_key: Secret<String>,
}

impl ApplicationSettings {
    pub fn addr(&self) -> (&str, u16) {
        (&self.host, self.port)
    }
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub port: u16,
    pub host: String,
    pub name: String,
    pub username: String,
    pub password: Secret<String>,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        use secrecy::ExposeSecret;

        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .port(self.port)
            .host(&self.host)
            .ssl_mode(ssl_mode)
            .username(&self.username)
            .password(self.password.expose_secret())
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.name)
    }
}
