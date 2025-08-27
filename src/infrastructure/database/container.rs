use crate::repository::database::battle_recruitments_repository::BattleRecruitmentsRepositoryImpl;
use sea_orm::DatabaseConnection;

/// Repository層のコンテナ
///
/// AppStateから共有のDB接続を受け取り、Repository群を管理します。
/// 従来のDIコンテナパターンから、よりシンプルで効率的なアプローチに変更。
#[derive(Debug)]
pub struct RepositoryContainer {
    battle_recruitment_repo: BattleRecruitmentsRepositoryImpl,
    // 他のrepositoryも追加可能
}

impl RepositoryContainer {
    /// 共有DB接続を使用してRepositoryContainerを作成
    ///
    /// # 引数
    /// * `db_connection` - 共有されるDB接続
    ///
    /// # 戻り値
    /// 新しいRepositoryContainerインスタンス
    pub fn new(db_connection: &DatabaseConnection) -> Self {
        let battle_recruitment_repo = BattleRecruitmentsRepositoryImpl::new(db_connection.clone());

        Self {
            battle_recruitment_repo,
        }
    }

    /// BattleRecruitmentRepositoryへの参照を取得
    pub fn battle_recruitment(&self) -> &BattleRecruitmentsRepositoryImpl {
        &self.battle_recruitment_repo
    }
}
