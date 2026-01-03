use crate::models::entities::guild_master::{
    battle_recruitment_schedule_days, battle_recruitment_schedules,
};
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

/// スケジュール作成パラメータ
pub struct CreateScheduleParams {
    pub name: String,
    pub guild_id: i64,
    pub channel_id: i64,
    pub quest_id: i32,
    pub battle_style_id: i32,
    pub quest_start_time: sea_orm::prelude::TimeTime,
    pub recruit_start_day_offset: i32,
    pub recruit_start_time: Option<sea_orm::prelude::TimeTime>,
    pub max_participants: Option<i32>,
    pub note: Option<String>,
    pub created_by: i64,
    pub day_of_weeks: Vec<i32>,
}

/// マルチ募集スケジュールリポジトリの抽象インターフェース
#[async_trait]
pub trait BattleRecruitmentScheduleRepository: Send + Sync {
    /// 有効な全スケジュールと曜日情報を取得
    async fn find_all_enabled_schedules_with_days(
        &self,
        db: &sea_orm::DatabaseConnection,
    ) -> Result<
        Vec<(
            battle_recruitment_schedules::Model,
            Vec<battle_recruitment_schedule_days::Model>,
        )>,
    >;

    /// ギルドIDでスケジュール取得
    async fn find_by_guild_id<C>(
        &self,
        db: &C,
        guild_id: i64,
    ) -> Result<
        Vec<(
            battle_recruitment_schedules::Model,
            Vec<battle_recruitment_schedule_days::Model>,
        )>,
    >
    where
        C: sea_orm::ConnectionTrait;

    /// 作成者IDでスケジュール取得
    async fn find_by_created_by<C>(
        &self,
        db: &C,
        created_by: i64,
    ) -> Result<
        Vec<(
            battle_recruitment_schedules::Model,
            Vec<battle_recruitment_schedule_days::Model>,
        )>,
    >
    where
        C: sea_orm::ConnectionTrait;

    /// IDでスケジュール取得
    async fn find_by_id<C>(
        &self,
        db: &C,
        id: i32,
    ) -> Result<
        Option<(
            battle_recruitment_schedules::Model,
            Vec<battle_recruitment_schedule_days::Model>,
        )>,
    >
    where
        C: sea_orm::ConnectionTrait;

    /// スケジュールと曜日を作成（トランザクション使用）
    async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        params: CreateScheduleParams,
    ) -> Result<(
        battle_recruitment_schedules::Model,
        Vec<battle_recruitment_schedule_days::Model>,
    )>;

    /// スケジュール削除（トランザクション使用、CASCADE により曜日も削除）
    async fn delete_with_txn(&self, txn: &DatabaseTransaction, id: i32) -> Result<()>;

    /// 有効/無効の切り替え（トランザクション使用）
    async fn toggle_enabled_with_txn(
        &self,
        txn: &DatabaseTransaction,
        id: i32,
        is_enabled: bool,
    ) -> Result<battle_recruitment_schedules::Model>;
}
