use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// shutdown_ritual_steps row model
#[derive(Debug, FromRow)]
pub struct ShutdownRitualStepRow {
    pub id: String,
    pub title: String,
    pub order_rank: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Domain model (currently identical to row)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownRitualStep {
    pub id: Uuid,
    pub title: String,
    pub order_rank: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<ShutdownRitualStepRow> for ShutdownRitualStep {
    type Error = String;

    fn try_from(row: ShutdownRitualStepRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::parse_str(&row.id).map_err(|e| e.to_string())?,
            title: row.title,
            order_rank: row.order_rank,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// shutdown_ritual_settings row model (singleton)
#[derive(Debug, FromRow)]
pub struct ShutdownRitualSettingsRow {
    pub id: String,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Domain model for shutdown ritual settings (singleton)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownRitualSettings {
    pub id: String,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ShutdownRitualSettingsRow> for ShutdownRitualSettings {
    fn from(row: ShutdownRitualSettingsRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}


