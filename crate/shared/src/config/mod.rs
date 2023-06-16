pub mod path;

pub fn get_config<'a, T: serde::Deserialize<'a>>() -> Result<Box<T>, configure::ConfigError> {
    let configuration_path = path::workspace()
        .join("configuration")
        .join("application.ron");

    let settings = configure::Config::builder()
        .add_source(configure::File::from(configuration_path))
        .build()?;

    settings.try_deserialize::<Box<T>>()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Application {
    pub server: Server,
    pub channel: Channel,
    pub version: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Server {
    port: u16,
    address: [u8; 4],
    cors_allowed_origins: [String; 5],
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Channel {
    mpsc_size: usize,
    bordcast_size: usize,
}

#[cfg(test)]
mod tests {

    #[test]
    fn workspace_path() {
        let path = super::path::workspace();
        println!("{path:?}");
    }

    #[test]
    fn get_config() {
        let path = super::get_config::<super::Application>();
        println!("{path:?}");
    }

    #[test]
    fn ai_data_path() {
        let path = super::path::data();
        println!("{path:?}");
    }
}
