use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generates a strongly-typed newtype wrapper around [`Uuid`].
///
/// Every ID in RUSVEL is its own type so the compiler prevents mixing
/// e.g. a `SessionId` where a `RunId` is expected.
///
/// Each generated type derives:
///   `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`
///
/// And provides:
///   - `new()` — random `UUIDv7` (time-ordered)
///   - `from_uuid(uuid)` — wrap an existing UUID
///   - `as_uuid()` — borrow the inner value
///   - `Display` — delegates to the inner UUID
macro_rules! define_id {
    ($($(#[$meta:meta])* $name:ident),+ $(,)?) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
            #[serde(transparent)]
            pub struct $name(Uuid);

            impl $name {
                /// Create a new time-ordered (v7) identifier.
                pub fn new() -> Self {
                    Self(Uuid::now_v7())
                }

                /// Wrap an existing [`Uuid`].
                pub fn from_uuid(id: Uuid) -> Self {
                    Self(id)
                }

                /// Borrow the inner [`Uuid`].
                pub fn as_uuid(&self) -> &Uuid {
                    &self.0
                }
            }

            impl Default for $name {
                fn default() -> Self {
                    Self::new()
                }
            }

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    self.0.fmt(f)
                }
            }

            impl From<Uuid> for $name {
                fn from(id: Uuid) -> Self {
                    Self(id)
                }
            }

            impl From<$name> for Uuid {
                fn from(id: $name) -> Uuid {
                    id.0
                }
            }
        )+
    };
}

define_id!(
    /// Identifies a workspace session (project, lead, campaign, …).
    SessionId,
    /// Identifies a single execution run within a session.
    RunId,
    /// Identifies a message thread within a run.
    ThreadId,
    /// Identifies an async job in the central queue.
    JobId,
    /// Identifies a reusable agent persona / profile.
    AgentProfileId,
    /// Identifies a domain event.
    EventId,
    /// Identifies a harvested opportunity.
    OpportunityId,
    /// Identifies a content item.
    ContentId,
    /// Identifies a CRM contact.
    ContactId,
    /// Identifies a goal.
    GoalId,
    /// Identifies a task.
    TaskId,
    /// Identifies a code snapshot.
    SnapshotId,
    /// Identifies a user (solo founder — usually exactly one).
    UserId,
    /// Identifies a logical workspace.
    WorkspaceId,
    /// Identifies a flow (DAG workflow) definition.
    FlowId,
    /// Identifies a single execution of a flow.
    FlowExecutionId,
    /// Identifies a node within a flow definition.
    FlowNodeId,
    /// Identifies a terminal multiplexer window within a session.
    WindowId,
    /// Identifies a terminal pane (PTY-backed shell or command).
    PaneId,
);

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique() {
        let a = SessionId::new();
        let b = SessionId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn id_roundtrips_through_serde() {
        let id = RunId::new();
        let json = serde_json::to_string(&id).unwrap();
        let back: RunId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn id_display_matches_uuid() {
        let uuid = Uuid::now_v7();
        let id = JobId::from_uuid(uuid);
        assert_eq!(id.to_string(), uuid.to_string());
    }

    #[test]
    fn different_id_types_are_distinct() {
        // This is a compile-time guarantee — the test just documents intent.
        let _s: SessionId = SessionId::new();
        let _r: RunId = RunId::new();
        // `_s == _r` would not compile — that's the point.
    }
}
