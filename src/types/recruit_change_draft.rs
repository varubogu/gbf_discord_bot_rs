//! 募集変更パネルの一時状態を管理する。
//!
//! 状態は`AppState`に所有させ、events層のグローバル変数を使用しない。

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// 募集変更下書きを一意に特定するキー
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecruitChangeDraftKey {
    pub user_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
}

/// 募集変更パネルで未確定の入力内容
#[derive(Debug, Clone, Default)]
pub struct RecruitChangeDraft {
    pub quest_name: Option<String>,
    pub battle_style_id: Option<i32>,
    pub battle_style_name: Option<String>,
    pub event_date: Option<DateTime<Utc>>,
}

/// 募集変更下書きの共有ストア
#[derive(Debug, Default)]
pub struct RecruitChangeDraftStore {
    drafts: RwLock<HashMap<RecruitChangeDraftKey, RecruitChangeDraft>>,
}

impl RecruitChangeDraftStore {
    /// 下書きを取得する。未作成の場合は空の下書きを返す。
    pub async fn get(&self, key: &RecruitChangeDraftKey) -> RecruitChangeDraft {
        self.drafts
            .read()
            .await
            .get(key)
            .cloned()
            .unwrap_or_default()
    }

    /// 下書きを更新する。
    pub async fn update(
        &self,
        key: RecruitChangeDraftKey,
        update: impl FnOnce(&mut RecruitChangeDraft),
    ) {
        let mut drafts = self.drafts.write().await;
        update(drafts.entry(key).or_default());
    }

    /// 下書きを破棄する。
    pub async fn remove(&self, key: &RecruitChangeDraftKey) {
        self.drafts.write().await.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn 下書きの生成_更新_破棄ができる() {
        let store = RecruitChangeDraftStore::default();
        let key = RecruitChangeDraftKey {
            user_id: 1,
            channel_id: 2,
            message_id: 3,
        };

        assert!(store.get(&key).await.quest_name.is_none());
        store
            .update(key.clone(), |draft| {
                draft.quest_name = Some("テストクエスト".to_string())
            })
            .await;
        assert_eq!(
            store.get(&key).await.quest_name.as_deref(),
            Some("テストクエスト")
        );

        store.remove(&key).await;
        assert!(store.get(&key).await.quest_name.is_none());
    }
}
