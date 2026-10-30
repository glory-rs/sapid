//! Application-owned confirmation/checkpoint state. Requests do not grant
//! authority, execute work, or change the supplied status.
use glory::ui::Result;

pub const MAX_CONFIRMATION_BYTES: usize = 32768;
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
fn text(value: &str, maximum: usize, empty: bool) -> bool {
    value.len() <= maximum && !value.contains(['\0', '\r']) && (empty || !value.trim().is_empty())
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConfirmationStatus {
    #[default]
    Pending,
    Approved,
    Rejected,
    Cancelled,
}
impl ConfirmationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "Awaiting confirmation",
            Self::Approved => "Approved by application",
            Self::Rejected => "Rejected by application",
            Self::Cancelled => "Cancelled by application",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmationDecision {
    Approve,
    Reject,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfirmationRequest {
    pub key: String,
    /// Application-supplied version of the displayed request. This is not an
    /// authorization token; the application decides whether it is still valid.
    pub revision: u64,
    pub decision: ConfirmationDecision,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfirmationData {
    pub title: String,
    pub description: String,
    pub details: String,
    pub status: ConfirmationStatus,
    pub revision: u64,
}
impl Default for ConfirmationData {
    fn default() -> Self {
        Self {
            title: "Confirmation requested".into(),
            description: String::new(),
            details: String::new(),
            status: ConfirmationStatus::Pending,
            revision: 0,
        }
    }
}
impl ConfirmationData {
    pub fn validate(&self) -> Result {
        validate(&self.title, &self.description, &self.details)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfirmationLabels {
    pub pending: String,
    pub approved: String,
    pub rejected: String,
    pub cancelled: String,
    pub approve: String,
    pub reject: String,
}
impl Default for ConfirmationLabels {
    fn default() -> Self {
        Self {
            pending: ConfirmationStatus::Pending.label().into(),
            approved: ConfirmationStatus::Approved.label().into(),
            rejected: ConfirmationStatus::Rejected.label().into(),
            cancelled: ConfirmationStatus::Cancelled.label().into(),
            approve: "Approve request".into(),
            reject: "Reject request".into(),
        }
    }
}
impl ConfirmationLabels {
    pub fn validate(&self) -> Result {
        for value in [
            &self.pending,
            &self.approved,
            &self.rejected,
            &self.cancelled,
            &self.approve,
            &self.reject,
        ] {
            if !text(value, 256, false) || value.chars().any(char::is_control) {
                return Err(invalid("invalid bounded confirmation label"));
            }
        }
        Ok(())
    }
    pub fn status(&self, status: ConfirmationStatus) -> &str {
        match status {
            ConfirmationStatus::Pending => &self.pending,
            ConfirmationStatus::Approved => &self.approved,
            ConfirmationStatus::Rejected => &self.rejected,
            ConfirmationStatus::Cancelled => &self.cancelled,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckpointStatus {
    #[default]
    Available,
    Current,
    Restoring,
    Restored,
    Unavailable,
}
impl CheckpointStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Available => "Available checkpoint",
            Self::Current => "Current checkpoint",
            Self::Restoring => "Restoration in progress",
            Self::Restored => "Restored by application",
            Self::Unavailable => "Unavailable checkpoint",
        }
    }
    pub fn can_request_restore(self) -> bool {
        self == Self::Available
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckpointRequest {
    pub key: String,
    pub revision: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckpointData {
    pub title: String,
    pub description: String,
    pub details: String,
    pub status: CheckpointStatus,
    pub revision: u64,
}
impl Default for CheckpointData {
    fn default() -> Self {
        Self {
            title: "Checkpoint".into(),
            description: String::new(),
            details: String::new(),
            status: CheckpointStatus::Available,
            revision: 0,
        }
    }
}
impl CheckpointData {
    pub fn validate(&self) -> Result {
        validate(&self.title, &self.description, &self.details)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckpointLabels {
    pub available: String,
    pub current: String,
    pub restoring: String,
    pub restored: String,
    pub unavailable: String,
    pub restore: String,
}
impl Default for CheckpointLabels {
    fn default() -> Self {
        Self {
            available: CheckpointStatus::Available.label().into(),
            current: CheckpointStatus::Current.label().into(),
            restoring: CheckpointStatus::Restoring.label().into(),
            restored: CheckpointStatus::Restored.label().into(),
            unavailable: CheckpointStatus::Unavailable.label().into(),
            restore: "Restore checkpoint".into(),
        }
    }
}
impl CheckpointLabels {
    pub fn validate(&self) -> Result {
        for value in [
            &self.available,
            &self.current,
            &self.restoring,
            &self.restored,
            &self.unavailable,
            &self.restore,
        ] {
            if !text(value, 256, false) || value.chars().any(char::is_control) {
                return Err(invalid("invalid bounded checkpoint label"));
            }
        }
        Ok(())
    }
    pub fn status(&self, status: CheckpointStatus) -> &str {
        match status {
            CheckpointStatus::Available => &self.available,
            CheckpointStatus::Current => &self.current,
            CheckpointStatus::Restoring => &self.restoring,
            CheckpointStatus::Restored => &self.restored,
            CheckpointStatus::Unavailable => &self.unavailable,
        }
    }
}
fn validate(title: &str, description: &str, details: &str) -> Result {
    if !text(title, 256, false)
        || title.chars().any(char::is_control)
        || !text(description, 8192, true)
        || !text(details, 24576, true)
    {
        return Err(invalid("invalid bounded confirmation/checkpoint text"));
    }
    if title.len() + description.len() + details.len() > MAX_CONFIRMATION_BYTES {
        return Err(invalid(
            "confirmation/checkpoint exceeds its independent total byte budget",
        ));
    }
    Ok(())
}
