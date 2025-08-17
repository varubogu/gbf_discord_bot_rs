use crate::infrastructure::database::connection::DatabaseConnectionManager;
use crate::repository::BattleRecruitmentRepository;
use crate::repository::database::battle_recruitment_repository::BattleRecruitmentRepositoryImpl;
use crate::types::PoiseError;
use std::sync::Arc;

/// Repository層の依存注入コンテナ
pub struct RepositoryContainer {
    pub battle_recruitment_repo: Arc<dyn BattleRecruitmentRepository>,
    // 他のrepositoryも追加可能
}

impl RepositoryContainer {
    pub async fn new() -> Result<Self, PoiseError> {
        let db_manager = DatabaseConnectionManager::new().await?;

        let battle_recruitment_repo = Arc::new(BattleRecruitmentRepositoryImpl::new(
            db_manager.connection().clone(),
        ));

        Ok(Self {
            battle_recruitment_repo,
        })
    }
}
