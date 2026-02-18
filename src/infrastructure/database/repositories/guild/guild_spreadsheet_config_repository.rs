use crate::errors::RepositoryError;
use crate::models::entities::guild_master::{guild_spreadsheet_exports, guild_spreadsheet_imports};
use crate::repository::GuildSpreadsheetConfigRepositoryTrait;
use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};

#[derive(Clone, Default)]
pub struct SeaOrmGuildSpreadsheetConfigRepository;

impl SeaOrmGuildSpreadsheetConfigRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GuildSpreadsheetConfigRepositoryTrait for SeaOrmGuildSpreadsheetConfigRepository {
    async fn find_import_spreadsheet_id<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
    ) -> Result<Option<String>, RepositoryError>
    where
        C: sea_orm::ConnectionTrait,
    {
        let result = guild_spreadsheet_imports::Entity::find()
            .filter(guild_spreadsheet_imports::Column::GuildId.eq(guild_id))
            .one(db)
            .await?;

        Ok(result.map(|model| model.spreadsheet_id))
    }

    async fn find_export_spreadsheet_id<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
    ) -> Result<Option<String>, RepositoryError>
    where
        C: sea_orm::ConnectionTrait,
    {
        let result = guild_spreadsheet_exports::Entity::find()
            .filter(guild_spreadsheet_exports::Column::GuildId.eq(guild_id))
            .one(db)
            .await?;

        Ok(result.map(|model| model.spreadsheet_id))
    }

    async fn upsert_import_spreadsheet_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        spreadsheet_id: &str,
    ) -> Result<(), RepositoryError> {
        let now = chrono::Utc::now();

        let active_model = guild_spreadsheet_imports::ActiveModel {
            guild_id: ActiveValue::Set(guild_id),
            spreadsheet_id: ActiveValue::Set(spreadsheet_id.to_string()),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };

        guild_spreadsheet_imports::Entity::insert(active_model)
            .on_conflict(
                OnConflict::column(guild_spreadsheet_imports::Column::GuildId)
                    .update_columns([
                        guild_spreadsheet_imports::Column::SpreadsheetId,
                        guild_spreadsheet_imports::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(txn)
            .await?;

        Ok(())
    }

    async fn upsert_export_spreadsheet_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        spreadsheet_id: &str,
    ) -> Result<(), RepositoryError> {
        let now = chrono::Utc::now();

        let active_model = guild_spreadsheet_exports::ActiveModel {
            guild_id: ActiveValue::Set(guild_id),
            spreadsheet_id: ActiveValue::Set(spreadsheet_id.to_string()),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };

        guild_spreadsheet_exports::Entity::insert(active_model)
            .on_conflict(
                OnConflict::column(guild_spreadsheet_exports::Column::GuildId)
                    .update_columns([
                        guild_spreadsheet_exports::Column::SpreadsheetId,
                        guild_spreadsheet_exports::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(txn)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_trait_implementation() {
        fn assert_impl<T: GuildSpreadsheetConfigRepositoryTrait>() {}
        assert_impl::<SeaOrmGuildSpreadsheetConfigRepository>();
    }
}
