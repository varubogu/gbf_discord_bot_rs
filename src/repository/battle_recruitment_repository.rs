use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::types::PoiseError;
use crate::models::battle_recruitment::BattleRecruitment;

/// バトル募集リポジトリの抽象インターフェース
/// データベースアクセスの詳細を隠蔽し、「データを保存する何か」への依存のみ提供
#[async_trait]
pub trait BattleRecruitmentRepository: Send + Sync {
    /// 新規募集を作成
    async fn create(
        &self, 
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: DateTime<Utc>,
    ) -> Result<BattleRecruitment, PoiseError>;

    /// メッセージIDで募集を取得
    async fn get_by_message(
        &self, 
        guild_id: i64, 
        channel_id: i64, 
        message_id: i64
    ) -> Result<Option<BattleRecruitment>, PoiseError>;

    /// 募集終了メッセージを更新
    async fn set_end_message(
        &self, 
        recruitment_id: i32, 
        message_id: i64
    ) -> Result<(), PoiseError>;

}