use crate::models::entities::guild_master::all_recruitment_notification_roles;
use crate::repository::AllRecruitmentNotificationRolesRepository;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, Set};
use tracing::{debug, error, info};

/// all_recruitment_notification_rolesテーブルのRepository
#[derive(Default)]
pub struct SeaOrmAllRecruitmentNotificationRolesRepository;

impl SeaOrmAllRecruitmentNotificationRolesRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AllRecruitmentNotificationRolesRepository for SeaOrmAllRecruitmentNotificationRolesRepository {
    /// 全募集通知ロールを登録（トランザクション内）
    async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        role_id: i64,
    ) -> Result<all_recruitment_notification_roles::Model> {
        debug!(
            guild_id = guild_id,
            role_id = role_id,
            "全募集通知ロールを登録します"
        );

        let now = chrono::Utc::now();

        let active_model = all_recruitment_notification_roles::ActiveModel {
            guild_id: Set(guild_id),
            seq: sea_orm::ActiveValue::NotSet, // AUTO_INCREMENTで自動設定
            role_id: Set(role_id),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let model = all_recruitment_notification_roles::Entity::insert(active_model)
            .exec_with_returning(txn)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    role_id = role_id,
                    "全募集通知ロールの登録に失敗しました"
                );
                e
            })?;

        info!(
            guild_id = guild_id,
            role_id = role_id,
            seq = model.seq,
            "全募集通知ロールを登録しました"
        );

        Ok(model)
    }

    /// 全募集通知ロールを削除（トランザクション内）
    async fn delete_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        role_id: i64,
    ) -> Result<u64> {
        debug!(
            guild_id = guild_id,
            role_id = role_id,
            "全募集通知ロールを削除します"
        );

        let result = all_recruitment_notification_roles::Entity::delete_many()
            .filter(all_recruitment_notification_roles::Column::GuildId.eq(guild_id))
            .filter(all_recruitment_notification_roles::Column::RoleId.eq(role_id))
            .exec(txn)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    role_id = role_id,
                    "全募集通知ロールの削除に失敗しました"
                );
                e
            })?;

        info!(
            guild_id = guild_id,
            role_id = role_id,
            deleted_count = result.rows_affected,
            "全募集通知ロールを削除しました"
        );

        Ok(result.rows_affected)
    }

    /// ギルドIDで全募集通知ロール一覧を取得（トランザクション内、seq昇順）
    async fn find_by_guild_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<all_recruitment_notification_roles::Model>> {
        debug!(
            guild_id = guild_id,
            "全募集通知ロール一覧を取得します（トランザクション内）"
        );

        let models = all_recruitment_notification_roles::Entity::find()
            .filter(all_recruitment_notification_roles::Column::GuildId.eq(guild_id))
            .order_by_asc(all_recruitment_notification_roles::Column::Seq)
            .all(txn)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "全募集通知ロール一覧の取得に失敗しました"
                );
                e
            })?;

        debug!(
            guild_id = guild_id,
            count = models.len(),
            "全募集通知ロール一覧を取得しました"
        );

        Ok(models)
    }

    /// 全募集通知ロールが存在するかチェック（トランザクション内）
    async fn exists_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        role_id: i64,
    ) -> Result<bool> {
        debug!(
            guild_id = guild_id,
            role_id = role_id,
            "全募集通知ロールの存在をチェックします"
        );

        let models = all_recruitment_notification_roles::Entity::find()
            .filter(all_recruitment_notification_roles::Column::GuildId.eq(guild_id))
            .filter(all_recruitment_notification_roles::Column::RoleId.eq(role_id))
            .all(txn)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    role_id = role_id,
                    "全募集通知ロールの存在チェックに失敗しました"
                );
                e
            })?;

        Ok(!models.is_empty())
    }
}
