use crate::repository::RecruitmentParticipantsRepository;
use crate::types::{AppError, Result};
use sea_orm::DatabaseTransaction;
use std::sync::Arc;
use tracing::{debug, info};

/// ボタンによる募集参加の結果
#[derive(Debug, Clone, PartialEq)]
pub enum ParticipationAction {
    /// 参加した
    Joined,
    /// 退出した
    Left,
}

/// RecruitmentParticipantsService - ボタン方式の募集参加者管理を行うサービス
#[derive(Debug)]
pub struct RecruitmentParticipantsService<R: RecruitmentParticipantsRepository> {
    participants_repo: Arc<R>,
}

impl<R: RecruitmentParticipantsRepository> RecruitmentParticipantsService<R> {
    /// 新しいRecruitmentParticipantsServiceを作成（依存性注入）
    pub fn new(participants_repo: Arc<R>) -> Self {
        Self { participants_repo }
    }

    /// 募集への参加/退出をトグルする（トランザクション対応）
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `recruitment_id` - 募集ID
    /// * `user_id` - ユーザーID
    /// * `element_id` - 属性ID（None = シンプル参加または全属性可能）
    ///
    /// # 戻り値
    /// * `Ok(ParticipationAction::Joined)` - 参加した場合
    /// * `Ok(ParticipationAction::Left)` - 退出した場合
    pub async fn toggle_participation(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        user_id: u64,
        element_id: Option<i32>,
    ) -> Result<ParticipationAction> {
        debug!(
            recruitment_id = recruitment_id,
            user_id = user_id,
            element_id = ?element_id,
            "参加/退出トグル処理開始"
        );

        // 既に参加しているかチェック
        let is_participating = self
            .participants_repo
            .get_by_user_and_element(txn, recruitment_id, user_id, element_id)
            .await?;

        if is_participating {
            // 退出処理
            let deleted = self
                .participants_repo
                .delete_by_element_with_txn(txn, recruitment_id, user_id, element_id)
                .await?;

            if deleted {
                info!(
                    recruitment_id = recruitment_id,
                    user_id = user_id,
                    element_id = ?element_id,
                    "ユーザーが退出しました"
                );
                Ok(ParticipationAction::Left)
            } else {
                // 削除対象がなかった場合（競合など）
                Err(AppError::Business {
                    message: "参加レコードの削除に失敗しました".to_string(),
                })
            }
        } else {
            // 参加処理
            let inserted = self
                .participants_repo
                .insert_with_txn(txn, recruitment_id, user_id, element_id)
                .await?;

            if inserted {
                info!(
                    recruitment_id = recruitment_id,
                    user_id = user_id,
                    element_id = ?element_id,
                    "ユーザーが参加しました"
                );
                Ok(ParticipationAction::Joined)
            } else {
                // 既に存在していた場合（競合など）
                Err(AppError::Business {
                    message: "既に参加しています".to_string(),
                })
            }
        }
    }

    /// ユーザーのすべての参加をキャンセルする（トランザクション対応）
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `recruitment_id` - 募集ID
    /// * `user_id` - ユーザーID
    ///
    /// # 戻り値
    /// * `Ok(count)` - 削除されたレコード数
    pub async fn leave_all(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        user_id: u64,
    ) -> Result<u64> {
        debug!(
            recruitment_id = recruitment_id,
            user_id = user_id,
            "全参加取り消し処理開始"
        );

        let count = self
            .participants_repo
            .delete_all_by_user_with_txn(txn, recruitment_id, user_id)
            .await?;

        info!(
            recruitment_id = recruitment_id,
            user_id = user_id,
            count = count,
            "ユーザーのすべての参加を取り消しました"
        );

        Ok(count)
    }

    /// 募集に参加しているユニークユーザー数を取得
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `recruitment_id` - 募集ID
    ///
    /// # 戻り値
    /// * `Ok(count)` - ユニークユーザー数
    pub async fn count_unique_participants(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<i64> {
        let count = self
            .participants_repo
            .count_unique_users(txn, recruitment_id)
            .await?;

        debug!(
            recruitment_id = recruitment_id,
            count = count,
            "募集参加者数を取得しました"
        );

        Ok(count)
    }

    /// ユーザーが参加している属性のリストを取得
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `recruitment_id` - 募集ID
    /// * `user_id` - ユーザーID
    ///
    /// # 戻り値
    /// * `Ok(elements)` - 参加している属性のリスト
    pub async fn get_user_elements(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        user_id: u64,
    ) -> Result<Vec<Option<i32>>> {
        let elements = self
            .participants_repo
            .get_user_elements(txn, recruitment_id, user_id)
            .await?;

        debug!(
            recruitment_id = recruitment_id,
            user_id = user_id,
            elements = ?elements,
            "ユーザーの参加属性を取得しました"
        );

        Ok(elements)
    }
}

#[cfg(test)]
mod tests {}
