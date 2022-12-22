use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum VMEventType {
    Start,
    GracefulStop { code: u32 },
    Crash,
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct VMEvent {
    event: VMEventType,
    at: DateTime<Utc>,
    application_id: Uuid,
}

impl VMEvent {
    pub fn now(event: VMEventType, application_id: Uuid) -> Self {
        Self {
            event,
            application_id,
            at: Utc::now(),
        }
    }
}
