use async_chat::utils::{receive_as_json, ChatResult};
use async_chat::{utils, FromClient, FromServer};
use async_std::io::prelude::BufReadExt;
use async_std::io::{stdin, BufReader, WriteExt};
use async_std::net::TcpStream;
use async_std::prelude::{FutureExt, StreamExt};
use async_std::task;
use std::env::args;
use std::sync::Arc;

fn main() -> ChatResult<()> {
    let address = args().nth(1).expect("Usage: client HOST:PORT");
    task::block_on(async {
        let socket = TcpStream::connect(address).await?;
        socket.set_nodelay(true)?;
        let to_server = send_commands(socket.clone());
        let from_server = handle_replies(socket);
        from_server.race(to_server).await?;
        Ok(())
    })
}

async fn send_commands(mut server: TcpStream) -> ChatResult<()> {
    println!(
        "Command:\n\
    join GROUP\n\
    post GROUP MESSAGE\n\
    Type CTRL+D (Unix) or CTRL+Z (Windows) to close connection
    "
    );
    let mut lines = BufReader::new(stdin()).lines();
    while let Some(result) = lines.next().await {
        let command = result.unwrap();
        if command.trim().is_empty() {
            continue;
        }
        let request = match parse_command(&command) {
            Some(from_client) => from_client,
            None => {
                eprintln!("Invalid command: {}", command);
                continue;
            }
        };
        utils::send_as_json(&mut server, request).await?;
        server.flush().await?;
    }
    Ok(())
}

fn parse_command(line: &str) -> Option<FromClient> {
    let parts: Vec<&str> = line.trim().splitn(3, " ").collect();
    match parts.as_slice() {
        ["join", group_name] => Some(FromClient::Join {
            group_name: Arc::new(group_name.to_string()),
        }),
        ["post", group_name, message] => Some(FromClient::Post {
            group_name: Arc::new(group_name.to_string()),
            message: Arc::new(message.to_string()),
        }),
        _ => None,
    }
}

async fn handle_replies(server: TcpStream) -> ChatResult<()> {
    let buffered = BufReader::new(server);
    let mut stream = receive_as_json(buffered);

    while let Some(result) = stream.next().await {
        match result? {
            FromServer::Message {
                group_name,
                message,
            } => {
                println!("Message posted for group {}: {}", group_name, message);
            }
            FromServer::Error(error_msg) => {
                println!("Error from server: {}", error_msg);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod client_tests {
    use crate::parse_command;
    use async_chat::FromClient;
    use std::sync::Arc;

    #[test]
    fn test_parse_command() {
        let cmd = parse_command("join c++devs").unwrap();
        assert_eq!(
            cmd,
            FromClient::Join {
                group_name: Arc::new("c++devs".to_string())
            }
        );

        let cmd = parse_command("post c++devs C++ sucks, use Rust!").unwrap();
        assert_eq!(
            cmd,
            FromClient::Post {
                group_name: Arc::new("c++devs".to_string()),
                message: Arc::new("C++ sucks, use Rust!".to_string()),
            }
        );

        let cmd = parse_command("blah blach");
        assert!(cmd.is_none());
    }
}
