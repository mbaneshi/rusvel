//! Messaging department — [`DepartmentApp`] for outbound channels (`ChannelPort`) when wired in the host.
//!
//! Registered **last** in the app `installed_departments()` list so channel composition stays at the composition root until expanded here.

mod manifest;

use async_trait::async_trait;
use rusvel_core::department::*;
use rusvel_core::error::Result;

pub struct MessagingDepartment;

impl MessagingDepartment {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for MessagingDepartment {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DepartmentApp for MessagingDepartment {
    fn manifest(&self) -> DepartmentManifest {
        manifest::messaging_manifest()
    }

    async fn register(&self, _ctx: &mut RegistrationContext) -> Result<()> {
        tracing::info!("Messaging department registered (channel tools deferred to host)");
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn department_creates() {
        let dept = MessagingDepartment::new();
        assert_eq!(dept.manifest().id, "messaging");
    }
}
