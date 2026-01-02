use crate::facades::recruitment::recruitment_schedule_facade::RecruitmentScheduleFacade;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::{CreateEmbed, Permissions};
use tracing::info;

use super::super::autocomplete::recruitment_schedule_auto_complete;

/// マルチ募集スケジュールを削除
///
/// 指定したIDのマルチ募集スケジュールを削除します。
/// 自分が作成したスケジュールのみ削除可能です（管理者は全スケジュール削除可能）。
#[poise::command(
    slash_command,
    rename = "recruitment-schedule-delete",
    guild_only,
    ephemeral = true,
    name_localized("ja", "定期募集削除"),
    description_localized("ja", "指定したIDのマルチ募集スケジュールを削除します")
)]
pub async fn recruitment_schedule_delete(
    ctx: PoiseContext<'_>,
    #[autocomplete = "recruitment_schedule_auto_complete"]
    #[name_localized("ja", "スケジュール番号")]
    #[description = "Schedule ID"]
    #[description_localized("ja", "削除するスケジュールのID")]
    schedule_id: i64,
) -> Result<()> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます".to_string(),
        })?;

    let user_id = ctx.author().id;

    info!(
        guild_id = guild_id.get(),
        user_id = user_id.get(),
        schedule_id = schedule_id,
        "定期募集削除コマンドが実行されました"
    );

    ctx.defer_ephemeral().await?;

    // 管理者権限をチェック
    let is_admin = if let Some(member) = ctx.author_member().await {
        member
            .permissions
            .map(|p| p.contains(Permissions::ADMINISTRATOR))
            .unwrap_or(false)
    } else {
        false
    };

    let app_state = &ctx.data().app_state;
    let facade = RecruitmentScheduleFacade::new(std::sync::Arc::new(app_state.clone()));
    facade
        .delete_recruitment_schedule(
            guild_id.get() as i64,
            schedule_id as i32,
            user_id.get() as i64,
            is_admin,
        )
        .await?;

    info!(
        schedule_id = schedule_id,
        guild_id = guild_id.get(),
        "定期募集スケジュールを削除しました"
    );

    let embed = CreateEmbed::default()
        .title("✅ 定期募集スケジュールを削除しました")
        .description(format!(
            "**スケジュールID**: {schedule_id}\n\n\
             このスケジュールは削除され、今後自動投稿されなくなります。"
        ))
        .color(0xff0000);

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
