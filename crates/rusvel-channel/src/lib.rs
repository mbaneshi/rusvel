//! Outbound messaging [`ChannelPort`] — implemented by Telegram/Discord adapters (Sprint 8).

mod telegram;

pub use rusvel_core::ports::ChannelPort;
pub use telegram::TelegramChannel;
