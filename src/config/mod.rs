use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;
use dialoguer::{Input, Password};
use std::process::exit;
use is_ip::is_ip;
use crate::util::validate_port;



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
  pub username: String,
  pub password: String,
  pub url: String,
  pub port: u16,
  pub audio_interface: String,
  pub file: Option<String>,
  pub no_recording: bool
}

impl Config {
  fn get_config_path() -> PathBuf {
    if let Ok(xdg_path) = std::env::var("XDG_CONFIG_HOME") {
      return PathBuf::from(xdg_path).join("tau").join("config.toml");
    }
    
    if let Ok(home_path) = std::env::var("HOME") {
      return PathBuf::from(home_path).join(".config").join("tau").join("config.toml");
    }

    PathBuf::from("config.toml")
  }

  /// Merges local config.toml with current CLI arguments if there are any.
  pub fn merge_cli_args(mut self, args: crate::args::Args) -> Self {
    if let Some(username) = args.username { self.username = username; }
    if let Some(password) = args.password { self.password = password; }
    if let Some(url) = args.url { self.url = url; }
    if let Some(port) = args.port { self.port = port; }
    if let Some(file) = args.file { self.file = Some(file); }
    if let Some(no_rec) = args.no_recording { self.no_recording = no_rec };
    self
  }

  /// Creates an instance of Config, and reads from the saved `config.toml` file stored on disc.
  /// If no `config.toml` file can be found, it prompts the user to enter one. 
  pub fn load_or_create(reset: bool) -> Config {
    let path = Self::get_config_path();
    if path.exists() && !reset {
      let settings = fs::read_to_string(&path).expect("could not read config file");
      toml::from_str(&settings).expect("Invalid config format")
    } else {
      println!("No config found at '{}'. Let's create one: ", path.display());
      let username: String = Input::new()
        .with_prompt("Username")
        .interact_text()
        .unwrap(); 
      
      let password: String = Password::new()
        .with_prompt("Password")
        .interact()
        .unwrap(); 

      let url: String = Input::new()
        .with_prompt("Icecast URL")
        .default("127.0.0.1".into())
        .interact_text().inspect(|a: &String| {
          if !is_ip(a) {
            eprintln!("IP is not valid");
            exit(1);
          }
        }).unwrap();

      let port: u16 = Input::new()
        .with_prompt("Port")
        .default(8000)
        .interact_text().inspect(|x| {
          if !(1..=0xFFFF).contains(x) {
            eprintln!("Port is not within valid range: 1 - 65535");
            exit(1);
          }
        }).unwrap();

      let audio_interface = Input::new()
        .with_prompt("Audio Interface")
        .default(crate::DEFAULT_INPUT.to_string())
        .interact_text()
        .unwrap();


      let file: String = Input::new()
        .with_prompt("Filename (leave empty for 'tau_[timestamp].ogg')")
        .allow_empty(true)
        .interact()
        .unwrap();

      let no_recording = Input::new()
        .with_prompt("Disable local recording (Disables filename)")
        .default(false)
        .interact_text()
        .unwrap();

      let config = Config {
        username,
        password,
        url,
        port,
        audio_interface,
        file: if file.trim().is_empty() {None} else { Some(file) },
        no_recording
      };

      if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Could not create config directory")
      }
      let toml_string = toml::to_string_pretty(&config).unwrap();
      fs::write(&path, toml_string).expect("Failed to write config file");

      config
    }
  }
}



pub fn load_or_create_config(reset: bool) -> Config {
  let path = get_config_path();

  if path.exists() && !reset {
    let settings = fs::read_to_string(&path).expect("could not read config file");
    toml::from_str(&settings).expect("Invalid config format")
  } else {
    println!("No config found at '{}'. Let's create one: ", path.display());
    let username: String = Input::new()
      .with_prompt("Username")
      .interact_text()
      .unwrap(); 
    
    let password: String = Password::new()
      .with_prompt("Password")
      .interact()
      .unwrap(); 

    let url: String = Input::new()
      .with_prompt("Icecast URL")
      .default("127.0.0.1".into())
      .interact_text()
      .unwrap();

    let port: u16 = Input::new()
      .with_prompt("Port")
      .default(8000)
      .interact_text()
      .unwrap();

    let audio_interface = Input::new()
      .with_prompt("Audio Interface")
      .default(crate::DEFAULT_INPUT.to_string())
      .interact_text()
      .unwrap();


    let file: String = Input::new()
      .with_prompt("Filename (leave empty for 'tau_[timestamp].ogg')")
      .allow_empty(true)
      .interact()
      .unwrap();

    let no_recording = Input::new()
      .with_prompt("Disable local recording (Disables filename)")
      .default(false)
      .interact_text()
      .unwrap();

    let config = Config {
      username,
      password,
      url,
      port,
      audio_interface,
      file: if file.trim().is_empty() {None} else { Some(file) },
      no_recording
    };

    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent).expect("Could not create config directory")
    }
    let toml_string = toml::to_string_pretty(&config).unwrap();
    fs::write(&path, toml_string).expect("Failed to write config file");

    config
  }
}

/// Merges local config.toml with current CLI arguments if there are any.
pub fn merge_cli_args(config: &mut Config, args: crate::args::Args) {
  if let Some(username) = args.username { config.username = username; }
  if let Some(password) = args.password { config.password = password; }
  if let Some(url) = args.url { config.url = url; }
  if let Some(port) = args.port { config.port = port; }
  if let Some(file) = args.file { config.file = Some(file); }
  if let Some(no_rec) = args.no_recording {config.no_recording = no_rec; }
}

fn get_config_path() -> PathBuf {
  if let Ok(xdg_path) = std::env::var("XDG_CONFIG_HOME") {
    return PathBuf::from(xdg_path).join("tau").join("config.toml");
  }
  
  if let Ok(home_path) = std::env::var("HOME") {
    return PathBuf::from(home_path).join(".config").join("tau").join("config.toml");
  }

  PathBuf::from("config.toml")
}

