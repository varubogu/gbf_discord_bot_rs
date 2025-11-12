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
    async fn get_all(&self) -> Result<Vec<Quest>>;

    /// ターゲットIDでクエストを検索
    async fn get_by_target_id(&self, target_id: i32) -> Result<Option<Quest>>;

    /// クエスト名またはエイリアスで部分一致検索
    /// クエスト名とエイリアスの両方から検索し、一致したものを返す
    async fn search_by_name_or_alias(&self, partial: &str) -> Result<Vec<QuestSearchResult>>;
}
