use crate::models::entities::guild_master::guild_environments::{
    self, Entity as GuildEnvironmentEntity,
};
use crate::models::guild_environments::GuildEnvironments;
use crate::repository::GuildEnvironmentRepository;
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, DbErr, EntityTrait, QueryFilter, Set,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct SeaOrmGuildEnvironmentRepository;

impl SeaOrmGuildEnvironmentRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GuildEnvironmentRepository for SeaOrmGuildEnvironmentRepository {
    async fn get_by_guild_and_key<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
        key: &str,
    ) -> Result<Option<GuildEnvironments>, DbErr>
    where
        C: sea_orm::ConnectionTrait,
    {
        let model = GuildEnvironmentEntity::find()
            .filter(guild_environments::Column::GuildId.eq(guild_id))
            .filter(guild_environments::Column::Key.eq(key))
            .one(db)
            .await?;

        Ok(model.map(|env| env.into()))
    }

    async fn get_multiple_by_guild<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
        keys: &[&str],
    ) -> Result<HashMap<String, String>, DbErr>
    where
        C: sea_orm::ConnectionTrait,
    {
        let keys_vec: Vec<String> = keys.iter().map(|k| k.to_string()).collect();
        let models = GuildEnvironmentEntity::find()
            .filter(guild_environments::Column::GuildId.eq(guild_id))
            .filter(guild_environments::Column::Key.is_in(keys_vec))
            .all(db)
            .await?;

        let map = models.into_iter().map(|m| (m.key, m.value)).collect();

        Ok(map)
    }

    async fn set_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        key: &str,
        value: &str,
    ) -> Result<GuildEnvironments, DbErr> {
        // 既存の環境変数を検索
        let existing = GuildEnvironmentEntity::find()
            .filter(guild_environments::Column::GuildId.eq(guild_id))
            .filter(guild_environments::Column::Key.eq(key))
            .one(txn)
            .await?;

        let result = if let Some(existing_env) = existing {
            // 既存の環境変数を更新
            let mut active_model: guild_environments::ActiveModel = existing_env.into();
            active_model.value = Set(value.to_string());
            active_model.updated_at = Set(chrono::Utc::now());

            active_model.update(txn).await?
        } else {
            // 新規環境変数を作成
            let new_env = guild_environments::ActiveModel {
                guild_id: Set(guild_id),
                key: Set(key.to_string()),
                value: Set(value.to_string()),
                created_at: Set(chrono::Utc::now()),
                updated_at: Set(chrono::Utc::now()),
            };

            new_env.insert(txn).await?
        };

        Ok(result.into())
    }
}
