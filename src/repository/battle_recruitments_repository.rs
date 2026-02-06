use crate::models::battle_recruitments::BattleRecruitments;
use crate::types::Result;
use crate::types::discord::{DiscordChannelId, DiscordGuildId, DiscordMessageId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// バトル募集作成パラメータ
/// Discord識別子にはドメイン型を使用し、Repository層の外部依存を排除
pub struct CreateBattleRecruitmentParams {
    pub guild_id: DiscordGuildId,
    pub channel_id: DiscordChannelId,
    pub message_id: DiscordMessageId,
    pub quest_id: i32,
    pub battle_style_id: i32,
    pub quest_start_at: DateTime<Utc>,
}

/// バトル募集リポジトリの抽象インターフェース
/// データベースアクセスの詳細を隠蔽し、「データを保存する何か」への依存のみ提供
#[async_trait]
pub trait BattleRecruitmentsRepository: Send + Sync + std::fmt::Debug {
    /// 新規募集を作成（トランザクション対応）
    async fn create_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        params: CreateBattleRecruitmentParams,
    ) -> Result<BattleRecruitments>;

    /// メッセージIDで募集を取得
    async fn get_by_message<'c, C>(
        &self,
        db: &'c C,
        guild_id: DiscordGuildId,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
    ) -> Result<Option<BattleRecruitments>>
    where
        C: sea_orm::ConnectionTrait;

    /// メッセージIDで募集を取得（トランザクション対応）
    async fn get_by_message_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        guild_id: DiscordGuildId,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
    ) -> Result<Option<BattleRecruitments>>;

    /// IDで募集を取得（トランザクション対応）
    async fn get_by_id_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<Option<BattleRecruitments>>;

    /// 募集終了メッセージを更新
    async fn set_end_message<'c, C>(
        &self,
        db: &'c C,
        recruitment_id: i32,
        message_id: DiscordMessageId,
    ) -> Result<()>
    where
        C: sea_orm::ConnectionTrait;

    /// 募集をキャンセル済み状態に更新（トランザクション対応）
    async fn set_canceled_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        message_id: DiscordMessageId,
    ) -> Result<()>;

    /// 募集情報を更新（トランザクション対応）
    async fn update_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        quest_id: i32,
        battle_style_id: i32,
        quest_start_at: DateTime<Utc>,
    ) -> Result<()>;

    /// 規定人数到達通知フラグを更新（トランザクション対応）
    async fn set_full_notification_sent_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        sent: bool,
    ) -> Result<()>;

    /// メッセージIDを更新
    async fn update_message_id<'c, C>(
        &self,
        db: &'c C,
        recruitment_id: i32,
        message_id: DiscordMessageId,
    ) -> Result<()>
    where
        C: sea_orm::ConnectionTrait;
}
