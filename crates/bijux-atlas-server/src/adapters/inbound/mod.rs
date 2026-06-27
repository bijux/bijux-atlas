// SPDX-License-Identifier: Apache-2.0

use crate::app::server::AppState;
use std::future::Future;

pub mod http {
    pub use bijux_atlas_runtime::adapters::inbound::http::*;
}

/// Build the HTTP server router through the server-owned inbound boundary.
pub fn build_server_router(state: AppState) -> axum::Router {
    http::router::build_router(state)
}

/// Serve the HTTP router with graceful shutdown through the server-owned boundary.
pub async fn serve_server_router_with_shutdown<F>(
    listener: tokio::net::TcpListener,
    state: AppState,
    shutdown: F,
) -> std::io::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    axum::serve(listener, build_server_router(state))
        .with_graceful_shutdown(shutdown)
        .await
}
