use crate::events::permission::check_bot_control_role;
use crate::infrastructure::database::session::set_current_guild_id;
use crate::repository::GuildQuestDisableRepository;
use crate::repository::QuestRepository;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::AutocompleteChoice;
use sea_orm::TransactionTrait;
use std::collections::HashSet;
use tracing::{error, info};

/// クエスト名の入力候補を取得（無効化されているクエストのみ）
async fn quest_name_autocomplete(ctx: PoiseContext<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get() as i64,
        None => {
            error!("ギルドIDが取得できませんでした");
            return vec![];
        }
    };

    let app_state = &ctx.data().app_state;
    let db_conn = app_state.guild_db().clone();
    let quest_repository = app_state.repositories.quest;

    // トランザクションを開始してguild_idを設定
    let txn = match db_conn.begin().await {
        Ok(t) => t,
        Err(e) => {
            error!(error = %e, "トランザクション開始に失敗しました");
            return vec![];
        }
    };

    // RLSポリシー用にセッション変数を設定
    if let Err(e) = set_current_guild_id(&txn, guild_id).await {
        error!(error = %e, "guild_idの設定に失敗しました");
        return vec![];
    }

    // 無効化されているクエストを検索
    let results = quest_repository
        .search_disabled_quests(&txn, guild_id, partial)
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "クエスト検索に失敗しました");
            vec![]
        });

    let _ = txn.rollback().await;

    // AutocompleteChoiceに変換
    results
        .into_iter()
        .take(25)
        .map(|r| {
            let display_name = if r.name == r.matched_text {
                r.name.clone()
            } else {
                format!("{} ({})", r.name, r.matched_text)
            };
            AutocompleteChoice::new(display_name, r.name)
        })
        .collect()
}

/// クエストを有効化
///
/// 指定したクエストを有効化します（最大6つまで）。有効化されたクエストは新規募集時のオートコンプリートに表示されるようになります。
#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    rename = "quest_enable",
    name_localized("ja", "クエスト有効化"),
    description_localized(
        "ja",
        "クエストを有効化します（最大6つ）。（gbf_bot_controlロール必須）"
    )
)]
pub async fn quest_enable(
    ctx: PoiseContext<'_>,
    #[autocomplete = "quest_name_autocomplete"]
    #[name_localized("ja", "クエスト名1")]
    #[description = "Quest name to enable"]
    #[description_localized("ja", "有効化するクエスト名")]
    quest_name_1: String,

    #[autocomplete = "quest_name_autocomplete"]
    #[name_localized("ja", "クエスト名2")]
    #[description = "Quest name to enable"]
    #[description_localized("ja", "有効化するクエスト名")]
    quest_name_2: Option<String>,

    #[autocomplete = "quest_name_autocomplete"]
    #[name_localized("ja", "クエスト名3")]
    #[description = "Quest name to enable"]
    #[description_localized("ja", "有効化するクエスト名")]
    quest_name_3: Option<String>,

    #[autocomplete = "quest_name_autocomplete"]
    #[name_localized("ja", "クエスト名4")]
    #[description = "Quest name to enable"]
    #[description_localized("ja", "有効化するクエスト名")]
    quest_name_4: Option<String>,

    #[autocomplete = "quest_name_autocomplete"]
    #[name_localized("ja", "クエスト名5")]
    #[description = "Quest name to enable"]
    #[description_localized("ja", "有効化するクエスト名")]
    quest_name_5: Option<String>,

    #[autocomplete = "quest_name_autocomplete"]
    #[name_localized("ja", "クエスト名6")]
    #[description = "Quest name to enable"]
    #[description_localized("ja", "有効化するクエスト名")]
    quest_name_6: Option<String>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます。".to_string(),
        })?
        .get() as i64;

    // クエスト名を収集して重複を除去
    let mut quest_names = vec![quest_name_1];
    if let Some(name) = quest_name_2 {
        quest_names.push(name);
    }
    if let Some(name) = quest_name_3 {
        quest_names.push(name);
    }
    if let Some(name) = quest_name_4 {
        quest_names.push(name);
    }
    if let Some(name) = quest_name_5 {
        quest_names.push(name);
    }
    if let Some(name) = quest_name_6 {
        quest_names.push(name);
    }

    // 重複を除去
    let unique_quest_names: Vec<String> = quest_names
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let app_state = &ctx.data().app_state;
    let db_conn = app_state.guild_db();
    let quest_repository = app_state.repositories.quest;
    let disable_repository = app_state.repositories.guild_quest_disable;

    // トランザクション開始
    let txn = db_conn.begin().await?;

    // RLSポリシー用にセッション変数を設定
    set_current_guild_id(&txn, guild_id).await?;

    let mut success_count = 0;
    let mut already_enabled = Vec::new();
    let mut not_found = Vec::new();

    for quest_name in &unique_quest_names {
        // クエスト名からクエストIDを取得
        let search_results = quest_repository
            .search_by_name_or_alias(&txn, quest_name)
            .await?;

        let quest = match search_results.into_iter().find(|r| &r.name == quest_name) {
            Some(q) => q,
            None => {
                not_found.push(quest_name.clone());
                continue;
            }
        };

        // 無効化されているか確認
        if !disable_repository
            .is_disabled(&txn, guild_id, quest.quest_id)
            .await?
        {
            already_enabled.push(quest_name.clone());
            continue;
        }

        // クエストを有効化（無効化レコードを削除）
        disable_repository
            .enable_quest(&txn, guild_id, quest.quest_id)
            .await?;

        success_count += 1;

        info!(
            guild_id = guild_id,
            quest_id = quest.quest_id,
            quest_name = %quest_name,
            "クエストを有効化しました"
        );
    }

    txn.commit().await?;

    // 結果メッセージを作成
    let mut message_parts = Vec::new();

    if success_count > 0 {
        message_parts.push(format!(
            "✅ {success_count}件のクエストを有効化しました。\n新規募集時のオートコンプリートに表示されるようになります。"
        ));
    }

    if !already_enabled.is_empty() {
        message_parts.push(format!(
            "\n⚠️ 既に有効化されています: {}",
            already_enabled.join(", ")
        ));
    }

    if !not_found.is_empty() {
        message_parts.push(format!(
            "\n❌ 見つかりませんでした: {}",
            not_found.join(", ")
        ));
    }

    if message_parts.is_empty() {
        message_parts.push("処理するクエストがありませんでした。".to_string());
    }

    ctx.say(message_parts.join("\n")).await?;

    Ok(())
}
