use std::net::SocketAddr;

use axum::Router;
use futures_util::FutureExt;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

mod world;

#[tokio::main]
async fn main() {
    let router: Router<> = Router::new()
        .fallback_service(ServeDir::new("static"));
    let addr = SocketAddr::from(([127,0,0,1], 8000));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router).await.unwrap();
}