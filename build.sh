wasm-pack build "$(dirname "$0")/crates/wgpu-frontend" --out-dir "$(dirname "$0")/crates/server/static" --release --target=web
cargo build "$(dirname "$0")/crates/server" --release