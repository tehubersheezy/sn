use crate::error::{Error, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_profile: Option<String>,
    #[serde(default)]
    pub profiles: BTreeMap<String, ProfileConfig>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileConfig {
    pub instance: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Credentials {
    #[serde(default)]
    pub profiles: BTreeMap<String, ProfileCredentials>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileCredentials {
    pub username: String,
    pub password: String,
}

/// Resolve the sn config directory via `directories::ProjectDirs`.
pub fn config_dir() -> Result<PathBuf> {
    ProjectDirs::from("", "", "sn")
        .map(|pd| pd.config_dir().to_path_buf())
        .ok_or_else(|| Error::Config("cannot resolve home directory for config".into()))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn credentials_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("credentials.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_end_in_expected_filenames() {
        let cfg = config_path().unwrap();
        assert_eq!(cfg.file_name().unwrap(), "config.toml");
        let cred = credentials_path().unwrap();
        assert_eq!(cred.file_name().unwrap(), "credentials.toml");
    }

    #[test]
    fn profiles_roundtrip_via_toml() {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "dev".into(),
            ProfileConfig {
                instance: "example.com".into(),
            },
        );
        let cfg = Config {
            default_profile: Some("dev".into()),
            profiles,
        };
        let s = toml::to_string(&cfg).unwrap();
        let parsed: Config = toml::from_str(&s).unwrap();
        assert_eq!(parsed, cfg);
    }

    #[test]
    fn credentials_roundtrip_via_toml() {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "dev".into(),
            ProfileCredentials {
                username: "u".into(),
                password: "p".into(),
            },
        );
        let cr = Credentials { profiles };
        let s = toml::to_string(&cr).unwrap();
        let parsed: Credentials = toml::from_str(&s).unwrap();
        assert_eq!(parsed, cr);
    }
}
