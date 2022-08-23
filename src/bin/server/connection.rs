use crate::GroupTable;
use async_chat::utils::{receive_as_json, send_as_json, ChatResult};
use async_chat::{FromClient, FromServer};
use async_std::io::{BufReader, WriteExt};
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::sync::Mutex;
use std::sync::Arc;

/// An outbound instance abstracts over the concept of a client connection. Since we can have
/// multiple groups which refer to a given client connection who is connected to the server, we
/// typically wrap it up in an Arc to ensure all groups point to the same outbound/client.
///
/// A given outbound controls access to its underlying tcp stream by wrapping it with a Mutex.
pub struct Outbound(Mutex<TcpStream>);

impl Outbound {
    pub fn new(to_client: TcpStream) -> Outbound {
        Outbound(Mutex::new(to_client))
    }

    pub async fn send(&self, packet: FromServer) -> ChatResult<()> {
        let mut guard = self.0.lock().await;
        send_as_json(&mut *guard, &packet).await?;
        guard.flush().await?;
        Ok(())
    }
}

pub async fn handle_client(socket: TcpStream, groups: Arc<GroupTable>) -> ChatResult<()> {
    let outgoing_socket = socket.clone();
    let outbound = Arc::new(Outbound::new(outgoing_socket));
    let reader = BufReader::new(socket);
    let groups = groups.clone();
    let mut from_client = receive_as_json(reader);
    while let Some(packet_result) = from_client.next().await {
        let packet: FromClient = packet_result?;
        let result = match packet {
            FromClient::Join { group_name } => {
                let group = groups.add_and_get(group_name);
                group.join(outbound.clone());
                Ok(())
            }
            FromClient::Post {
                group_name,
                message,
            } => match groups.get(&group_name) {
                Some(group) => {
                    group.post(message);
                    Ok(())
                }
                None => Err(format!("Group doesn't exist {}", group_name)),
            },
        };
        if let Err(e) = result {
            let server_error = FromServer::Error(e);
            outbound.send(server_error).await?;
        }
    }
    Ok(())
}
