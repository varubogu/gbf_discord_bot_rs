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

    /// メッセージIDで募集を取得（トランザクション対応）
    async fn get_by_message_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        guild_id: DiscordGuildId,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
    ) -> Result<Option<BattleRecruitments>>;

    /// メッセージIDで募集を取得（通常接続）
    async fn get_by_message_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
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

    /// 募集終了メッセージを更新（トランザクション対応）
    async fn set_end_message_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        message_id: DiscordMessageId,
    ) -> Result<()>;

    /// 募集終了メッセージを更新（通常接続）
    async fn set_end_message_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        recruitment_id: i32,
        message_id: DiscordMessageId,
    ) -> Result<()>;

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

    /// メッセージIDを更新（トランザクション対応）
    async fn update_message_id_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        message_id: DiscordMessageId,
    ) -> Result<()>;

    /// メッセージIDを更新（通常接続）
    async fn update_message_id_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        recruitment_id: i32,
        message_id: DiscordMessageId,
    ) -> Result<()>;

    /// 指定日時より前の募集を削除（クリーンアップ用）
    async fn delete_before_date_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        before: DateTime<Utc>,
    ) -> Result<u64>;

    /// ギルド内の募集中バトル一覧を取得（トランザクション対応）
    ///
    /// is_recruiting=true, is_canceled=false, message_id!=0 かつ
    /// quest_start_at が現在以降の件を quest_start_at 昇順で返す。
    /// 呼び出し前に set_current_guild_id で RLS 設定が必要。
    async fn get_active_by_guild_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<BattleRecruitments>>;
}

/// Arc<T>に対するBattleRecruitmentsRepositoryの実装
/// これによりArc<ConcreteRepository>を直接使用できる
#[async_trait]
impl<T> BattleRecruitmentsRepository for std::sync::Arc<T>
where
    T: BattleRecruitmentsRepository + ?Sized,
{
    async fn create_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        params: CreateBattleRecruitmentParams,
    ) -> Result<BattleRecruitments> {
        (**self).create_with_txn(txn, params).await
    }

    async fn get_by_message_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        guild_id: DiscordGuildId,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
    ) -> Result<Option<BattleRecruitments>> {
        (**self)
            .get_by_message_with_txn(txn, guild_id, channel_id, message_id)
            .await
    }

    async fn get_by_message_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        guild_id: DiscordGuildId,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
    ) -> Result<Option<BattleRecruitments>> {
        (**self)
            .get_by_message_with_db(db, guild_id, channel_id, message_id)
            .await
    }

    async fn get_by_id_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<Option<BattleRecruitments>> {
        (**self).get_by_id_with_txn(txn, recruitment_id).await
    }

    async fn set_end_message_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        message_id: DiscordMessageId,
    ) -> Result<()> {
        (**self)
            .set_end_message_with_txn(txn, recruitment_id, message_id)
            .await
    }

    async fn set_end_message_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        recruitment_id: i32,
        message_id: DiscordMessageId,
    ) -> Result<()> {
        (**self)
            .set_end_message_with_db(db, recruitment_id, message_id)
            .await
    }

    async fn set_canceled_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        message_id: DiscordMessageId,
    ) -> Result<()> {
        (**self)
            .set_canceled_with_txn(txn, recruitment_id, message_id)
            .await
    }

    async fn update_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        quest_id: i32,
        battle_style_id: i32,
        quest_start_at: DateTime<Utc>,
    ) -> Result<()> {
        (**self)
            .update_with_txn(
                txn,
                recruitment_id,
                quest_id,
                battle_style_id,
                quest_start_at,
            )
            .await
    }

    async fn set_full_notification_sent_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        sent: bool,
    ) -> Result<()> {
        (**self)
            .set_full_notification_sent_with_txn(txn, recruitment_id, sent)
            .await
    }

    async fn update_message_id_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        message_id: DiscordMessageId,
    ) -> Result<()> {
        (**self)
            .update_message_id_with_txn(txn, recruitment_id, message_id)
            .await
    }

    async fn update_message_id_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        recruitment_id: i32,
        message_id: DiscordMessageId,
    ) -> Result<()> {
        (**self)
            .update_message_id_with_db(db, recruitment_id, message_id)
            .await
    }

    async fn delete_before_date_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        before: DateTime<Utc>,
    ) -> Result<u64> {
        (**self).delete_before_date_with_txn(txn, before).await
    }

    async fn get_active_by_guild_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<BattleRecruitments>> {
        (**self).get_active_by_guild_with_txn(txn, guild_id).await
    }
}
