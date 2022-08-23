use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod utils;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum FromClient {
    Join {
        group_name: Arc<String>,
    },
    Post {
        group_name: Arc<String>,
        message: Arc<String>,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum FromServer {
    Message {
        group_name: Arc<String>,
        message: Arc<String>,
    },
    Error(String),
}

#[cfg(test)]
mod lib_tests {
    use crate::FromClient;
    use std::sync::Arc;

    #[test]
    fn test_from_client() {
        let from_client = FromClient::Post {
            group_name: Arc::new("dogs".to_string()),
            message: Arc::new("bulldogs are scary!".to_string()),
        };
        let str_repr = serde_json::to_string(&from_client).unwrap();

        assert_eq!(
            str_repr,
            r#"{"Post":{"group_name":"dogs","message":"bulldogs are scary!"}}"#
        );

        assert_eq!(
            serde_json::from_str::<FromClient>(&str_repr).unwrap(),
            from_client
        );
    }
}
