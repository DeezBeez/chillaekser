use std::{collections::HashMap, path::Path};

use anyhow::Context;
use tokio::{
    fs::{DirBuilder, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};

const CONFIG_PATH: &str = "./Config/";
const SETTINGS_FILE_NAME: &str = "Settings.json";
//const CHANNEL_FILE_NAME: &str = "Channels.json";

pub struct Config {}

impl Config {
    pub async fn create_settings_file() -> anyhow::Result<()> {
        // Settings Path
        let settings_path = format!("{}{}", CONFIG_PATH, SETTINGS_FILE_NAME);
        let settings_path = Path::new(&settings_path);
        if settings_path.exists() {
            println!("Settings file already exists!");
            return Ok(());
        }
        // Create Config Folder if it doesnt exist
        let path: &Path = Path::new(CONFIG_PATH);
        if !path.exists() {
            DirBuilder::new()
                .create(path)
                .await
                .context("Failed to create directory")?;
        }

        // Create Settings HashMap
        let mut settings: HashMap<String, String> = HashMap::new();
        settings.insert("token".to_string(), "NO_TOKEN".to_string());
        settings.insert(
            "create_channel_name".to_string(),
            "+ create channel".to_string(),
        );
        settings.insert(
            "create_channel_category".to_string(),
            "user channel".to_string(),
        );

        // Create Settings File
        let mut settings_file: File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(settings_path)
            .await
            .context("Failed to open/create settings file")?;
        let settings_string = serde_json::ser::to_string_pretty(&settings)?;
        settings_file.write_all(settings_string.as_bytes()).await?;
        settings_file.flush().await?;
        Ok(())
    }

    pub async fn get_token() -> anyhow::Result<String> {
        let key = "token";
        let v = match Config::get_setting(key).await {
            Ok(s) => s,
            Err(_) => {
                return Err(anyhow::Error::msg(format!(
                    "Failed to find '{}' in settings file",
                    key
                )))
            }
        };
        Ok(v)
    }

    pub async fn get_create_channel_name() -> anyhow::Result<String> {
        let key = "create_channel_name";
        let v = match Config::get_setting(key).await {
            Ok(s) => s,
            Err(_) => {
                return Err(anyhow::Error::msg(format!(
                    "Failed to find '{}' in settings file",
                    key
                )))
            }
        };
        Ok(v)
    }

    pub async fn get_create_channel_category() -> anyhow::Result<String> {
        let key = "create_channel_category";
        let v = match Config::get_setting(key).await {
            Ok(s) => s,
            Err(_) => {
                return Err(anyhow::Error::msg(format!(
                    "Failed to find '{}' in settings file",
                    key
                )))
            }
        };
        Ok(v)
    }

    async fn get_setting(search_for: &str) -> anyhow::Result<String> {
        let mut settings_file = Config::get_readable_settings_file().await?;
        let mut s = String::new();
        settings_file.read_to_string(&mut s).await?;
        let hm: HashMap<String, String> = serde_json::from_str(s.as_str())?;
        let setting = match hm.get(search_for) {
            Some(s) => s,
            None => {
                return Err(anyhow::Error::msg(
                    "Failed to find 'token' in settings file",
                ))
            }
        };
        Ok(setting.to_string())
    }

    async fn get_readable_settings_file() -> anyhow::Result<File> {
        // Settings Path
        let settings_path = format!("{}{}", CONFIG_PATH, SETTINGS_FILE_NAME);
        let settings_path = Path::new(&settings_path);
        if !settings_path.exists() {
            return Err(anyhow::Error::msg(
                "get_readable_settings_file: Settings file does not exists!",
            ));
        }
        let settings_file = OpenOptions::new()
            .read(true)
            .open(settings_path)
            .await
            .context("Failed to open settings file")?;
        Ok(settings_file)
    }
}
