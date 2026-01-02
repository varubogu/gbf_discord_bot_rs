use crate::facades::recruitment::recruitment_schedule_facade::RecruitmentScheduleFacade;
use crate::services::schedule::schedule_query_service::ScheduleListItem;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use tracing::info;

/// マルチ募集スケジュール一覧を表示
///
/// 登録されているマルチ募集スケジュールを表示します。
#[poise::command(
    slash_command,
    rename = "recruitment-schedule-list",
    guild_only,
    ephemeral = true,
    name_localized("ja", "定期募集一覧"),
    description_localized("ja", "登録されているマルチ募集スケジュールを表示します")
)]
pub async fn recruitment_schedule_list(
    ctx: PoiseContext<'_>,
    #[name_localized("ja", "全員表示")]
    #[description = "Show all schedules in this server (default: true)"]
    #[description_localized("ja", "このサーバーの全員のスケジュールを表示（デフォルト: true）")]
    show_all: Option<bool>,
) -> Result<()> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます".to_string(),
        })?;

    let user_id = ctx.author().id;
    let show_all = show_all.unwrap_or(true);

    info!(
        guild_id = guild_id.get(),
        user_id = user_id.get(),
        show_all = show_all,
        "定期募集一覧コマンドが実行されました"
    );

    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;
    // ✅ Facadeを利用して一覧を取得（トランザクションやタイムゾーン取得はFacade内部）
    let facade = RecruitmentScheduleFacade::new(std::sync::Arc::new(app_state.clone()));
    let schedules: Vec<ScheduleListItem> = facade
        .list_recruitment_schedules(guild_id.get() as i64, user_id.get() as i64, show_all)
        .await?;

    if schedules.is_empty() {
        let embed = CreateEmbed::default()
            .title("📅 定期募集スケジュール一覧")
            .description(if show_all {
                "登録されているスケジュールはありません。\n\n`/recruitment-schedule-create` コマンドでスケジュールを作成してください。"
            } else {
                "あなたが作成したスケジュールはありません。\n\n`/recruitment-schedule-create` コマンドでスケジュールを作成してください。"
            })
            .color(0xffaa00);

        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    }

    // 最大10件に制限
    let display_count = schedules.len().min(10);
    let total_count = schedules.len();

    let mut description = String::new();
    for (i, item) in schedules.iter().take(display_count).enumerate() {
        let status = if item.is_enabled {
            "✅ 有効"
        } else {
            "❌ 無効"
        };

        let dismissal_display = item
            .dismissal_times
            .as_ref()
            .map(|d| format!("\n                解散: {d}"))
            .unwrap_or_default();

        description.push_str(&format!(
            "{}. **{}** (ID: {}) {}\n\
                クエスト: {}\n\
                曜日: {} ({})\n\
                開始: {:02}:{:02}\n\
                募集: {}日前の{}{}\n\
                作成者: <@{}>\n\n",
            i + 1,
            item.name,
            item.id,
            status,
            item.quest_name,
            item.days_str,
            item.timezone,
            item.quest_start_hour,
            item.quest_start_minute,
            item.recruit_day_offset,
            item.recruit_time_str,
            dismissal_display,
            item.created_by
        ));
    }

    if total_count > display_count {
        description.push_str(&format!(
            "\n*...他 {} 件のスケジュールがあります*",
            total_count - display_count
        ));
    }

    let title = if show_all {
        "📅 定期募集スケジュール一覧（全員）"
    } else {
        "📅 定期募集スケジュール一覧（自分のみ）"
    };

    let embed = CreateEmbed::default()
        .title(title)
        .description(description)
        .color(0x00aaff)
        .footer(CreateEmbedFooter::new(format!(
            "全 {total_count} 件のスケジュール（{display_count}件表示）"
        )));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

// 表示用の曜日フォーマットはサービス層で実施（ScheduleQueryService::format_days）
