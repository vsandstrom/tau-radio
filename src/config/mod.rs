use std::{path::PathBuf, process::exit, fs};
use dialoguer::{Input, Password};
use serde::{Serialize, Deserialize};
use is_ip::is_ip;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
  pub username: String,
  pub password: String,
  pub url: String,
  pub port: u16,
  pub mount: String,
  pub audio_interface: String,
  pub file: Option<String>,
  pub no_recording: bool
}


#[derive(Debug, thiserror::Error)]
pub enum TauConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Invalid IP address: {0}")]
    InvalidIp(String),

    #[error("Invalid port number: {0}")]
    InvalidPort(u16),

    #[error("User input error: {0}")]
    Input(String),
}

impl Config {
  fn get_config_path() -> PathBuf {
    let local_dir = PathBuf::new().join("tau").join("config.toml");
    match (std::env::var("XDG_CONFIG_HOME"), std::env::var("HOME")) {
      // XDG_HOME
      (Ok(path), Err(_)) => PathBuf::from(path).join(local_dir),
      // HOME
      (Err(_), Ok(path)) => PathBuf::from(path).join(".config").join(local_dir),
      // Fallback
      _ => PathBuf::from("config.toml")
    }
  }

  /// Merges local config.toml with current CLI arguments if there are any.
  pub fn merge_cli_args(mut self, args: crate::args::Args) -> Self {
    if let Some(username) = args.username { self.username = username; }
    if let Some(password) = args.password { self.password = password; }
    if let Some(url) = args.url { self.url = url; }
    if let Some(port) = args.port { self.port = port; }
    if let Some(mount) = args.mount { self.mount = mount; }
    if let Some(file) = args.file { self.file = Some(file); }
    if let Some(no_rec) = args.no_recording { self.no_recording = no_rec };
    self
  }

  fn load_config(path: &PathBuf) -> Result<Config, toml::de::Error> {
    let settings = fs::read_to_string(&path).expect("could not read config file");
    toml::from_str(&settings) //.expect("Invalid config format")
  }

  /// Creates an instance of Config, and reads from the saved `config.toml` file stored on disc.
  /// If no `config.toml` file can be found, it prompts the user to enter one. 
  pub fn load_or_create(reset: bool) -> Result<Config, TauConfigError> {
    let path = Self::get_config_path();
    if path.exists() && !reset {
      match Self::load_config(&path) {
        Ok(config) => Ok(config),
        Err(e) => Err(TauConfigError::Toml(e))
      }
    } else {
      println!("No config found at '{}'. Let's create one: ", path.display());
      println!("Credentials must correspond to what is set in icecast.xml");
      let username: String = Input::new()
        .with_prompt("Username")
        .interact_text()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;
      
      let password: String = Password::new()
        .with_prompt("Password")
        .interact()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;

      let url: String = Input::new()
        .with_prompt("Icecast URL")
        .default("127.0.0.1".into())
        .interact_text()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;
      if !is_ip(&url) { return Err(TauConfigError::InvalidIp(url)) }

      let port: u16 = Input::new()
        .with_prompt("Port")
        .default(8000)
        .interact_text()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;
      if !(1..=0xFFFF).contains(&port) { return Err(TauConfigError::InvalidPort(port)) }

      let mount = Input::new()
        .with_prompt("Icecast mount point")
        .default("tau.ogg".into())
        .interact_text()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;

      let audio_interface = Input::new()
        .with_prompt("Audio Interface")
        .default(crate::DEFAULT_INPUT.to_string())
        .interact_text()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;


      let file: String = Input::new()
        .with_prompt("Filename (leave empty for 'tau_[timestamp].ogg')")
        .allow_empty(true)
        .interact()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;

      let no_recording = Input::new()
        .with_prompt("Disable local recording (Disables filename)")
        .default(false)
        .interact_text()
        .map_err(|e| TauConfigError::Input(e.to_string()))?;

      let config = Config {
        username,
        password,
        url,
        port,
        mount,
        audio_interface,
        file: if file.trim().is_empty() {None} else { Some(file) },
        no_recording
      };

      if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?; //.expect("Could not create config directory")
      }

      let toml_string = toml::to_string_pretty(&config).unwrap();
      fs::write(&path, toml_string)?; //.expect("Failed to write config file");

      Ok(config)
    }
  }
}
