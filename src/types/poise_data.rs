use crate::types::{AppError, AppState};

/// PoiseData（AppStateを含む）
#[derive(Debug)]
pub struct PoiseData {
    pub app_state: AppState,
}

pub type PoiseContext<'a> = poise::Context<'a, PoiseData, AppError>;
