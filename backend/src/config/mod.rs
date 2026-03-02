use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub global: GlobalConfig,
    pub apps: AppsConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalConfig {
    #[serde(rename = "web-url")]
    pub web_url: String,
    #[serde(rename = "jwt-secret", default = "default_jwt_secret")]
    pub jwt_secret: String,
}

fn default_jwt_secret() -> String {
    "change-me-in-production".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppsConfig {
    #[serde(default)]
    pub medias: Vec<String>,
    #[serde(default)]
    pub documents: Vec<String>,
    #[serde(default)]
    pub videos: Vec<String>,
    #[serde(default)]
    pub music: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_host")]
    pub host: String,
    #[serde(default = "default_db_port")]
    pub port: u16,
    #[serde(default = "default_db_name")]
    pub name: String,
    #[serde(default = "default_db_user")]
    pub user: String,
    #[serde(default)]
    pub password: String,
}

fn default_db_host() -> String {
    "localhost".to_string()
}
fn default_db_port() -> u16 {
    5432
}
fn default_db_name() -> String {
    "home_nas".to_string()
}
fn default_db_user() -> String {
    "postgres".to_string()
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: default_db_host(),
            port: default_db_port(),
            name: default_db_name(),
            user: default_db_user(),
            password: String::new(),
        }
    }
}

/// Parsed app path entry: name -> real_path
#[derive(Debug, Clone)]
pub struct AppPath {
    pub name: String,
    pub path: String,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.user,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.name
        )
    }

    /// Parse app paths from config format "name:path" or just "path"
    pub fn parse_app_paths(paths: &[String]) -> Vec<AppPath> {
        paths
            .iter()
            .map(|s| {
                if let Some((name, path)) = s.split_once(':') {
                    AppPath {
                        name: name.to_string(),
                        path: path.to_string(),
                    }
                } else {
                    // Extract last directory name as the name
                    let path = s.trim_end_matches('/');
                    let name = path.rsplit('/').next().unwrap_or(path);
                    AppPath {
                        name: name.to_string(),
                        path: s.to_string(),
                    }
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_app_paths() {
        let paths = vec![
            "photos:/mnt/nas/photos".to_string(),
            "/mnt/nas/documents".to_string(),
        ];
        let parsed = Config::parse_app_paths(&paths);
        assert_eq!(parsed[0].name, "photos");
        assert_eq!(parsed[0].path, "/mnt/nas/photos");
        assert_eq!(parsed[1].name, "documents");
        assert_eq!(parsed[1].path, "/mnt/nas/documents");
    }
}
