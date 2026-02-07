use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::GuildQuestDisableRepository;
use crate::repository::QuestRepository;
use crate::types::{PoiseContext, Result};
use poise::ChoiceParameter;
use sea_orm::TransactionTrait;

#[derive(Debug, Clone, Copy, ChoiceParameter)]
pub enum QuestFilterType {
    #[name = "All quests"]
    #[name = "全て"]
    All,
    #[name = "Enabled only"]
    #[name = "有効のみ"]
    EnabledOnly,
    #[name = "Disabled only"]
    #[name = "無効のみ"]
    DisabledOnly,
}

/// クエスト一覧を表示
///
/// クエストの一覧を表示します。有効/無効で絞り込むことができます。
#[poise::command(
    slash_command,
    guild_only,
    ephemeral = true,
    rename = "quest_list",
    name_localized("ja", "クエスト一覧"),
    description_localized("ja", "クエストの一覧を表示します。（gbf_bot_controlロール必須）")
)]
pub async fn quest_list(
    ctx: PoiseContext<'_>,
    #[name_localized("ja", "絞り込み")]
    #[description = "Filter type"]
    #[description_localized("ja", "有効/無効で絞り込み")]
    filter: Option<QuestFilterType>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます。".to_string(),
        })?
        .get() as i64;

    let app_state = &ctx.data().app_state;
    let db_conn = app_state.guild_db();
    let quest_repository = app_state.repositories.quest;
    let disable_repository = app_state.repositories.guild_quest_disable;

    // トランザクション開始
    let txn = db_conn.begin().await?;

    // RLSポリシー用にセッション変数を設定
    set_current_guild_id(&txn, guild_id).await?;

    let filter_type = filter.unwrap_or(QuestFilterType::All);

    let message = match filter_type {
        QuestFilterType::All => {
            // 全クエストを取得
            let all_quests = quest_repository.get_all(&txn).await?;
            let disabled_ids = disable_repository
                .get_disabled_quest_ids(&txn, guild_id)
                .await?;

            // クエスト名と有効/無効のリストを作成
            let mut lines = vec!["# クエスト一覧".to_string(), "".to_string()];

            for quest in all_quests.iter().take(100) {
                let status = if disabled_ids.contains(&quest.id) {
                    "❌ 無効"
                } else {
                    "✅ 有効"
                };
                lines.push(format!("{} {}", status, quest.name));
            }

            if all_quests.len() > 100 {
                lines.push(format!("\n...他{}件", all_quests.len() - 100));
            }

            lines.join("\n")
        }
        QuestFilterType::EnabledOnly => {
            // 有効なクエストのみ取得
            let results = quest_repository
                .search_enabled_quests(&txn, guild_id, "")
                .await?;

            let mut lines = vec!["# 有効なクエスト一覧".to_string(), "".to_string()];

            for result in results.iter().take(100) {
                lines.push(format!("✅ {}", result.name));
            }

            if results.len() > 100 {
                lines.push(format!("\n...他{}件", results.len() - 100));
            }

            if results.is_empty() {
                lines.push("有効なクエストはありません。".to_string());
            }

            lines.join("\n")
        }
        QuestFilterType::DisabledOnly => {
            // 無効なクエストのみ取得
            let results = quest_repository
                .search_disabled_quests(&txn, guild_id, "")
                .await?;

            let mut lines = vec!["# 無効なクエスト一覧".to_string(), "".to_string()];

            for result in results.iter().take(100) {
                lines.push(format!("❌ {}", result.name));
            }

            if results.len() > 100 {
                lines.push(format!("\n...他{}件", results.len() - 100));
            }

            if results.is_empty() {
                lines.push("無効なクエストはありません。".to_string());
            }

            lines.join("\n")
        }
    };

    txn.rollback().await?;

    ctx.say(message).await?;

    Ok(())
}
