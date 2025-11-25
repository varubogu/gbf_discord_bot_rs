use crate::models::entities::guilds;
use crate::types::Result;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, Set};
use tracing::{debug, error, info};

/// guildsテーブルのRepository
pub struct GuildRepository {
    db: DatabaseConnection,
}

impl GuildRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// ギルドを登録または更新（トランザクション内）
    /// ギルドが既に存在する場合は名前のみ更新
    pub async fn upsert_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        name: String,
    ) -> Result<guilds::Model> {
        debug!(guild_id = guild_id, name = %name, "ギルドを登録または更新します");

        let now = chrono::Utc::now();

        // 既存のギルドを確認
        let existing_guild = guilds::Entity::find_by_id(guild_id).one(txn).await?;

        let model = if let Some(existing) = existing_guild {
            // 既存のギルドが存在する場合、名前のみ更新
            let mut active_model: guilds::ActiveModel = existing.into();
            active_model.name = Set(name.clone());
            active_model.updated_at = Set(now);

            active_model.update(txn).await.map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "ギルドの更新に失敗しました"
                );
                e
            })?
        } else {
            // 新規ギルドを作成
            let active_model = guilds::ActiveModel {
                guild_id: Set(guild_id),
                name: Set(name.clone()),
                recruit_channel_id: Set(None),
                timezone: Set(None),
                default_recruit_duration: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
            };

            active_model.insert(txn).await.map_err(|e| {
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "ギルドの登録に失敗しました"
                );
                e
            })?
        };

        info!(guild_id = guild_id, name = %name, "ギルドを登録または更新しました");

        Ok(model)
    }

    /// ギルドIDでギルドを取得
    pub async fn get_by_id(&self, guild_id: i64) -> Result<Option<guilds::Model>> {
        debug!(guild_id = guild_id, "ギルドを取得します");

        let model = guilds::Entity::find_by_id(guild_id)
            .one(&self.db)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id = guild_id, "ギルドの取得に失敗しました");
                e
            })?;

        Ok(model)
    }

    /// ギルドが存在するか確認
    pub async fn exists(&self, guild_id: i64) -> Result<bool> {
        let guild = self.get_by_id(guild_id).await?;
        Ok(guild.is_some())
    }
}
