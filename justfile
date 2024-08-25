set dotenv-load

build:
    cargo build --release

flash:
    cargo-espflash espflash flash --target-dir ./target/ --release --monitor
