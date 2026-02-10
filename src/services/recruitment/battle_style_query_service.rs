use crate::models::battle_styles::BattleStyle;
use crate::repository::BattleStyleRepository;
use crate::types::{AppError, Result};
use sea_orm::DatabaseConnection;
use tracing::debug;

/// 攻略方法クエリService
/// 攻略方法検索・取得の責務を持つ
pub struct BattleStyleQueryService<B>
where
    B: BattleStyleRepository,
{
    repository: B,
}

impl<B> BattleStyleQueryService<B>
where
    B: BattleStyleRepository,
{
    pub fn new(repository: B) -> Self {
        Self { repository }
    }

    /// すべての攻略方法を取得
    pub async fn get_all_battle_styles(&self, db: &DatabaseConnection) -> Result<Vec<BattleStyle>> {
        let models = self.repository.get_all(db).await?;
        let battle_styles: Vec<BattleStyle> = models.into_iter().map(|m| m.into()).collect();

        debug!(count = battle_styles.len(), "攻略方法一覧を取得しました");

        Ok(battle_styles)
    }

    /// 攻略方法IDで取得
    pub async fn get_battle_style_by_id(
        &self,
        db: &DatabaseConnection,
        battle_style_id: i32,
    ) -> Result<BattleStyle> {
        let model = self
            .repository
            .get_by_id(db, battle_style_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "攻略方法ID {battle_style_id} が見つかりませんでした"
                ))
            })?;

        let battle_style: BattleStyle = model.into();

        debug!(battle_style_id = battle_style_id, "攻略方法を取得しました");

        Ok(battle_style)
    }
}
