use crate::facades::recruitment::quest_management_facade;
use crate::types::{PoiseContext, Result};
use poise::ChoiceParameter;

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

    let filter_type = filter.unwrap_or(QuestFilterType::All);
    let filter = match filter_type {
        QuestFilterType::All => quest_management_facade::QuestListFilter::All,
        QuestFilterType::EnabledOnly => quest_management_facade::QuestListFilter::EnabledOnly,
        QuestFilterType::DisabledOnly => quest_management_facade::QuestListFilter::DisabledOnly,
    };
    let list_result =
        quest_management_facade::list_quests(&ctx.data().app_state, guild_id, filter).await?;

    let message = match list_result {
        quest_management_facade::QuestListResult::All(all_quests) => {
            // クエスト名と有効/無効のリストを作成
            let mut lines = vec!["# クエスト一覧".to_string(), "".to_string()];

            for quest in all_quests.iter().take(100) {
                let status = if quest.is_enabled {
                    "✅ 有効"
                } else {
                    "❌ 無効"
                };
                lines.push(format!("{} {}", status, quest.name));
            }

            if all_quests.len() > 100 {
                lines.push(format!("\n...他{}件", all_quests.len() - 100));
            }

            lines.join("\n")
        }
        quest_management_facade::QuestListResult::Enabled(results) => {
            let mut lines = vec!["# 有効なクエスト一覧".to_string(), "".to_string()];

            for result in results.iter().take(100) {
                lines.push(format!("✅ {result}"));
            }

            if results.len() > 100 {
                lines.push(format!("\n...他{}件", results.len() - 100));
            }

            if results.is_empty() {
                lines.push("有効なクエストはありません。".to_string());
            }

            lines.join("\n")
        }
        quest_management_facade::QuestListResult::Disabled(results) => {
            let mut lines = vec!["# 無効なクエスト一覧".to_string(), "".to_string()];

            for result in results.iter().take(100) {
                lines.push(format!("❌ {result}"));
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

    ctx.say(message).await?;

    Ok(())
}
