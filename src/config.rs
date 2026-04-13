/// Application configuration.
///
/// In v1 the relay URL is hardcoded per Constitution Principle II and FR-019.
pub struct Config {
    pub relay_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            relay_url: "wss://relay.mostro.network".to_string(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}
