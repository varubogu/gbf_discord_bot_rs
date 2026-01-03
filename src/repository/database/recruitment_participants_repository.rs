use crate::models::entities::worker::recruitment_participants::{
    ActiveModel, Column, Entity as RecruitmentParticipantEntity,
};
use crate::repository::RecruitmentParticipantsRepository;
use crate::types::{AppError, Result};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, QuerySelect, Set,
};

/// SeaORM を使用した募集参加者リポジトリの実装
#[derive(Debug)]
pub struct SeaOrmRecruitmentParticipantsRepository;

impl Default for SeaOrmRecruitmentParticipantsRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SeaOrmRecruitmentParticipantsRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RecruitmentParticipantsRepository for SeaOrmRecruitmentParticipantsRepository {
    async fn insert_with_txn(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        user_id: u64,
        element_id: Option<i32>,
    ) -> Result<bool> {
        // 既に存在するかチェック
        let exists = RecruitmentParticipantEntity::find()
            .filter(Column::RecruitmentId.eq(recruitment_id))
            .filter(Column::UserId.eq(user_id as i64))
            .filter(match element_id {
                Some(id) => Column::ElementId.eq(id),
                None => Column::ElementId.is_null(),
            })
            .one(txn)
            .await
            .map_err(AppError::Database)?;

        if exists.is_some() {
            return Ok(false); // 既に存在する場合は false を返す
        }

        // 新規作成
        let mut active_model = ActiveModel::new();
        active_model.recruitment_id = Set(recruitment_id);
        active_model.user_id = Set(user_id as i64);
        active_model.element_id = Set(element_id);

        active_model.insert(txn).await.map_err(|e| match e {
            DbErr::RecordNotInserted => AppError::Business {
                message: "参加レコードの挿入に失敗しました".to_string(),
            },
            _ => AppError::Database(e),
        })?;

        Ok(true)
    }

    async fn delete_by_element_with_txn(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        user_id: u64,
        element_id: Option<i32>,
    ) -> Result<bool> {
        let delete_result = RecruitmentParticipantEntity::delete_many()
            .filter(Column::RecruitmentId.eq(recruitment_id))
            .filter(Column::UserId.eq(user_id as i64))
            .filter(match element_id {
                Some(id) => Column::ElementId.eq(id),
                None => Column::ElementId.is_null(),
            })
            .exec(txn)
            .await
            .map_err(AppError::Database)?;

        Ok(delete_result.rows_affected > 0)
    }

    async fn delete_all_by_user_with_txn(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        user_id: u64,
    ) -> Result<u64> {
        let delete_result = RecruitmentParticipantEntity::delete_many()
            .filter(Column::RecruitmentId.eq(recruitment_id))
            .filter(Column::UserId.eq(user_id as i64))
            .exec(txn)
            .await
            .map_err(AppError::Database)?;

        Ok(delete_result.rows_affected)
    }

    async fn get_by_user_and_element<'c, C>(
        &self,
        db: &'c C,
        recruitment_id: i32,
        user_id: u64,
        element_id: Option<i32>,
    ) -> Result<bool>
    where
        C: sea_orm::ConnectionTrait,
    {
        let result = RecruitmentParticipantEntity::find()
            .filter(Column::RecruitmentId.eq(recruitment_id))
            .filter(Column::UserId.eq(user_id as i64))
            .filter(match element_id {
                Some(id) => Column::ElementId.eq(id),
                None => Column::ElementId.is_null(),
            })
            .one(db)
            .await
            .map_err(AppError::Database)?;

        Ok(result.is_some())
    }

    async fn count_unique_users<'c, C>(&self, db: &'c C, recruitment_id: i32) -> Result<i64>
    where
        C: sea_orm::ConnectionTrait,
    {
        use sea_orm::sea_query::Expr;

        // SELECT COUNT(DISTINCT user_id) FROM recruitment_participants WHERE recruitment_id = ?
        let count = RecruitmentParticipantEntity::find()
            .filter(Column::RecruitmentId.eq(recruitment_id))
            .select_only()
            .column_as(Expr::cust("COUNT(DISTINCT user_id)"), "distinct_user_count")
            .into_tuple::<i64>()
            .one(db)
            .await
            .map_err(AppError::Database)?
            .unwrap_or(0);

        Ok(count)
    }

    async fn get_user_elements<'c, C>(
        &self,
        db: &'c C,
        recruitment_id: i32,
        user_id: u64,
    ) -> Result<Vec<Option<i32>>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let results = RecruitmentParticipantEntity::find()
            .filter(Column::RecruitmentId.eq(recruitment_id))
            .filter(Column::UserId.eq(user_id as i64))
            .all(db)
            .await
            .map_err(AppError::Database)?;

        Ok(results.into_iter().map(|model| model.element_id).collect())
    }

    async fn get_all_participant_user_ids<'c, C>(
        &self,
        db: &'c C,
        recruitment_id: i32,
    ) -> Result<Vec<u64>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let results = RecruitmentParticipantEntity::find()
            .filter(Column::RecruitmentId.eq(recruitment_id))
            .all(db)
            .await
            .map_err(AppError::Database)?;

        // user_idの重複を排除してu64に変換
        let user_ids: std::collections::HashSet<u64> = results
            .into_iter()
            .map(|model| model.user_id as u64)
            .collect();

        Ok(user_ids.into_iter().collect())
    }
}
