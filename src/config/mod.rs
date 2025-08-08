use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::process::exit;
use std::fs;
use dialoguer::{Input, Password};


#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
  pub username: String,
  pub password: String,
  pub url: String,
  pub port: u16,
  pub file: Option<String>,
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

pub fn load_or_create_config() -> Config {
  let path = get_config_path();

  if path.exists() {
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

    let file: String = Input::new()
      .with_prompt("Filename (leave empty for 'tau_[timestamp].ogg')")
      .allow_empty(true)
      .interact()
      .unwrap();

    let config = Config {
      username,
      password,
      url,
      port,
      file: if file.trim().is_empty() {None} else { Some(file) }
    };

    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent).expect("Could not create config directory")
    }
    let toml_string = toml::to_string_pretty(&config).unwrap();
    fs::write(&path, toml_string).expect("Failed to write config file");

    config
  }
}
