https://github.com/Systemscape/slint-futuresdr-demo/assets/20155974/8cae6b7a-0b9b-4df7-a601-74c47afc7894

# Building for WASM

## Prerequisites

### Trunk
```bash
cargo install --locked trunk
# or with binstall
cargo binstall trunk
```

## Running

1. (Optional) adapt `index.html` to use the correct feature set. Default is the entirely self-contained version.
2. Run `trunk serve --release` (takes a while)
3. Open your browser at the address shown by trunk (e.g., `http://127.0.0.1:8080/`)

# Building for Desktop

Use `cargo run` or `cargo run --bin <binary> --features <feature1>,<feature2>` to build and run the program.

## Fully integrated, all features
```bash
cargo run --release
```

## Playback Recorded Data
If you want, you can generate a new vector and make settings with
```bash
cargo run --bin record_to_file --features record_to_file --no-default-features
```

```bash
cargo run --features replay_vec --no-default-features
```

## Stream from PC to Browser via Websocket
Start the Websocket transmitter in a separate session:
```bash
cargo run --bin websocket_tx --features websocket_tx --no-default-features
```

and then the websocket receiver:
```bash
cargo run --features websocket_rx --no-default-features
```


# Features and Binaries
In order to have everything contained in a single crate, different functionality is feature-gated
and the ones that are not related to producing plots are in separate binaries.

By Default `svg` and `futuresdr_integrated` features are enabled.
**CAUTION**: The SVG backend is rather slow, so use it with `--release`.

The features are described in `Cargo.toml`. The following combinations are valid:

| svg | futuresdr_integrated | record_to_file | replay_vec | websocket_tx | websocket_rx | `--bin`          |
|-----|----------------------|----------------|------------|--------------|--------------|------------------|
| x   | x                    |                |            |              |              | `run_plot`       |
| x   |                      |                | x          |              |              | `run_plot`       |
| x   |                      |                |            |              | x            | `run_plot`       |
|     |                      | x              |            |              |              | `record_to_file` |
|     |                      |                |            | x            |              | `websocket_tx`   |

