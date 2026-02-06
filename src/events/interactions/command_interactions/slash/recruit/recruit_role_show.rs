use crate::events::permission::check_bot_control_role;
use crate::facades::recruitment::role_management;
use crate::types::{PoiseContext, Result};

#[poise::command(
    slash_command,
    name_localized("ja", "募集ロール確認"),
    check = "check_bot_control_role",
    ephemeral = true,
    description_localized(
        "ja",
        "マルチバトル募集の通知ロール設定を確認します（gbf_bot_controlロール必須）"
    )
)]
pub async fn recruit_role_show(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    // ギルドIDを取得
    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Generic("このコマンドはサーバー内でのみ使用できます".to_string())
    })?;

    let app_state = &ctx.data().app_state;

    // Facadeを呼び出し
    let settings = role_management::show_recruitment_notification_roles(app_state, guild_id.get()).await?;

    // 設定が存在しない場合
    if settings.all_recruitment_roles.is_empty() && settings.quest_recruitment_roles.is_empty() {
        ctx.say("⚠️ 募集通知ロールが登録されていません。").await?;
        return Ok(());
    }

    // メッセージを構築
    let mut message = "**現在の募集通知ロール設定:**\n\n".to_string();

    // 全募集通知ロール
    if !settings.all_recruitment_roles.is_empty() {
        message.push_str("**【すべての募集】**\n");
        for role_id in &settings.all_recruitment_roles {
            message.push_str(&format!("• <@&{role_id}>\n"));
        }
        message.push('\n');
    }

    // クエスト別募集通知ロール
    if !settings.quest_recruitment_roles.is_empty() {
        message.push_str("**【クエスト別募集】**\n");

        // クエストIDでソート
        let mut quest_ids: Vec<i32> = settings.quest_recruitment_roles.keys().copied().collect();
        quest_ids.sort_unstable();

        for quest_id in quest_ids {
            if let Some(role_ids) = settings.quest_recruitment_roles.get(&quest_id) {
                let quest_name = settings
                    .quest_names
                    .get(&quest_id)
                    .map(|s| s.as_str())
                    .unwrap_or("不明なクエスト");

                message.push_str(&format!("**{quest_name}**\n"));
                for role_id in role_ids {
                    message.push_str(&format!("• <@&{role_id}>\n"));
                }
                message.push('\n');
            }
        }
    }

    ctx.say(&message).await?;

    Ok(())
}
