# Strategy Plugin SDK

This repo supports strategy plugins via a lightweight SDK workflow.

## Built-In vs SDK Plugins

- Built-in: shipped in `src/strategy.rs` and always available.
- SDK: generated as a separate Rust crate under `plugins_sdk/<id>/`, then registered into `config/sdk_plugins.toml`.

List runtime-visible plugins:

```bash
cargo run -- plugins
```

## Create A New SDK Plugin

```bash
cargo run -- sdk-init --id my_new_alpha --output-dir plugins_sdk
cargo run -- sdk-check --package-dir plugins_sdk/my_new_alpha
```

Edit `plugins_sdk/my_new_alpha/src/lib.rs`, then register it:

```bash
cargo run -- sdk-register --package-dir plugins_sdk/my_new_alpha --name "My New Alpha"
```

This writes/updates `config/sdk_plugins.toml`.

## Run With An SDK Plugin

```bash
cargo run -- run --config config/bot.toml --strategy-plugin my_new_alpha --lang en
```

## Parameterization

SDK plugins can expose simple runtime parameters via `config/sdk_plugins.toml`.

Example (`my_alpha`):

```toml
[[plugins]]
plugin_id = "my_alpha"
name = "My Alpha"
description = "Layered multi-factor SDK"
enabled = true
min_price = 1.0
alpha_volume_scale = 0.00001
```

See: `src/sdk.rs` and `plugins_sdk/my_alpha/`.

