use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Secure {
    pub key: PathBuf,
    pub cert: PathBuf,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileServer {
    pub file_path: PathBuf,
    pub route_path: String,
    pub fallback_file: PathBuf,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    pub route: String,
    pub target: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub port: u16,
    pub https_config: Option<Secure>,
    pub file_server: Option<FileServer>,
    pub proxies: Option<Vec<Proxy>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

impl Config {
    pub fn from_file(path: &PathBuf) -> Self {
        let config = std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Unable to read config file {:?}", path));
        let mut config = Config::from_str(&config);

        if let Some(parent) = path.parent().map(|parent| parent.to_path_buf()) {
            if let Some(https_config) = &mut config.https_config {
                https_config.key = parent.join(&https_config.key);
                https_config.cert = parent.join(&https_config.cert);
            }

            if let Some(file_server) = &mut config.file_server {
                file_server.file_path = parent.join(&file_server.file_path);
                file_server.fallback_file = parent.join(&file_server.fallback_file);
            }
        }

        config
    }
    pub fn from_str(config: &str) -> Self {
        serde_json::from_str(config)
            .unwrap_or_else(|_| panic!("Unable to parse config\n{:?}", config))
    }
}
