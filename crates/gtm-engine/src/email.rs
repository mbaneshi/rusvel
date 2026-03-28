//! SMTP and mock email adapters for outreach (S-034).

use std::sync::Mutex;

use async_trait::async_trait;
use chrono::Utc;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use rusvel_core::domain::Event;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::{EventId, SessionId};
use rusvel_core::ports::EventPort;

use crate::events;

/// Outbound email for a sequence step or one-off send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailMessage {
    pub to: String,
    pub from: String,
    pub subject: String,
    pub body: String,
}

/// Sends email (SMTP, mock, or custom).
#[async_trait]
pub trait EmailAdapter: Send + Sync {
    async fn send(&self, message: &EmailMessage) -> Result<()>;
}

/// Records every message in memory for tests; never touches the network.
#[derive(Debug, Default)]
pub struct MockEmailAdapter {
    sent: Mutex<Vec<EmailMessage>>,
}

impl MockEmailAdapter {
    pub fn new() -> Self {
        Self {
            sent: Mutex::new(Vec::new()),
        }
    }

    pub fn take_sent(&self) -> Vec<EmailMessage> {
        std::mem::take(&mut *self.sent.lock().expect("mock lock"))
    }

    pub fn sent_len(&self) -> usize {
        self.sent.lock().expect("mock lock").len()
    }
}

#[async_trait]
impl EmailAdapter for MockEmailAdapter {
    async fn send(&self, message: &EmailMessage) -> Result<()> {
        self.sent.lock().expect("mock lock").push(message.clone());
        Ok(())
    }
}

/// SMTP via `RUSVEL_SMTP_HOST`, `RUSVEL_SMTP_PORT` (default 587), `RUSVEL_SMTP_USER`,
/// `RUSVEL_SMTP_PASSWORD`, `RUSVEL_SMTP_FROM` (defaults to `noreply@<host>`).
#[derive(Debug, Clone)]
pub struct SmtpEmailAdapter {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    default_from: String,
}

impl SmtpEmailAdapter {
    /// Returns `None` when `RUSVEL_SMTP_HOST` is unset (opt-in, same spirit as API token auth).
    pub fn from_env() -> Option<Result<Self>> {
        let host = std::env::var("RUSVEL_SMTP_HOST").ok()?;
        Some(Self::from_values(
            &host,
            std::env::var("RUSVEL_SMTP_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(587),
            std::env::var("RUSVEL_SMTP_USER").unwrap_or_default(),
            std::env::var("RUSVEL_SMTP_PASSWORD").unwrap_or_default(),
            std::env::var("RUSVEL_SMTP_FROM").ok(),
        ))
    }

    pub fn from_values(
        host: &str,
        port: u16,
        username: String,
        password: String,
        from_override: Option<String>,
    ) -> Result<Self> {
        let host = host.trim();
        let mut relay = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
            .map_err(|e| RusvelError::Config(format!("SMTP relay: {e}")))?;
        relay = relay
            .port(port)
            .timeout(Some(std::time::Duration::from_secs(30)));
        if !username.is_empty() {
            relay = relay.credentials(Credentials::new(username, password));
        }
        let transport = relay.build();
        let default_from = from_override.unwrap_or_else(|| format!("noreply@{host}"));
        Ok(Self {
            transport,
            default_from,
        })
    }

    fn build_lettre_message(msg: &EmailMessage) -> Result<Message> {
        let from: Mailbox = msg
            .from
            .parse()
            .map_err(|e: lettre::address::AddressError| {
                RusvelError::Validation(format!("invalid From: {e}"))
            })?;
        let to: Mailbox = msg.to.parse().map_err(|e: lettre::address::AddressError| {
            RusvelError::Validation(format!("invalid To: {e}"))
        })?;
        Message::builder()
            .from(from)
            .to(to)
            .subject(&msg.subject)
            .body(msg.body.clone())
            .map_err(|e| RusvelError::Internal(format!("email build: {e}")))
    }
}

#[async_trait]
impl EmailAdapter for SmtpEmailAdapter {
    async fn send(&self, message: &EmailMessage) -> Result<()> {
        let mut m = message.clone();
        if m.from.is_empty() {
            m.from = self.default_from.clone();
        }

        let max_retries = 3u32;
        let mut attempts = 0u32;

        loop {
            let email = Self::build_lettre_message(&m)?;
            match self.transport.send(email).await {
                Ok(_) => return Ok(()),
                Err(e) if attempts < max_retries => {
                    attempts += 1;
                    let delay = std::time::Duration::from_millis(500 * 2u64.pow(attempts - 1));
                    tracing::warn!(
                        attempt = attempts,
                        max = max_retries,
                        delay_ms = delay.as_millis() as u64,
                        "SMTP send failed, retrying: {e}"
                    );
                    tokio::time::sleep(delay).await;
                }
                Err(e) => {
                    return Err(RusvelError::Internal(format!("SMTP send: {e}")));
                }
            }
        }
    }
}

/// Sends via `adapter`, then emits [`crate::events::EMAIL_SENT`] on the bus.
pub async fn send_email_with_event(
    adapter: &dyn EmailAdapter,
    events: &dyn EventPort,
    session_id: Option<SessionId>,
    message: EmailMessage,
) -> Result<()> {
    adapter.send(&message).await?;
    let event = Event {
        id: EventId::new(),
        session_id,
        run_id: None,
        source: "gtm".into(),
        kind: events::EMAIL_SENT.into(),
        payload: serde_json::json!({
            "to": message.to,
            "from": message.from,
            "subject": message.subject,
        }),
        created_at: Utc::now(),
        metadata: serde_json::json!({}),
    };
    events.emit(event).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rusvel_core::domain::EventFilter;
    use rusvel_core::id::EventId;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingEvents {
        emitted: AtomicUsize,
    }

    #[async_trait]
    impl EventPort for CountingEvents {
        async fn emit(&self, _: Event) -> Result<EventId> {
            self.emitted.fetch_add(1, Ordering::SeqCst);
            Ok(EventId::new())
        }
        async fn get(&self, _: &EventId) -> Result<Option<Event>> {
            Ok(None)
        }
        async fn query(&self, _: EventFilter) -> Result<Vec<Event>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn mock_records_send() {
        let mock = MockEmailAdapter::new();
        let msg = EmailMessage {
            to: "a@b.c".into(),
            from: "x@y.z".into(),
            subject: "Hi".into(),
            body: "Body".into(),
        };
        mock.send(&msg).await.unwrap();
        let sent = mock.take_sent();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].subject, "Hi");
    }

    #[tokio::test]
    async fn send_with_event_emits_once() {
        let mock = MockEmailAdapter::new();
        let events = CountingEvents {
            emitted: AtomicUsize::new(0),
        };
        send_email_with_event(
            &mock,
            &events,
            None,
            EmailMessage {
                to: "a@b.c".into(),
                from: "x@y.z".into(),
                subject: "S".into(),
                body: "B".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(events.emitted.load(Ordering::SeqCst), 1);
        assert_eq!(mock.sent_len(), 1);
    }
}
