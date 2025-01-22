wasm-pack build "crates/wgpu-frontend" --out-dir "../../crates/server/static" --release --target=web --no-typescript --no-pack
cargo build --bin sweeper-server