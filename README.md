
[![Compiles on macOS (Intel / Silicon), Ubuntu and NixOS](https://github.com/tau-org/tau-radio/actions/workflows/rust.yml/badge.svg?event=pull_request)](https://github.com/tau-org/tau-radio/actions/workflows/rust.yml)

# tau-radio

`tau-radio` livestreams audio from a device on your machine to a
[`tau-tower`](https://github.com/tau-org/tau-tower) server, which serves it to
listeners. It captures an audio device, encodes to Ogg Opus, and can also record
the stream to a local file while it broadcasts. Pair it with
[Asciinema](https://asciinema.org) to broadcast a terminal session with live
audio.

Tau is two programs:

- **`tau-radio`** (this program) is the broadcaster client. It runs on your
  machine, captures an audio device, and sends the stream out.
- **`tau-tower`** is the server. It runs on a VPS, receives that stream, and
  serves it to many listeners at once.

`tau-radio` needs a running `tau-tower` to send to. Set the server up first with
the
[`tau-tower` self-hosting guide](https://github.com/tau-org/tau-tower/blob/main/docs/self-hosting.md).
To try Tau locally before running on a VPS, run both programs on one machine with
[`tau-tower` local testing](https://github.com/tau-org/tau-tower/blob/main/docs/local-test.md).

---

## Install tau-radio

Install the native audio libraries and headers that `tau-radio` links against:
Opus, Ogg, libshout, and the audio backend for your operating system.

macOS example:

```bash
brew install opus libopusenc libogg libshout
```

Linux example:

```bash
sudo apt update
sudo apt install \
  build-essential autoconf automake libtool pkg-config \
  libjack-dev libasound2-dev libpipewire-0.3-dev \
  libopus-dev libopusenc-dev libopusfile-dev opus-tools \
  libogg-dev libshout-dev
```

Then install `tau-radio`:

```bash
cargo install --git https://github.com/tau-org/tau-radio
```

If the shell says `command not found: tau-radio`, add Cargo's binary directory
to your shell path:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$HOME/.zshrc"
source "$HOME/.zshrc"
```

On first run, `tau-radio` looks for a config file. It uses
`$XDG_CONFIG_HOME/tau/config.toml` when `XDG_CONFIG_HOME` is set, and
`$HOME/.config/tau/config.toml` otherwise. This is the same on macOS and Linux.

If no config file exists, `tau-radio` prompts you to create one. To start over
later, run `tau-radio --reset-config` and answer the prompts again.

Note that the interactive prompt defaults `upstream_port` to `8000`, which suits a
direct test. For the public Caddy setup, enter `8001` instead. See
[the proxy setup guide](https://github.com/tau-org/tau-tower/blob/main/docs/proxy-setup.md) for more information.

Values inside angle brackets are placeholders. Replace the whole placeholder,
including the brackets, before saving the config file.

```toml
username = "<source-username>"
password = "<source-password>"

url = "<your-audio-domain>"

upstream_port = 8001

audio_interface = "<audio-device>"
tls = true
```

Set `url` to the domain or IP address only. Do not include `https://`, `ws://`, or `/tau.ogg`.

To connect directly to `tau-tower` without Caddy, use the server's
`listen_port` as `upstream_port` and set `tls = false`.

## Choose your audio device

`tau-radio` captures an audio input named by `audio_interface`. To stream system
audio rather than a microphone, route it into a virtual loopback device first. On
macOS, you can use [BlackHole](https://github.com/ExistentialAudio/BlackHole). On Linux,
use PipeWire, whose monitor exposes system output as an input. Then set
`audio_interface` to that device:

```toml
audio_interface = "BlackHole 2ch"   # macOS
audio_interface = "pipewire"        # Linux
```

To stream a microphone instead, set `audio_interface` to that input's exact name.

`audio_interface` must match a device name character for character, and
`tau-radio` does not print a device list. If the name is wrong, it exits with
`The audio_interface you have chosen does not match anything on your system`.
List the real names with a system tool, for example:

```bash
system_profiler SPAudioDataType     # macOS
pactl list sources short            # Linux with PipeWire
```

The device must run at 48 kHz. If `tau-radio` reports `StreamConfigNotSupported`,
set the device sample rate to 48 kHz and start it again.

## Record the stream locally

While it broadcasts, `tau-radio` also writes a local Ogg Opus recording. By
default, recordings go to `$HOME/tau/recordings/` with the filename
`tau_<timestamp>.ogg`.

Control recording with CLI flags or config:

- `--no-recording` turns off the local recording.
- `--output <dir>` sets the recordings directory.
- `--file <name>` sets the base filename. The same value can be set as `file`
  in `config.toml`.

Example, stream without recording and override credentials for one run:

```bash
tau-radio \
  --username <source-username> \
  --password <source-password> \
  --no-recording
```

## Use tau-radio with Asciinema

`tau-radio` provides the audio for a live Asciinema terminal stream; it does not
stream video or terminal output itself. Deploy the Asciinema server and start a
stream with the
[Add Asciinema guide](https://github.com/tau-org/tau-tower/blob/main/docs/asciinema_setup.md).

For every config field, see the
[`tau-tower` config reference](https://github.com/tau-org/tau-tower/blob/main/docs/config-reference.md).

## Funding

This project is funded through [NGI Zero Core](https://nlnet.nl/core), a fund established by [NLnet](https://nlnet.nl) with financial support from the European Commission's [Next Generation Internet](https://ngi.eu) program. Learn more at the [NLnet project page](https://nlnet.nl/project/Tau).

[<img src="https://nlnet.nl/logo/banner.png" alt="NLnet foundation logo" width="20%" />](https://nlnet.nl)
[<img src="https://nlnet.nl/image/logos/NGI0_tag.svg" alt="NGI Zero Logo" width="20%" />](https://nlnet.nl/core)
