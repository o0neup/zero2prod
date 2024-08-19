use std::str::FromStr;

use secrecy::Secret;
use serde::{de, Deserialize};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;
use std::time::Duration;

use crate::domain::SubscriberEmail;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct DbOptions(pub PgConnectOptions);

#[derive(Debug, serde::Deserialize, Clone)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub timeout_milliseconds: u64,
    pub auth_token: Secret<String>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Settings {
    pub database_url: DbOptions,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

impl TryFrom<&str> for DbOptions {
    type Error = sqlx::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        PgConnectOptions::from_str(value).map(DbOptions)
    }
}

impl<'de> Deserialize<'de> for DbOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DbOptions::try_from(s.as_str()).map_err(de::Error::custom)
    }
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_milliseconds)
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            other => Err(format!(
                "{} is not a supported environment. Use 'local' or 'production'.",
                other
            )),
        }
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current dir");
    let configuration_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .add_source(config::Environment::default().separator("__"))
        .build()?;
    settings.try_deserialize::<Settings>()
}
