#[allow(unused)]
use inline_colorization::*;
use std::path::Path;

pub fn print_started_session_msg(
  devname: String,
  path: &Path,
  no_rec: bool,
) {
  println!(
    "\
    \n{style_bold}{color_bright_yellow}Streaming from: \
    \t{style_reset}{color_bright_cyan}{}{color_reset} \
    ",
    devname
  );
  if !no_rec {
    println!(
      "{style_bold}{color_bright_yellow}Saving local copy to: \
      \t{style_reset}{color_bright_cyan}{}{color_reset}",
      path.display()
    );
  } else {
    println!(
      "{color_red}{style_bold}Local recording \
      is disabled.{style_reset}{color_reset}"
    );
  }
  println!("Press Ctrl+C to stop.");
}

pub fn print_connected_to_host(
  url: &String,
  port: &u16
) {
  println!(
    "\
    \n{style_bold}{color_bright_yellow}Streaming to: \
    \t{color_bright_cyan}http://{}:{}/tau.ogg{color_reset}",
    url, port
  );
}
