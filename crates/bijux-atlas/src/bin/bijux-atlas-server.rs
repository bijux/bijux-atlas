// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> Result<(), String> {
    bijux_atlas::app::server::host::main_entry().await
}
