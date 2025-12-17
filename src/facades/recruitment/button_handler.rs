use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::recruitment_participants_repository::RecruitmentParticipantsRepositoryImpl;
use crate::services::recruitment::recruitment_participants_service::{
    ParticipationAction, RecruitmentParticipantsService,
};
use crate::services::recruitment::recruitment_query_service::RecruitmentQueryService;
use crate::types::{AppError, AppState, RecruitmentComponentId, Result};
use poise::serenity_prelude::{ComponentInteraction, Context};
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{error, info, instrument};

/// 募集ボタンのクリックを処理する（Facade層）
///
/// # 責務
/// - トランザクション境界の管理
/// - Service層の協調
/// - Discord APIとのやり取り
///
/// # 引数
/// * `ctx` - Discord Context
/// * `interaction` - ボタンクリックのインタラクション
/// * `app_state` - アプリケーション状態
#[instrument(level = "info", skip(ctx, interaction, app_state))]
pub async fn handle_recruitment_button(
    ctx: &Context,
    interaction: &ComponentInteraction,
    app_state: &AppState,
) -> Result<()> {
    info!("募集ボタンクリック処理開始");

    // Custom IDをパース
    let component_id = RecruitmentComponentId::parse(&interaction.data.custom_id)?;
    info!(component_id = ?component_id, "Custom IDをパースしました");

    // Guild IDを取得
    let guild_id = interaction
        .guild_id
        .ok_or_else(|| AppError::Business {
            message: "ギルドコンテキストが必要です".to_string(),
        })?
        .get();

    let user_id = interaction.user.id.get();
    let message_id = interaction.message.id.get();
    let channel_id = interaction.channel_id.get();

    // DB接続とトランザクション開始
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // 1. メッセージIDから募集情報を取得
        let query_service = RecruitmentQueryService::new();
        let recruitment = query_service
            .get_recruitment_by_message(&txn, guild_id, channel_id, message_id)
            .await?
            .ok_or_else(|| AppError::Business {
                message: "募集が見つかりませんでした".to_string(),
            })?;

        info!(recruitment_id = recruitment.id, "募集情報を取得しました");

        // 2. キャンセル済みチェック
        if recruitment.is_canceled {
            return Err(AppError::Business {
                message: "この募集はキャンセル済みです".to_string(),
            });
        }

        // 3. 期限切れチェック
        let now = chrono::Utc::now();
        if recruitment.quest_start_at < now {
            return Err(AppError::Business {
                message: "この募集は期限切れです".to_string(),
            });
        }

        // 4. Service層を使って参加/退出処理
        let participants_repo = RecruitmentParticipantsRepositoryImpl::new();
        let service =
            RecruitmentParticipantsService::<RecruitmentParticipantsRepositoryImpl>::new(
                Arc::new(participants_repo),
            );

        let response_message: String = match component_id {
            RecruitmentComponentId::Join => {
                // シンプル参加
                let action = service
                    .toggle_participation(&txn, recruitment.id, user_id, None)
                    .await?;
                match action {
                    ParticipationAction::Joined => "✅ 参加しました！".to_string(),
                    ParticipationAction::Left => "👋 参加を取り消しました".to_string(),
                }
            }
            RecruitmentComponentId::JoinElement(element_id) => {
                // 属性参加
                let element_name = get_element_name(element_id);
                let action = service
                    .toggle_participation(&txn, recruitment.id, user_id, Some(element_id))
                    .await?;
                match action {
                    ParticipationAction::Joined => {
                        format!("✅ {}属性で参加しました！", element_name)
                    }
                    ParticipationAction::Left => {
                        format!("👋 {}属性の参加を取り消しました", element_name)
                    }
                }
            }
            RecruitmentComponentId::JoinAllElements => {
                // 全属性可能参加（element_idはNULL）
                let action = service
                    .toggle_participation(&txn, recruitment.id, user_id, None)
                    .await?;
                match action {
                    ParticipationAction::Joined => "✅ 全属性可能として参加しました！".to_string(),
                    ParticipationAction::Left => "👋 全属性可能参加を取り消しました".to_string(),
                }
            }
            RecruitmentComponentId::LeaveAll => {
                // すべて取り消し
                let count = service.leave_all(&txn, recruitment.id, user_id).await?;
                if count > 0 {
                    "👋 すべての参加を取り消しました".to_string()
                } else {
                    "ℹ️ 参加していませんでした".to_string()
                }
            }
        };

        // 5. 参加者数を取得
        let participant_count = service
            .count_unique_participants(&txn, recruitment.id)
            .await?;

        info!(
            recruitment_id = recruitment.id,
            participant_count = participant_count,
            "参加者数を取得しました"
        );

        // 6. メッセージを更新して参加者一覧を反映
        update_recruitment_message(
            ctx,
            &txn,
            &recruitment,
            message_id,
            channel_id,
        )
        .await?;

        // 7. インタラクションに応答（deferの後なのでedit_response）
        interaction
            .edit_response(
                &ctx.http,
                poise::serenity_prelude::EditInteractionResponse::new()
                    .content(format!(
                        "{}\n\n現在の参加者数: **{}人**",
                        response_message, participant_count
                    )),
            )
            .await
            .map_err(|e| AppError::Discord(e))?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!("募集ボタンクリック処理が正常に完了しました");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "募集ボタンクリック処理でエラーが発生しました");

            // ユーザーにエラーメッセージを返す（deferの後なのでedit_response）
            if let Err(response_err) = interaction
                .edit_response(
                    &ctx.http,
                    poise::serenity_prelude::EditInteractionResponse::new()
                        .content(format!("❌ エラー: {}", e.user_message())),
                )
                .await
            {
                error!(error = %response_err, "エラーメッセージの送信に失敗しました");
            }

            Err(e)
        }
    }
}

/// 募集メッセージの参加者一覧を更新する
///
/// # 引数
/// * `ctx` - Discord Context
/// * `txn` - データベーストランザクション
/// * `recruitment` - 募集情報
/// * `message_id` - メッセージID
/// * `channel_id` - チャンネルID
#[instrument(level = "info", skip(ctx, txn))]
async fn update_recruitment_message(
    ctx: &Context,
    txn: &sea_orm::DatabaseTransaction,
    recruitment: &crate::models::battle_recruitments::BattleRecruitments,
    message_id: u64,
    channel_id: u64,
) -> Result<()> {
    use poise::serenity_prelude::{ChannelId, CreateEmbed, EditMessage, MessageId};

    info!("募集メッセージの参加者一覧を更新します");

    // 1. battle_styleの情報を取得（属性・絵文字の情報）
    let query_service = RecruitmentQueryService::new();
    let battle_style = query_service
        .get_battle_style_by_id(txn, recruitment.battle_style_id)
        .await?
        .ok_or_else(|| AppError::Business {
            message: "攻略方法が見つかりませんでした".to_string(),
        })?;

    // 2. DBから参加者一覧を取得
    use crate::models::entities::recruitment_participants::{
        Column as ParticipantColumn, Entity as RecruitmentParticipantEntity,
    };
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let participants = RecruitmentParticipantEntity::find()
        .filter(ParticipantColumn::RecruitmentId.eq(recruitment.id))
        .all(txn)
        .await
        .map_err(|e| AppError::Database(e))?;

    // 3. 参加者一覧のテキストを作成
    let participants_text =
        create_participants_text(&battle_style.display_name, &participants, ctx).await?;

    // 3.5. ユニーク参加者数を計算（複数属性でも1人とカウント）
    use std::collections::HashSet;
    let unique_user_ids: HashSet<i64> = participants.iter().map(|p| p.user_id).collect();
    let participant_count = unique_user_ids.len();

    // 4. メッセージを取得して更新
    let channel = ChannelId::new(channel_id);
    let mut message = channel
        .message(&ctx.http, MessageId::new(message_id))
        .await
        .map_err(|e| AppError::Discord(e))?;

    // 既存のembedを取得（最初のembedを使用）
    let existing_embed = message.embeds.first().cloned();

    // 新しいembedを作成（既存の内容を保持しつつdescriptionとfooterを更新）
    let new_embed = if let Some(old_embed) = existing_embed {
        let mut embed = CreateEmbed::new();
        if let Some(title) = &old_embed.title {
            embed = embed.title(title);
        }
        if let Some(color) = old_embed.colour {
            embed = embed.color(color);
        }
        embed = embed
            .description(&participants_text)
            .footer(poise::serenity_prelude::CreateEmbedFooter::new(format!(
                "参加者数: {}人",
                participant_count
            )));
        embed
    } else {
        // embedが存在しない場合は新規作成
        CreateEmbed::new()
            .title("参加者一覧")
            .description(&participants_text)
            .footer(poise::serenity_prelude::CreateEmbedFooter::new(format!(
                "参加者数: {}人",
                participant_count
            )))
            .color(0x0099ff)
    };

    // メッセージのembedを更新
    message
        .edit(&ctx.http, EditMessage::new().embed(new_embed))
        .await
        .map_err(|e| AppError::Discord(e))?;

    info!("募集メッセージの参加者一覧を更新しました");
    Ok(())
}

/// 参加者一覧のテキストを作成する
///
/// # 引数
/// * `battle_style_name` - 攻略方法の名前
/// * `participants` - 参加者一覧
/// * `ctx` - Discord Context（ユーザー情報取得用）
async fn create_participants_text(
    battle_style_name: &str,
    participants: &[crate::models::entities::recruitment_participants::Model],
    _ctx: &Context,
) -> Result<String> {
    use std::collections::HashMap;

    // 属性IDごとに参加者をグループ化（Noneは0として扱う）
    let mut participants_by_element: HashMap<i32, Vec<u64>> = HashMap::new();
    for participant in participants {
        let element_id = participant.element_id.unwrap_or(0);
        participants_by_element
            .entry(element_id)
            .or_insert_with(Vec::new)
            .push(participant.user_id as u64);
    }

    let mut text = String::new();

    // 6属性の場合
    if battle_style_name == "6属性" {
        use crate::types::{ALL_ELEMENTS_EMOJI, ELEMENT_EMOJIS, ELEMENT_NAMES};

        for (idx, (emoji, name)) in ELEMENT_EMOJIS.iter().zip(ELEMENT_NAMES.iter()).enumerate() {
            let element_id = (idx + 1) as i32;
            if let Some(user_ids) = participants_by_element.get(&element_id) {
                let user_mentions: Vec<String> = user_ids
                    .iter()
                    .map(|&uid| format!("<@{}>", uid))
                    .collect();
                text.push_str(&format!("{} {}: {}\n", emoji, name, user_mentions.join(" ")));
            } else {
                text.push_str(&format!("{} {}: なし\n", emoji, name));
            }
        }

        // 全属性可能（element_id = 0）
        if let Some(user_ids) = participants_by_element.get(&0) {
            let user_mentions: Vec<String> = user_ids
                .iter()
                .map(|&uid| format!("<@{}>", uid))
                .collect();
            text.push_str(&format!("{} 全属性可能: {}\n", ALL_ELEMENTS_EMOJI, user_mentions.join(" ")));
        } else {
            text.push_str(&format!("{} 全属性可能: なし\n", ALL_ELEMENTS_EMOJI));
        }
    } else {
        // シンプル参加の場合（element_id = null）
        use crate::types::SIMPLE_JOIN_EMOJI;

        if let Some(user_ids) = participants_by_element.get(&0) {
            let user_mentions: Vec<String> = user_ids
                .iter()
                .map(|&uid| format!("<@{}>", uid))
                .collect();
            text.push_str(&format!("{} 参加: {}\n", SIMPLE_JOIN_EMOJI, user_mentions.join(" ")));
        } else {
            text.push_str(&format!("{} 参加: なし\n", SIMPLE_JOIN_EMOJI));
        }
    }

    if text.is_empty() {
        Ok("現在参加者はいません。".to_string())
    } else {
        Ok(text)
    }
}

/// 属性IDから属性名を取得する
///
/// # 引数
/// * `element_id` - 属性ID（1: 火, 2: 水, 3: 土, 4: 風, 5: 光, 6: 闇）
///
/// # 戻り値
/// 属性名（不明な場合は"不明"）
fn get_element_name(element_id: i32) -> &'static str {
    match element_id {
        1 => "火",
        2 => "水",
        3 => "土",
        4 => "風",
        5 => "光",
        6 => "闇",
        _ => "不明",
    }
}
