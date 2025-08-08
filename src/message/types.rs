use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: Uuid,
    pub username: String,
    pub ip: IpAddr,
    pub port: u16,
    pub last_seen: DateTime<Utc>,
}

impl Peer {
    pub fn new(username: String, ip: IpAddr, port: u16) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            ip,
            port,
            last_seen: Utc::now(),
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "discovery")]
    Discovery {
        username: String,
        port: u16,
        peer_id: Uuid,
    },
    #[serde(rename = "discovery_response")]
    DiscoveryResponse {
        username: String,
        port: u16,
        peer_id: Uuid,
    },
    #[serde(rename = "message")]
    ChatMessage {
        sender: String,
        recipient: String, // "all" for broadcast
        content: String,
        timestamp: DateTime<Utc>,
        message_id: Uuid,
    },
    #[serde(rename = "user_join")]
    UserJoin {
        username: String,
        peer_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "user_leave")]
    UserLeave {
        username: String,
        peer_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        peer_id: Uuid,
        timestamp: DateTime<Utc>,
    },
}

impl Message {
    pub fn discovery(username: String, port: u16, peer_id: Uuid) -> Self {
        Message::Discovery {
            username,
            port,
            peer_id,
        }
    }

    pub fn discovery_response(username: String, port: u16, peer_id: Uuid) -> Self {
        Message::DiscoveryResponse {
            username,
            port,
            peer_id,
        }
    }

    pub fn chat_message(sender: String, recipient: String, content: String) -> Self {
        Message::ChatMessage {
            sender,
            recipient,
            content,
            timestamp: Utc::now(),
            message_id: Uuid::new_v4(),
        }
    }

    pub fn user_join(username: String, peer_id: Uuid) -> Self {
        Message::UserJoin {
            username,
            peer_id,
            timestamp: Utc::now(),
        }
    }

    pub fn user_leave(username: String, peer_id: Uuid) -> Self {
        Message::UserLeave {
            username,
            peer_id,
            timestamp: Utc::now(),
        }
    }

    pub fn heartbeat(peer_id: Uuid) -> Self {
        Message::Heartbeat {
            peer_id,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatEvent {
    pub peer: Peer,
    pub message: Message,
}

impl ChatEvent {
    pub fn new(peer: Peer, message: Message) -> Self {
        Self { peer, message }
    }
}
