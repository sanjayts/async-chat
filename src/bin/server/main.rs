mod connection;
mod group;
mod group_table;

use crate::connection::handle_client;
use crate::group_table::GroupTable;
use async_chat::utils::ChatResult;
use async_std::net::TcpListener;
use async_std::prelude::*;
use async_std::task;
use std::env::args;
use std::sync::Arc;

fn main() -> ChatResult<()> {
    let address = args().nth(1).expect("Usage: server HOST");
    let group_table = Arc::new(GroupTable::new());
    task::block_on(async {
        let server = TcpListener::bind(address).await?;
        let mut connections = server.incoming();
        while let Some(socket_result) = connections.next().await {
            let socket = socket_result?;
            let groups = group_table.clone();
            // When writing this piece, I made the mistake of missing out the spawn which resulted
            // in a very weird behaviour -- a single client working as expected but multiple
            // clients deadlocking the system -- now we know why that happens! :)
            task::spawn(async {
                let result = handle_client(socket, groups).await;
                if let Err(e) = result {
                    eprintln!("Failed to handle client connection: {}", e);
                }
            });
        }
        Ok(())
    })
}
