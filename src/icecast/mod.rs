use shout::{ShoutConnBuilder, ShoutConn}; 
use crate::exit;
use crate::config::Config;

pub fn create_icecast_connection(config: Config) -> ShoutConn {
  match ShoutConnBuilder::new()
    .host(config.url.clone())
    .port(config.port)
    .user(config.username.clone())
    .password(config.password.clone())
    .mount(config.mount)
    .protocol(shout::ShoutProtocol::HTTP)
    .format(shout::ShoutFormat::Ogg)
    .build() {
      Ok(shout) => shout,
      Err(e) => {
        eprintln!("Could not connect to the IceCast host: {e:?}");
        exit(1); 
      }
    }
}
