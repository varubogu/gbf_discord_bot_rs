use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::schedule::BattleRecruitmentScheduleRepository;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::CreateEmbed;
use sea_orm::TransactionTrait;
use tracing::{error, info};

use super::autocomplete::recruitment_schedule_auto_complete;

/// マルチ募集スケジュールの有効/無効を切り替え
///
/// 指定したIDのマルチ募集スケジュールの有効/無効を切り替えます。
/// 自分が作成したスケジュールのみ切り替え可能です（管理者は全スケジュール切り替え可能）。
#[poise::command(
    slash_command,
    rename = "recruitment-schedule-toggle",
    guild_only,
    ephemeral = true,
    name_localized("ja", "定期募集切り替え"),
    description_localized("ja", "指定したIDのマルチ募集スケジュールの有効/無効を切り替えます"),
)]
pub async fn recruitment_schedule_toggle(
    ctx: PoiseContext<'_>,
    #[autocomplete = "recruitment_schedule_auto_complete"]
    #[name_localized("ja", "スケジュール番号")]
    #[description = "Schedule ID"]
    #[description_localized("ja", "切り替えるスケジュールのID")]
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
        "定期募集切り替えコマンドが実行されました"
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

    // 権限チェック: 自分が作成したスケジュールのみ切り替え可能（管理者は全スケジュール切り替え可能）
    let is_bot_controller = check_bot_control_role(ctx).await.is_ok();
    if schedule.created_by != user_id.get() as i64 && !is_bot_controller {
        txn.rollback().await?;
        return Err(crate::types::AppError::Business {
            message: "このスケジュールを操作する権限がありません".to_string(),
        });
    }

    // 有効/無効を切り替え
    let new_status = !schedule.is_enabled;

    match schedule_repo
        .toggle_enabled_with_txn(&txn, schedule_id as i32, new_status)
        .await
    {
        Ok(updated_schedule) => {
            txn.commit().await?;

            info!(
                schedule_id = schedule_id,
                guild_id = guild_id.get(),
                new_status = new_status,
                "定期募集スケジュールの有効/無効を切り替えました"
            );

            let status_text = if new_status { "有効" } else { "無効" };
            let status_icon = if new_status { "✅" } else { "❌" };

            let embed = CreateEmbed::default()
                .title(format!(
                    "{} 定期募集スケジュールを{}にしました",
                    status_icon, status_text
                ))
                .description(format!(
                    "**スケジュールID**: {}\n\
                     **ステータス**: {}\n\n\
                     {}",
                    schedule_id,
                    status_text,
                    if new_status {
                        "このスケジュールは再び自動投稿されます。"
                    } else {
                        "このスケジュールは一時停止され、自動投稿されなくなります。"
                    }
                ))
                .color(if new_status { 0x00ff00 } else { 0xff9900 });

            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "定期募集スケジュールの切り替えに失敗しました");
            return Err(e);
        }
    }

    Ok(())
}
