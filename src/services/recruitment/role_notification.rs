use crate::repository::database::all_recruitment_notification_roles_repository::AllRecruitmentNotificationRolesRepository;
use crate::repository::database::quest_recruitment_notification_roles_repository::QuestRecruitmentNotificationRolesRepository;
use crate::types::Result;
use sea_orm::DatabaseTransaction;
use tracing::{debug, info};

/// ロール通知サービス
pub struct RoleNotificationService {
    all_roles_repo: AllRecruitmentNotificationRolesRepository,
    quest_roles_repo: QuestRecruitmentNotificationRolesRepository,
}

impl Default for RoleNotificationService {
    fn default() -> Self {
        Self::new()
    }
}

impl RoleNotificationService {
    pub fn new() -> Self {
        Self {
            all_roles_repo: AllRecruitmentNotificationRolesRepository::new(),
            quest_roles_repo: QuestRecruitmentNotificationRolesRepository::new(),
        }
    }

    /// 募集メッセージ用のロールメンション文字列を生成
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `guild_id` - ギルドID
    /// * `quest_id` - クエストID
    ///
    /// # 戻り値
    /// ロールメンション文字列（例: `<@&123456789> <@&987654321>`）
    pub async fn get_role_mentions(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
    ) -> Result<String> {
        debug!(
            guild_id = guild_id,
            quest_id = quest_id,
            "ロールメンション文字列を生成します"
        );

        // 全募集通知ロールを取得
        let all_roles = self
            .all_roles_repo
            .find_by_guild_with_txn(txn, guild_id)
            .await?;

        // クエスト別通知ロールを取得
        let quest_roles = self
            .quest_roles_repo
            .find_by_guild_and_quest_with_txn(txn, guild_id, quest_id)
            .await?;

        // メンション文字列を生成
        let mut mentions = Vec::new();

        // 全募集通知ロール（seq昇順）
        for role in all_roles {
            mentions.push(format!("<@&{}>", role.role_id));
        }

        // クエスト別通知ロール（seq昇順）
        for role in quest_roles {
            mentions.push(format!("<@&{}>", role.role_id));
        }

        let mention_str = mentions.join(" ");

        info!(
            guild_id = guild_id,
            quest_id = quest_id,
            mention_count = mentions.len(),
            "ロールメンション文字列を生成しました"
        );

        Ok(mention_str)
    }

    /// 全募集通知ロールを追加（重複チェック付き）
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `guild_id` - ギルドID
    /// * `role_id` - ロールID
    ///
    /// # 戻り値
    /// * `true` - 新規登録された
    /// * `false` - 既に登録済みでスキップされた
    pub async fn add_all_recruitment_role(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        role_id: i64,
    ) -> Result<bool> {
        // 重複チェック
        if self
            .all_roles_repo
            .exists_with_txn(txn, guild_id, role_id)
            .await?
        {
            info!(
                guild_id = guild_id,
                role_id = role_id,
                "全募集通知ロールは既に登録済みです（スキップ）"
            );
            return Ok(false);
        }

        // 新規登録
        self.all_roles_repo
            .create_with_txn(txn, guild_id, role_id)
            .await?;

        Ok(true)
    }

    /// クエスト別募集通知ロールを追加（重複チェック付き）
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `guild_id` - ギルドID
    /// * `quest_id` - クエストID
    /// * `role_id` - ロールID
    ///
    /// # 戻り値
    /// * `true` - 新規登録された
    /// * `false` - 既に登録済みでスキップされた
    pub async fn add_quest_recruitment_role(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
        role_id: i64,
    ) -> Result<bool> {
        // 重複チェック
        if self
            .quest_roles_repo
            .exists_with_txn(txn, guild_id, quest_id, role_id)
            .await?
        {
            info!(
                guild_id = guild_id,
                quest_id = quest_id,
                role_id = role_id,
                "クエスト別募集通知ロールは既に登録済みです（スキップ）"
            );
            return Ok(false);
        }

        // 新規登録
        self.quest_roles_repo
            .create_with_txn(txn, guild_id, quest_id, role_id)
            .await?;

        Ok(true)
    }

    /// 全募集通知ロールを削除
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `guild_id` - ギルドID
    /// * `role_id` - ロールID
    ///
    /// # 戻り値
    /// 削除された行数
    pub async fn remove_all_recruitment_role(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        role_id: i64,
    ) -> Result<u64> {
        self.all_roles_repo
            .delete_with_txn(txn, guild_id, role_id)
            .await
    }

    /// クエスト別募集通知ロールを削除
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `guild_id` - ギルドID
    /// * `quest_id` - クエストID
    /// * `role_id` - ロールID
    ///
    /// # 戻り値
    /// 削除された行数
    pub async fn remove_quest_recruitment_role(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
        role_id: i64,
    ) -> Result<u64> {
        self.quest_roles_repo
            .delete_with_txn(txn, guild_id, quest_id, role_id)
            .await
    }
}
