use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};
#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}
#[derive(Debug, serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    // Demand  if we want the connection to be encrypted
    pub require_ssl: bool,
}

// get configuration
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialise our config reader
    let mut settings = config::Config::default();

    //
    let base_path = std::env::current_dir().expect("Failed to determine the current directroy");
    //
    let configuration_directory = base_path.join("configuration");

    // Add config values from a file name configuration
    // It will look for any top level file with an extension
    // That config knows how to paste: yaml, json etc
    settings.merge(config::File::from(configuration_directory.join("base")).required(true))?;
    // Detect the running environemnt
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONEMNT");

    // Layer on env-specific values
    settings.merge(
        config::File::from(configuration_directory.join(environment.as_str())).required(true),
    )?;

    // Add in settings from environment variables (with a prefix of APP and "__" as a separator)
    // E.g "APP_APPLICATION_PORT =5001 would set `Settings.application.port`"
    settings.merge(config::Environment::with_prefix("app").separator("__"))?;
    //Try to convert the config values
    settings.try_into()
}

// Possible runtime environment for our application
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

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not supported environment, Use either 'local' or 'production'.",
                other
            )),
        }
    }
}

impl DatabaseSettings {
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.database_name)
            .log_statements(log::LevelFilter::Trace)
            .to_owned()
    }

    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .username(&self.username)
            .password(&self.password)
            .port(self.port)
            .host(&self.host)
            .ssl_mode(ssl_mode)
    }
}
