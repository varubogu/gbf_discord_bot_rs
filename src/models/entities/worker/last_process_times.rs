use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "worker", table_name = "last_process_times")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub process_type: i32,
    pub execute_time: Option<DateTimeUtc>,
    pub memo: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            process_type: sea_orm::NotSet,
            execute_time: sea_orm::NotSet,
            memo: sea_orm::NotSet,
            created_at: sea_orm::Set(now),
            updated_at: sea_orm::Set(now),
        }
    }
}

// 処理種類のenum定義（Python側のLastProcessTypeに対応）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LastProcessType {
    Schedule = 1,
    SpreadsheetLoad = 2,
    SpreadsheetPush = 3,
    BattleRecruitmentSchedule = 4,
}

impl LastProcessType {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::Schedule),
            2 => Some(Self::SpreadsheetLoad),
            3 => Some(Self::SpreadsheetPush),
            4 => Some(Self::BattleRecruitmentSchedule),
            _ => None,
        }
    }

    pub fn memo(&self) -> &'static str {
        match self {
            Self::Schedule => "最終スケジュール実行日時",
            Self::SpreadsheetLoad => "最終Googleスプレッドシート読み込み日時",
            Self::SpreadsheetPush => "最終Googleスプレッドシート書き込み日時",
            Self::BattleRecruitmentSchedule => "最終マルチ募集スケジュール実行日時",
        }
    }
}
