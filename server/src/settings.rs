use std::env;
use std::path::Path;

use anyhow::Context;

use config::{Config, File};

use secrecy::Secret;

use serde::Deserialize;

use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub app: ApplicationSettings,
    pub database: DatabaseSettings,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        let path = env::current_dir()?.join("settings");

        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        Config::builder()
            .add_source(File::from(path.join("dev")).required(true))
            .build()?
            .try_deserialize()
            .context("Failed to load/deserialize settings")
    }
}

#[derive(Debug, Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
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
