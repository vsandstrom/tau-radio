mod prompts;
use dialoguer::{Input, Password};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
// use inline_colorization::*;

use crate::{
  args::{
    validate_port,
    validate_url_or_ip
  },
};

pub enum UrlOrIp {
  Url(String),
  Ip(String)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub url: String,
    pub port: u16,
    pub audio_interface: String,
    pub file: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TauConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("invalid IP: {0}")]
    InvalidIp(String),

    #[error("invalid port number: {0}")]
    InvalidPort(u16),

    #[error("user input error: {0}")]
    Input(String),
}

impl Config {
  fn get_config_path() -> PathBuf {
    let local_dir = PathBuf::new().join("tau").join("config.toml");
    match (std::env::var("XDG_CONFIG_HOME"), std::env::var("HOME")) {
      // XDG_CONFIG_HOME
      (Ok(path), _) => PathBuf::from(path).join(local_dir),
      // HOME
      (_, Ok(path)) => PathBuf::from(path).join(".config").join(local_dir),
      // Fallback
      _ => PathBuf::from("config.toml"),
    }
  }

  /// Merges local config.toml with current CLI arguments if there are any.
  pub fn merge_cli_args(mut self, args: &crate::args::Args) -> Self {
    if let Some(un) = &args.username {self.username = un.to_string()}
    if let Some(pw) = &args.password {self.password = pw.to_string()}
    if let Some(u)  = &args.url      {self.url      = u.to_string()}
    if let Some(p)      = args.port      {self.port     = p}
    if let Some(f)  = &args.file     {self.file     = Some(f.to_string())}
    self
  }

  fn load_config(path: &PathBuf) -> Result<Config, TauConfigError> {
    let settings = fs::read_to_string(path)?; //.expect("could not read config file");
    match toml::from_str(&settings) {
      Ok(config) => Ok(config),
      Err(e) => Err(TauConfigError::Toml(e)),
    }
  }

  /// Creates an instance of Config, and reads from the saved `config.toml` file stored on disc.
  /// If no `config.toml` file can be found, it prompts the user to enter one.
  pub fn load_or_create(reset: bool) -> Result<Config, TauConfigError> {
    let path = Self::get_config_path();
    if path.exists() && !reset {
      Self::load_config(&path)
    } else {
      prompts::config_not_found(&path);
      prompts::warn_about_credentials();
      let username: String = Input::new()
        .with_prompt(prompts::username_prompt())
        .interact_text()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;

      let password: String = Password::new()
        .with_prompt(prompts::password_prompt())
        .interact()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;
      
      let url: String = Input::new()
        .with_prompt(prompts::ip_prompt())
        .default("127.0.0.1".to_string())
        .interact_text()
        .map_err(|e| TauConfigError::Input(e.to_string()))
        .and_then(validate_url_or_ip)?;

      let port: u16 = Input::new()
        .with_prompt(prompts::port_prompt())
        .default(8000)
        .interact_text()
        .map_err(|e| TauConfigError::Input(e.to_string()))
        .and_then(validate_port)?;

      let audio_interface = Input::new()
        .with_prompt(prompts::audio_interface_prompt())
        .default(crate::DEFAULT_INPUT.to_string())
        .interact_text()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;

      let file: String = Input::new()
        .with_prompt(prompts::filename_prompt())
        .allow_empty(true)
        .interact()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;

      let config = Config {
        username,
        password,
        url,
        port,
        audio_interface,
        file: if file.trim().is_empty() { None } 
              else { Some(file) },
      };

      if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
      }

      let toml_string = toml::to_string_pretty(&config).unwrap();
      fs::write(&path, toml_string)?;
      prompts::config_created(&path);
      Ok(config)
    }
  }
}

pub struct Credentials {
  username: String,
  password: String,
}

impl Credentials {
  pub fn new(username: String, password: String) -> Self {
    Self { username, password }
  }
  pub fn get_username(&self) -> String { self.username.clone() }
  pub fn get_password(&self) -> String { self.password.clone() }
}
