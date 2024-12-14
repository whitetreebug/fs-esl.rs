# fs-esl.rs

learn tokio and create esl lib for freeswitch

## Example

```shell
cargo run --example inbound
```

## How to use

### add configure to Cargo.toml

```Toml
fs-esl = { git = "https://github.com/whitetreebug/fs-esl.rs.git" }
log = "*"
fern = { version = "0.6.2", features = ["chrono", "colored"] }
```
