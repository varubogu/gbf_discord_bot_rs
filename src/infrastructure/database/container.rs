use crate::repository::database::battle_recruitments_repository::BattleRecruitmentsRepositoryImpl;

/// Repository層のコンテナ
///
/// ステートレスなRepositoryインスタンスを提供します。
/// Repositoryはフィールドを持たず、DB接続は呼び出し時にパラメータとして渡されます。
#[derive(Debug)]
pub struct RepositoryContainer {
    battle_recruitment_repo: BattleRecruitmentsRepositoryImpl,
    // 他のrepositoryも追加可能
}

impl RepositoryContainer {
    /// RepositoryContainerを作成
    ///
    /// # 戻り値
    /// 新しいRepositoryContainerインスタンス
    pub fn new() -> Self {
        let battle_recruitment_repo = BattleRecruitmentsRepositoryImpl::new();

        Self {
            battle_recruitment_repo,
        }
    }

    /// BattleRecruitmentRepositoryへの参照を取得
    pub fn battle_recruitment(&self) -> &BattleRecruitmentsRepositoryImpl {
        &self.battle_recruitment_repo
    }
}
