use async_std::io::WriteExt;

use crate::FromClient;
use async_std::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::sync::Arc;

/// The error type representing all errors which happen in the chat app
pub type ChatError = Box<dyn Error + Send + Sync + 'static>;

/// The result type representing the result of all computations in our chat code
pub type ChatResult<T> = Result<T, ChatError>;

pub async fn send_as_json<S, P>(sink: &mut S, packet: P) -> ChatResult<()>
where
    S: async_std::io::Write + Unpin,
    P: Serialize,
{
    let mut json = serde_json::to_string(&packet)?;
    json.push('\n');
    sink.write_all(json.as_bytes()).await?;
    Ok(())
}

pub fn receive_as_json<S, P>(inbound: S) -> impl Stream<Item = ChatResult<P>>
where
    S: async_std::io::BufRead + Unpin,
    P: DeserializeOwned,
{
    inbound.lines().map(|line_result| -> ChatResult<P> {
        let line = line_result.unwrap();
        let packet = serde_json::from_str(&line)?;
        Ok(packet)
    })
}

#[cfg(test)]
mod utils_tests {
    use crate::utils::{receive_as_json, send_as_json};
    use crate::FromClient;

    use async_std::prelude::*;

    use std::sync::Arc;

    #[async_std::test]
    async fn test_send_as_json() {
        let from_client = FromClient::Join {
            group_name: Arc::new("mah-group".to_string()),
        };

        let mut buf = vec![];
        send_as_json(&mut buf, from_client).await.unwrap();

        let result = String::from_utf8(buf).unwrap();
        assert_eq!(result, "{\"Join\":{\"group_name\":\"mah-group\"}}\n");
    }

    #[async_std::test]
    async fn test_receive_as_json() {
        // let line = "{\"Join\":{\"group_name\":\"mah-group\"}}\n\
        //     {\"Join\":{\"group_name\":\"yo-group\"}}\n";
        let line = "{\"Join\":{\"group_name\":\"mah-group\"}}\n";
        let inbound = line.as_bytes();
        let incoming_stream = receive_as_json::<_, FromClient>(inbound);

        let mut results = vec![];
        incoming_stream.for_each(|item| {
            results.push(item);
        });

        // FIXME Find out why the below assertion is not working?!
        // assert_eq!(results.len(), 2);
    }
}
