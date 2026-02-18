use crate::infrastructure::database::repositories::recruitment::{
    SeaOrmBattleRecruitmentsRepository, SeaOrmRecruitmentParticipantsRepository,
};

/// Repository層のコンテナ
///
/// ステートレスなRepositoryインスタンスを提供します。
/// Repositoryはフィールドを持たず、DB接続は呼び出し時にパラメータとして渡されます。
#[derive(Debug)]
pub struct RepositoryContainer {
    battle_recruitment_repo: SeaOrmBattleRecruitmentsRepository,
    recruitment_participants_repo: SeaOrmRecruitmentParticipantsRepository,
    // 他のrepositoryも追加可能
}

impl Default for RepositoryContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl RepositoryContainer {
    /// RepositoryContainerを作成
    ///
    /// # 戻り値
    /// 新しいRepositoryContainerインスタンス
    pub fn new() -> Self {
        let battle_recruitment_repo = SeaOrmBattleRecruitmentsRepository::new();
        let recruitment_participants_repo = SeaOrmRecruitmentParticipantsRepository::new();

        Self {
            battle_recruitment_repo,
            recruitment_participants_repo,
        }
    }

    /// BattleRecruitmentRepositoryへの参照を取得
    pub fn battle_recruitment(&self) -> &SeaOrmBattleRecruitmentsRepository {
        &self.battle_recruitment_repo
    }

    /// RecruitmentParticipantsRepositoryへの参照を取得
    pub fn recruitment_participants(&self) -> &SeaOrmRecruitmentParticipantsRepository {
        &self.recruitment_participants_repo
    }
}
