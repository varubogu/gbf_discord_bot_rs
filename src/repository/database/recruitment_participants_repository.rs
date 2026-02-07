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
#[derive(Debug, Clone, Copy)]
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

    /// 特定の参加レコードを取得（内部共通実装）
    async fn get_by_user_and_element_internal<'c, C>(
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

    /// 募集に参加しているユニークユーザー数を取得（内部共通実装）
    async fn count_unique_users_internal<'c, C>(db: &'c C, recruitment_id: i32) -> Result<i64>
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

    /// ユーザーが参加している属性のリストを取得（内部共通実装）
    async fn get_user_elements_internal<'c, C>(
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

    /// 募集に参加している全ユーザーのIDリストを取得（内部共通実装）
    async fn get_all_participant_user_ids_internal<'c, C>(
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

    /// 募集IDで参加者一覧を取得（内部共通実装）
    async fn find_by_recruitment_id_internal<'c, C>(
        db: &'c C,
        recruitment_id: i32,
    ) -> Result<Vec<crate::models::entities::worker::recruitment_participants::Model>>
    where
        C: sea_orm::ConnectionTrait,
    {
        RecruitmentParticipantEntity::find()
            .filter(Column::RecruitmentId.eq(recruitment_id))
            .all(db)
            .await
            .map_err(AppError::Database)
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

    async fn get_by_user_and_element_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        user_id: u64,
        element_id: Option<i32>,
    ) -> Result<bool> {
        Self::get_by_user_and_element_internal(txn, recruitment_id, user_id, element_id).await
    }

    async fn get_by_user_and_element_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        recruitment_id: i32,
        user_id: u64,
        element_id: Option<i32>,
    ) -> Result<bool> {
        Self::get_by_user_and_element_internal(db, recruitment_id, user_id, element_id).await
    }

    async fn count_unique_users_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<i64> {
        Self::count_unique_users_internal(txn, recruitment_id).await
    }

    async fn count_unique_users_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        recruitment_id: i32,
    ) -> Result<i64> {
        Self::count_unique_users_internal(db, recruitment_id).await
    }

    async fn get_user_elements_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
        user_id: u64,
    ) -> Result<Vec<Option<i32>>> {
        Self::get_user_elements_internal(txn, recruitment_id, user_id).await
    }

    async fn get_user_elements_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        recruitment_id: i32,
        user_id: u64,
    ) -> Result<Vec<Option<i32>>> {
        Self::get_user_elements_internal(db, recruitment_id, user_id).await
    }

    async fn get_all_participant_user_ids_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<Vec<u64>> {
        Self::get_all_participant_user_ids_internal(txn, recruitment_id).await
    }

    async fn get_all_participant_user_ids_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        recruitment_id: i32,
    ) -> Result<Vec<u64>> {
        Self::get_all_participant_user_ids_internal(db, recruitment_id).await
    }

    async fn find_by_recruitment_id_with_txn(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<Vec<crate::models::entities::worker::recruitment_participants::Model>> {
        Self::find_by_recruitment_id_internal(txn, recruitment_id).await
    }

    async fn find_by_recruitment_id_with_db(
        &self,
        db: &sea_orm::DatabaseConnection,
        recruitment_id: i32,
    ) -> Result<Vec<crate::models::entities::worker::recruitment_participants::Model>> {
        Self::find_by_recruitment_id_internal(db, recruitment_id).await
    }
}
