wasm-pack build "crates/wgpu-frontend" --out-dir "../../crates/server/static" --release --target=web --no-typescript --no-pack
cargo build --bin sweeper-server --release
scp ./target/release/sweeper-server root@infinitesweeper.online:~/sweeper-server
ssh root@infinitesweeper.online "mv ~/sweeper-server ~/sweeper-rs/target/release/sweeper-server"