#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Debug, serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

// get configuration
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialise our config reader
    let mut settings = config::Config::default();

    // Add config values from a file name configuration
    // It will look for any top level file with an extension
    // That config knows how to patse: yaml, json etc
    settings.merge(config::File::with_name("configuration"))?;

    //Try to convert tge config values
    settings.try_into()
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}
