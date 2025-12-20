use crate::models::quests::Quest;
use crate::types::Result;
use async_trait::async_trait;

/// クエスト名とエイリアスの検索結果
#[derive(Debug, Clone)]
pub struct QuestSearchResult {
    pub quest_id: i32,
    pub name: String,
    pub matched_text: String, // マッチした名前またはエイリアス
}

/// クエストリポジトリの抽象インターフェース
/// データベースアクセスの詳細を隠蔽し、「データを保存する何か」への依存のみ提供
#[async_trait]
pub trait QuestRepository: Send + Sync {
    /// 全クエストを取得
    ///
    /// # Arguments
    /// * `db` - DatabaseConnection または DatabaseTransaction
    async fn get_all<'c, C>(&self, db: &'c C) -> Result<Vec<Quest>>
    where
        C: sea_orm::ConnectionTrait;

    /// ターゲットIDでクエストを検索
    ///
    /// # Arguments
    /// * `db` - DatabaseConnection または DatabaseTransaction
    /// * `target_id` - クエストID
    async fn get_by_target_id<'c, C>(&self, db: &'c C, target_id: i32) -> Result<Option<Quest>>
    where
        C: sea_orm::ConnectionTrait;

    /// クエスト名またはエイリアスで部分一致検索
    /// クエスト名とエイリアスの両方から検索し、一致したものを返す
    ///
    /// # Arguments
    /// * `db` - DatabaseConnection または DatabaseTransaction
    /// * `partial` - 検索文字列
    async fn search_by_name_or_alias<'c, C>(
        &self,
        db: &'c C,
        partial: &str,
    ) -> Result<Vec<QuestSearchResult>>
    where
        C: sea_orm::ConnectionTrait;
}
