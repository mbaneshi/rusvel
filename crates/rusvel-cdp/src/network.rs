//! Network domain hooks for passive capture (`Network.responseReceived`, etc.).
//!
//! Phase 1 does not emit captured payloads; [`crate::CdpClient::observe`] uses a
//! broadcast channel ready for future `Network.enable` wiring.
//!
//! When capture is implemented, emit via [`crate::CdpClient::publish_browser_event`] so
//! [`rusvel_terminal::inject_browser_event_log`] can mirror lines into browser panes.

/// Reserved for Phase 2: enable CDP Network domain and map responses to [`BrowserEvent::DataCaptured`](rusvel_core::BrowserEvent::DataCaptured).
#[derive(Debug, Default, Clone)]
pub struct NetworkCapture;

impl NetworkCapture {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
