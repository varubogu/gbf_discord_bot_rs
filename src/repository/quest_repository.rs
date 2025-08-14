use async_trait::async_trait;
use crate::types::PoiseError;
use crate::models::quest::{Quest, QuestAlias};

/// クエストリポジトリの抽象インターフェース
/// データベースアクセスの詳細を隠蔽し、「データを保存する何か」への依存のみ提供
#[async_trait]
pub trait QuestRepository: Send + Sync {
    /// 全クエストを取得
    async fn get_all(&self) -> Result<Vec<Quest>, PoiseError>;
    
    /// 全クエストエイリアスを取得
    async fn get_aliases(&self) -> Result<Vec<QuestAlias>, PoiseError>;
    
    /// エイリアスでクエストを検索
    async fn get_by_alias(&self, alias: &str) -> Result<Option<Quest>, PoiseError>;
    
    /// ターゲットIDでクエストを検索
    async fn get_by_target_id(&self, target_id: i32) -> Result<Option<Quest>, PoiseError>;
}