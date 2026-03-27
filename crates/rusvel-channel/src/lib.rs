//! Outbound messaging [`ChannelPort`] — implemented by Telegram/Discord adapters (Sprint 8).

use async_trait::async_trait;
use rusvel_core::error::Result;
use rusvel_core::id::SessionId;
use serde_json::Value;

/// Sends notifications or interactive messages to an external channel.
#[async_trait]
pub trait ChannelPort: Send + Sync {
    fn channel_kind(&self) -> &'static str;

    async fn send_message(&self, session_id: &SessionId, payload: Value) -> Result<()>;
}
