mod args;
mod audio;
mod config;
mod err;
mod threads;
mod ui;
mod util;

use crate::args::Args;
use crate::config::Config;
use crate::err::AUDIO_INTERFACE_NOT_FOUND;

use clap::Parser;
use cpal::{
    SampleRate,
    traits::{DeviceTrait, StreamTrait},
};

use cpal::StreamConfig;
#[allow(unused)]
use inline_colorization::*;
use ringbuf::traits::{Producer, Split};
use std::path::PathBuf;

#[cfg(target_os = "macos")]
const DEFAULT_INPUT: &str = "BlackHole 2ch";
#[cfg(target_os = "linux")]
const DEFAULT_INPUT: &str = "pipewire";

const DEFAULT_SR: i32 = 48000;
// TODO: Handle multichannel stream based on user config
const DEFAULT_CH: usize = 2;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let output = &args.output.clone();
    let config = Config::load_or_create(args.reset_config).map(|c| c.merge_cli_args(&args))?;
    let filename = crate::util::format_filename(config.file.clone());
    let home = std::env::var("HOME")?;
    let out_dir = match output {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(home).join("tau").join("recordings"),
    };

    let path = out_dir.join(filename.clone().to_string());
    if path.exists() {
        return Err(anyhow::anyhow!(
            "{}\n\tUnable to overwrite already existing file:{}\n\t{}{}{}",
            color_yellow,
            color_reset,
            color_red,
            path.display(),
            color_reset
        ));
    }

    let host = cpal::default_host();
    let device = crate::audio::find_audio_device(&host, &config)?;
    let (mut tx, rx) = ringbuf::HeapRb::<f32>::new(DEFAULT_SR as usize * 4).split();
    let icecast = crate::audio::create_icecast_connection(config.clone())?;

    // Create streaming threads, which loop endlessly
    // TODO: Gracefully shut down
    let _ = {
        if args.no_recording {
            crate::threads::icecast_thread(icecast, rx, filename.clone())
        } else {
            crate::threads::icecast_rec_thread(icecast, rx, &out_dir, filename.clone())
        }
    };

    let requested_config = StreamConfig {
        channels: DEFAULT_CH as u16,
        sample_rate: SampleRate(DEFAULT_SR as u32),
        buffer_size: cpal::BufferSize::Default,
    };

    let stream = device
        .build_input_stream(
            &requested_config,
            move |buf, _info| {
                tx.push_slice(buf);
            },
            |e| {
                eprintln!("{e}");
                std::process::exit(1)
            },
            None,
        )
        .map_err(crate::err::handle_input_build_error)?;

    stream.play()?;

    // Prints pretty message
    crate::ui::print_started_session_msg(
        config.audio_interface,
        &config.url,
        &config.port,
        &path,
        args.no_recording,
    );

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
