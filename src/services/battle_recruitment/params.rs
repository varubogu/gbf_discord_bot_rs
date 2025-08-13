use crate::models::quest::Quest;

pub(crate) struct NewParameter {
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_id: i64,
    pub quest: Quest,
    pub target_id: i32,
    pub battle_type_id: i32,
    pub expiry_date: chrono::DateTime<chrono::Utc>,
}

pub(crate) struct UpdateParameter {
    pub quest: Quest,
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_id: i64,
    pub target_id: i32,
    pub battle_type_id: i32,
    pub expiry_date: chrono::DateTime<chrono::Utc>,
}

pub(crate) struct PaticipantsParameter {
    pub guild_id: i64,
    pub channel_id: i64,
}

pub(crate) struct CancelParameter {
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_id: i64,
}