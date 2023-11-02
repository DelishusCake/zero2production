use uuid::Uuid;

use chrono::{DateTime, Utc};

use serde::Serialize;

use crate::domain::{EmailAddress, PersonName};

/// New Subscription request
#[derive(Debug)]
pub struct NewSubscription {
    pub name: PersonName,
    pub email: EmailAddress,
}

/// Stored Subscription record
#[derive(Debug, Serialize)]
pub struct Subscription {
    /// ID of the subscription
    pub id: Uuid,
    /// User supplied data
    /// TODO: Should this be parsed back into domain objects?
    pub name: String,
    pub email: String,
    /// Confirmation timestamp.
    /// `None` if the subscription is not confirmed, and therefore cannot receive newsletter emails
    pub confirmed_at: Option<DateTime<Utc>>,
    /// Creation and update timestamps
    /// NOTE: Auto-set and updated by database triggers
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
