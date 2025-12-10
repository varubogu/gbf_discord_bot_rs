use crate::repository::database::battle_recruitments_repository::BattleRecruitmentsRepositoryImpl;
use crate::repository::database::recruitment_participants_repository::RecruitmentParticipantsRepositoryImpl;

/// Repository層のコンテナ
///
/// ステートレスなRepositoryインスタンスを提供します。
/// Repositoryはフィールドを持たず、DB接続は呼び出し時にパラメータとして渡されます。
#[derive(Debug)]
pub struct RepositoryContainer {
    battle_recruitment_repo: BattleRecruitmentsRepositoryImpl,
    recruitment_participants_repo: RecruitmentParticipantsRepositoryImpl,
    // 他のrepositoryも追加可能
}

impl RepositoryContainer {
    /// RepositoryContainerを作成
    ///
    /// # 戻り値
    /// 新しいRepositoryContainerインスタンス
    pub fn new() -> Self {
        let battle_recruitment_repo = BattleRecruitmentsRepositoryImpl::new();
        let recruitment_participants_repo = RecruitmentParticipantsRepositoryImpl::new();

        Self {
            battle_recruitment_repo,
            recruitment_participants_repo,
        }
    }

    /// BattleRecruitmentRepositoryへの参照を取得
    pub fn battle_recruitment(&self) -> &BattleRecruitmentsRepositoryImpl {
        &self.battle_recruitment_repo
    }

    /// RecruitmentParticipantsRepositoryへの参照を取得
    pub fn recruitment_participants(&self) -> &RecruitmentParticipantsRepositoryImpl {
        &self.recruitment_participants_repo
    }
}
