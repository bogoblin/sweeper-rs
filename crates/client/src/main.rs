fn main() {
    pollster::block_on(sweeper_client::run());
}