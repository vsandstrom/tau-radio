use chrono::Local;
use std::sync::Arc;

/// Formats the file name of output file, using current local datetime.
/// If no filename is given in cli arguments. default = `tau_[datetime].ogg`
pub fn format_filename(filename: Option<String>) -> Arc<String> {
  let now = Local::now().format("%d-%m-%Y_%H_%M_%S").to_string();
  Arc::new(
    filename
      .map(|f| format!("{f}.ogg"))
      .unwrap_or_else(|| format!("tau_{}.ogg", now)),
  )
}

#[derive(Clone)]
pub struct Headers {pub headers: Option<Vec<Vec<u8>>>}

impl Headers {
  pub fn prepare_headers(&mut self, buf: &Vec<Vec<u8>>) {
    self.headers = Some(vec![buf[0].to_vec(), buf[1].to_vec()]);
  }
}

pub fn validate_bos_and_tags(data: &[u8]) -> core::result::Result<&[u8], ()> {
  let n_segs = data[26] as usize;
  let offset = 27+n_segs;
  if data.len() < 27 + 8 { return Err(()) }
  if matches!(&data[offset..offset+8], b"OpusTags" | b"OpusHead") {
    return Ok(data);
  }
  Err(())
}
