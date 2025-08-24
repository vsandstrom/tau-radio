use shout::{ShoutConnBuilder, ShoutConn}; 
use crate::config::Config;

pub fn create_icecast_connection(config: Config) -> anyhow::Result<ShoutConn> {
  match ShoutConnBuilder::new()
    .host(config.url.clone())
    .port(config.port)
    .user(config.username.clone())
    .password(config.password.clone())
    .mount(config.mount)
    .protocol(shout::ShoutProtocol::HTTP)
    .format(shout::ShoutFormat::Ogg)
    .build() {
    Ok(shout) => Ok(shout),
    Err(e) => {
      Err(anyhow::anyhow!("Could not connect to the IceCast host: {e:?}"))
    }
  }
}
