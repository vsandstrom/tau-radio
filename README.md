## Usage:
This tool is built for livestreaming audio to a server hosting a
[`tau-tower`](https://github.com/tau-org/tau-tower)
web-radio server instance. You are able to hijack an audio device connected 
to your computer and transmit its signal directly to this radio stream. 

For convenience, the option to record the current stream is also available,
encoded in the same format as the stream itself. Currently the only option is
**ogg opus**.

---

To install:
```bash
$ cargo install --git https://github.com/tau-org/tau-radio
```

The first time using the tool, it will search your system for a config file. 
It looks for it in the directory:
```bash
$ $HOME/.config/tau/config.toml
```

If there is no config file located there, you will be prompted to create one. 

[![asciicast](https://asciinema.org/a/2lXsKE2jRhdfQ8r2OEoDHk8fF.svg)](https://asciinema.org/a/2lXsKE2jRhdfQ8r2OEoDHk8fF)


If you want to temporarily overwrite the config, you are able to pass arguments.

```bash
# Ex: Uses temporary credentials, and disables the local recording. 
$ tau-radio \
  --username <username> \
  --password <password> \
  --no-recording
```

### Dependencies

**On macOS** (using Homebrew):
```bash
$ brew install opus libopusenc libogg libshout
```

**On Linux** (using apt):
```bash
$ sudo apt update
$ sudo apt install libopus-dev libopusenc-dev libogg-dev libshout-dev
```

