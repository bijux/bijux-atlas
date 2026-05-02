// SPDX-License-Identifier: Apache-2.0

use crate::app::server::AppState;
use std::future::Future;

pub mod cli;
pub mod client;
pub mod http;

/// Build the HTTP server router through the inbound adapter surface.
pub fn build_server_router(state: AppState) -> axum::Router {
    http::router::build_router(state)
}

/// Serve the HTTP router with graceful shutdown through the inbound adapter boundary.
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
