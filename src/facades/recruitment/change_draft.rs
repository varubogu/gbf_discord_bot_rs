//! 募集変更パネルの下書きを操作するFacade。
//!
//! 一時状態へのアクセスをイベント層から隠蔽し、状態管理の窓口をFacadeに限定する。

use crate::types::{AppState, RecruitChangeDraft, RecruitChangeDraftKey};

/// 募集変更下書きのユースケースを提供するFacade。
pub struct RecruitChangeDraftFacade;

impl RecruitChangeDraftFacade {
    /// 指定した操作の下書きを取得する。
    pub async fn get(app_state: &AppState, key: &RecruitChangeDraftKey) -> RecruitChangeDraft {
        app_state.recruit_change_drafts().get(key).await
    }

    /// 指定した操作の下書きを更新する。
    pub async fn update(
        app_state: &AppState,
        key: RecruitChangeDraftKey,
        update: impl FnOnce(&mut RecruitChangeDraft),
    ) {
        app_state.recruit_change_drafts().update(key, update).await;
    }

    /// 指定した操作の下書きを破棄する。
    pub async fn remove(app_state: &AppState, key: &RecruitChangeDraftKey) {
        app_state.recruit_change_drafts().remove(key).await;
    }
}
