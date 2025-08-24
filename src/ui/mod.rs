#[allow(unused)]
use inline_colorization::*;
use std::sync::Arc;

pub fn print_started_session_msg(devname: String, url: &String, port: &u16, filename: &Arc<String>, no_rec: bool) {
  println!("\
    \n{style_bold}{color_bright_yellow}Recording from: \t{style_reset}{color_bright_cyan}{}{color_reset} \
    \n{style_bold}{color_bright_yellow}Streaming live to: \t{color_bright_cyan}http://{}:{}/tau.ogg{color_reset}", 
    devname, url, port
  );
  if !no_rec { 
    println!("{style_bold}{color_bright_yellow}Saving local copy to: \t{style_reset}{color_bright_cyan}{}{color_reset}", filename); 
  } else { 
    println!("{color_red}{style_bold}Local recording is disabled.{style_reset}{color_reset}"); 
  }
  println!("Press Ctrl+C to stop.");
}
