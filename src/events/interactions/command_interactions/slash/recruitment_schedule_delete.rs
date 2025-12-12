use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::schedule::BattleRecruitmentScheduleRepository;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::CreateEmbed;
use sea_orm::TransactionTrait;
use tracing::{error, info};

use super::autocomplete::recruitment_schedule_auto_complete;

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
    description_localized("ja", "指定したIDのマルチ募集スケジュールを削除します"),
)]
pub async fn recruitment_schedule_delete(
    ctx: PoiseContext<'_>,
    #[autocomplete = "recruitment_schedule_auto_complete"]
    #[name_localized("ja", "スケジュール番号")]
    #[description = "Schedule ID"]
    #[description_localized("ja", "削除するスケジュールのID")]
    schedule_id: i64,
) -> Result<()> {
    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます".to_string(),
        }
    })?;

    let user_id = ctx.author().id;

    info!(
        guild_id = guild_id.get(),
        user_id = user_id.get(),
        schedule_id = schedule_id,
        "定期募集削除コマンドが実行されました"
    );

    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;
    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id.get() as i64).await?;

    let schedule_repo = BattleRecruitmentScheduleRepository::new();

    // スケジュールを取得して権限チェック
    let schedule_opt = schedule_repo
        .find_by_id(&txn, schedule_id as i32)
        .await?;

    let (schedule, _days) = schedule_opt.ok_or_else(|| {
        crate::types::AppError::Business {
            message: format!("スケジュールID {} が見つかりません", schedule_id),
        }
    })?;

    // 権限チェック: 自分が作成したスケジュールのみ削除可能（管理者は全スケジュール削除可能）
    let is_bot_controller = check_bot_control_role(ctx).await.is_ok();
    if schedule.created_by != user_id.get() as i64 && !is_bot_controller {
        txn.rollback().await?;
        return Err(crate::types::AppError::Business {
            message: "このスケジュールを削除する権限がありません".to_string(),
        });
    }

    // スケジュールを削除
    match schedule_repo.delete_with_txn(&txn, schedule_id as i32).await {
        Ok(_) => {
            txn.commit().await?;

            info!(
                schedule_id = schedule_id,
                guild_id = guild_id.get(),
                "定期募集スケジュールを削除しました"
            );

            let embed = CreateEmbed::default()
                .title("✅ 定期募集スケジュールを削除しました")
                .description(format!(
                    "**スケジュールID**: {}\n\n\
                     このスケジュールは削除され、今後自動投稿されなくなります。",
                    schedule_id
                ))
                .color(0xff0000);

            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "定期募集スケジュールの削除に失敗しました");
            return Err(e);
        }
    }

    Ok(())
}
