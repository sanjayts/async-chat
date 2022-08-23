use crate::connection::Outbound;
use async_chat::FromServer;
use async_std::task;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;

/// The group struct responsible for broadcasting messages to all subscribers for a givne group
pub struct Group {
    name: Arc<String>,
    sender: broadcast::Sender<Arc<String>>,
}

impl Group {
    pub fn new(name: Arc<String>) -> Group {
        let (sender, _receiver) = broadcast::channel(1000);
        Group { name, sender }
    }

    pub fn join(&self, outbound: Arc<Outbound>) {
        let receiver = self.sender.subscribe();
        task::spawn(handle_subscription(self.name.clone(), receiver, outbound));
    }

    pub fn post(&self, message: Arc<String>) {
        let _result = self.sender.send(message);
    }
}

async fn handle_subscription(
    name: Arc<String>,
    mut receiver: Receiver<Arc<String>>,
    outbound: Arc<Outbound>,
) {
    loop {
        let packet = match receiver.recv().await {
            Ok(message) => FromServer::Message {
                group_name: name.clone(),
                message: message.clone(),
            },
            Err(broadcast::error::RecvError::Lagged(n)) => {
                FromServer::Error(format!("Dropped {} messages from {}", n, name))
            }
            Err(e) => {
                eprintln!("Receiving of message failed due to {}", e.to_string());
                break;
            }
        };
        if outbound.send(packet).await.is_err() {
            eprintln!("Failed to send message to outbound");
            break;
        }
    }
}
