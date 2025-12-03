// use crate::models::quests::Quest;
// use crate::types::Result;
// use async_trait::async_trait;

// /// クエストリポジトリの抽象インターフェース
// /// データベースアクセスの詳細を隠蔽し、「データを保存する何か」への依存のみ提供
// #[async_trait]
// pub trait QuestAliasesRepository: Send + Sync {
//     /// 全クエストを取得
//     async fn get_all(&self) -> Result<Vec<Quest>>;

//     /// ターゲットIDでクエストを検索
//     async fn get_by_target_id(&self, target_id: i32) -> Result<Option<Quest>>;
// }
