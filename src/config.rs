use dirs;
use failure::err_msg;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{stdin, Write};
use std::path::{Path, PathBuf};

pub const LOCALHOST: &str = "http://localhost:8888/callback";
const FILE_NAME: &str = "client.yml";
const CONFIG_DIR: &str = ".config";
const APP_CONFIG_DIR: &str = "spotify-tui";

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientConfig {
    pub client_id: String,
    pub client_secret: String,
    pub device_id: Option<String>,
}

impl ClientConfig {
    pub fn new() -> ClientConfig {
        ClientConfig {
            client_id: "".to_string(),
            client_secret: "".to_string(),
            device_id: None,
        }
    }

    fn get_or_build_path(&self) -> Result<(PathBuf), failure::Error> {
        match dirs::home_dir() {
            Some(home) => {
                let path = Path::new(&home);
                let home_config_dir = path.join(CONFIG_DIR);
                let app_config_dir = home_config_dir.join(APP_CONFIG_DIR);

                if !home_config_dir.exists() {
                    fs::create_dir(&home_config_dir)?;
                }

                if !app_config_dir.exists() {
                    fs::create_dir(&app_config_dir)?;
                }

                let config_file_path = &app_config_dir.join(FILE_NAME);

                Ok(config_file_path.to_path_buf())
            }
            None => Err(err_msg("No $HOME directory found for client config")),
        }
    }

    pub fn set_device_id(&mut self, device_id: String) -> Result<(), failure::Error> {
        let config_file_path = self.get_or_build_path()?;
        let config_string = fs::read_to_string(&config_file_path)?;
        let mut config_yml: ClientConfig = serde_yaml::from_str(&config_string)?;

        self.device_id = Some(device_id.clone());
        config_yml.device_id = Some(device_id);

        let new_config = serde_yaml::to_string(&config_yml)?;
        let mut config_file = fs::File::create(&config_file_path)?;
        write!(config_file, "{}", new_config)?;
        Ok(())
    }

    pub fn load_config(&mut self) -> Result<(), failure::Error> {
        let config_file_path = self.get_or_build_path()?;
        if config_file_path.exists() {
            let config_string = fs::read_to_string(&config_file_path)?;
            let config_yml: ClientConfig = serde_yaml::from_str(&config_string)?;

            self.client_id = config_yml.client_id;
            self.client_secret = config_yml.client_secret;
            self.device_id = config_yml.device_id;

            Ok(())
        } else {
            let banner = fs::read_to_string("banner.txt")?;
            println!("{}", banner);

            println!("Config will be saved to {}", config_file_path.display());

            println!("\nHow to get setup:\n");

            let instructions = [
               "Go to the Spotify dashboard - https://developer.spotify.com/dashboard/applications",
               "Click `Create a Client ID` and create an app",
               "Now click `Edit Settings`",
               &format!("Add `{}` to the Redirect URIs - you don't need anything running on localhost", LOCALHOST),
               "You are now ready to authenticate with Spotify!",
            ];

            let mut number = 1;
            for item in instructions.iter() {
                println!("  {}. {}", number, item);
                number += 1;
            }

            // TODO: Handle empty input?
            let mut client_id = String::new();
            println!("\nEnter your Client ID: ");
            stdin().read_line(&mut client_id)?;

            let mut client_secret = String::new();
            println!("\nEnter your Client Secret: ");
            stdin().read_line(&mut client_secret)?;

            let config_yml = ClientConfig {
                client_id: client_id.trim().to_string(),
                client_secret: client_secret.trim().to_string(),
                device_id: None,
            };

            let content_yml = serde_yaml::to_string(&config_yml)?;

            let mut new_config = fs::File::create(&config_file_path)?;
            write!(new_config, "{}", content_yml)?;

            self.client_id = config_yml.client_id;
            self.client_secret = config_yml.client_secret;
            self.device_id = config_yml.device_id;

            Ok(())
        }
    }
}