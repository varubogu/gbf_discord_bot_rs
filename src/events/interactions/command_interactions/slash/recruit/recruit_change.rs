use crate::events::helpers::get_message_from_context;
use crate::events::permission::resolve_bot_control;
use crate::facades::guild_settings::GuildSettingsFacade;
use crate::facades::recruitment::change::{
    RecruitmentChangeContent, change_recruitment_information,
};
use crate::gateway::PoiseDiscordGateway;
use crate::services::message::MessageTextId;
use crate::services::unified_datetime_parser::{
    DateTimeParseOptions, ParsedDateTime, parse_datetime,
};
use crate::types::discord::{DiscordGuildId, MessageData};
use crate::types::{AppError, PoiseContext, Result};
use poise::serenity_prelude::Message;
use std::collections::HashMap;
use std::sync::Arc;

use super::super::autocomplete::{battle_style_auto_complete, quest_auto_complete};

#[poise::command(
    slash_command,
    name_localized("ja", "マルチバトル募集内容変更"),
    description_localized("ja", "マルチバトル募集内容を変更します。"),
    ephemeral = true
)]
pub async fn recruit_change(
    ctx: PoiseContext<'_>,

    #[name_localized("ja", "募集メッセージ")]
    #[description = "recruit message"]
    #[description_localized("ja", "募集中のメッセージIDまたはメッセージURL")]
    message: Message,

    #[name_localized("ja", "クエスト名")]
    #[description = "quest name or alias"]
    #[description_localized("ja", "クエスト名またはクエスト別名（変更する場合のみ指定）")]
    #[autocomplete = "quest_auto_complete"]
    quest: Option<String>,

    #[name_localized("ja", "クエスト出発日時")]
    #[description = "Quest departure date and time"]
    #[description_localized("ja", "クエスト出発日時（変更する場合のみ指定）")]
    event_date: Option<String>,

    #[name_localized("ja", "マルチ攻略方法")]
    #[description = "battle style"]
    #[description_localized("ja", "マルチ攻略方法（変更する場合のみ指定）")]
    #[autocomplete = "battle_style_auto_complete"]
    battle_style: Option<i32>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    // パラメータが何も指定されていない場合はエラー
    if quest.is_none() && event_date.is_none() && battle_style.is_none() {
        let message = get_message_from_context(
            &ctx,
            ctx.data().app_state.message_service(),
            MessageTextId::RecruitmentCommandChangeNoChanges,
            HashMap::new(),
        )
        .await
        .unwrap_or_else(|_| "変更する項目を少なくとも1つ指定してください。".to_string());

        return Err(crate::types::AppError::Business { message });
    }

    // タイムゾーンを取得（日時が指定されている場合のみ）
    let parsed_date = if let Some(date_str) = event_date {
        // ギルドIDを取得
        let guild_id = ctx.guild_id().ok_or_else(|| {
            crate::types::AppError::Generic(
                "このコマンドはサーバー内でのみ使用できます".to_string(),
            )
        })?;

        // タイムゾーンを取得（Facade経由）
        let app_state = &ctx.data().app_state;
        let timezone_facade = GuildSettingsFacade::new(Arc::new(app_state.clone()));
        let timezone = timezone_facade.get_timezone(guild_id.get() as i64).await?;

        // 日時文字列をDateTime<Utc>に変換（サーバー設定のタイムゾーンとして解釈）
        let options = DateTimeParseOptions::for_quest_departure(timezone);
        let results = parse_datetime(&date_str, &options)?;
        let parsed_date = match &results[0] {
            ParsedDateTime::Absolute(dt) => *dt,
            _ => {
                return Err(crate::types::AppError::Business {
                    message: "クエスト出発日時は絶対日時で指定してください".to_string(),
                });
            }
        };
        Some(parsed_date)
    } else {
        None
    };

    // 募集内容変更を実行
    // events層でpoise型からドメイン型への変換を行う
    let app_state = &ctx.data().app_state;
    let gateway = PoiseDiscordGateway::new(Arc::clone(&ctx.serenity_context().http));
    let message_data = MessageData::from(message.clone());
    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);

    // 実行者情報を解決（events層でDiscordコンテキストから取得し、ドメイン値として渡す）
    let invoker_user_id = ctx.author().id.get();
    let has_bot_control = resolve_bot_control(&ctx).await;

    match change_recruitment_information(
        app_state,
        &gateway,
        DiscordGuildId::new(guild_id),
        &message_data,
        RecruitmentChangeContent {
            quest,
            event_date: parsed_date,
            battle_style_id: battle_style,
        },
        invoker_user_id,
        has_bot_control,
    )
    .await
    {
        Ok(_) => {
            // 処理完了をユーザーに通知
            let success_message = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentCommandChangeSuccess,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| "募集内容を更新しました。".to_string());

            ctx.send(
                poise::CreateReply::default()
                    .content(success_message)
                    .ephemeral(true),
            )
            .await?;
            Ok(())
        }
        Err(AppError::Business { .. }) => {
            // 権限エラー等のビジネスエラーはロケール対応メッセージを表示
            let error_msg = get_message_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentCommandChangePermissionDenied,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|_| {
                "この募集の変更は作成者本人または gbf_bot_control ロールを持つ管理者のみ可能です。"
                    .to_string()
            });

            ctx.send(
                poise::CreateReply::default()
                    .content(error_msg)
                    .ephemeral(true),
            )
            .await?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}
