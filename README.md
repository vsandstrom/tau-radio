## Usage:
This tool is built for livestreaming audio to a server hosting an Icecast
web-radio instance. You are able to hijack a audio device connected to your
computer and transmit its signal directly to this radio stream. 

For convenience, the option to record the current stream is also available,
encoded in the same format as the stream itself. Currently the only option is
**ogg opus**.

The first time using the tool, it will search your system for a config file. 
It looks for it in the directory: (early days so only tested on macOS yet)
```bash
$ $HOME/.config/tau/config.toml
```

If there is no config file located there, you will be prompted to create one. 

```
$ cargo run
```

If you want to temporarily overwrite the config, you are able to pass arguments.

```bash
# Uses temporary credentials, and disables the local recording. 
$ cargo run -- --username <username> --password <password> --no-recording
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
