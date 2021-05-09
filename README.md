# rust-mqtt-chat

This repo contains simple network chat which uses MQTT and is written in Rust. This app was made mainly as a TDD exercise for me, so keep that in mind when using it.

![screen shot](screenshot.png)


## Setup

Clone this repo and run `cargo build` to compile.

## Usage

```bash
cargo run -- --server tcp://localhost:1883 --room kitchen --user chef --password knife
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
