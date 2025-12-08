use poise::serenity_prelude::{AutocompleteChoice, Channel};
use sea_orm::TransactionTrait;
use tracing::{error, info};

use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::channel_type_repository::ChannelTypeRepository;
use crate::repository::database::guild_channel_repository::GuildChannelRepository;
use crate::repository::database::guild_repository::GuildRepository;
use crate::types::{PoiseContext, Result};
use crate::services::permission::check_bot_control_role;


/// チャンネル種別の選択肢を取得
async fn channel_type_autocomplete<'a>(
    ctx: PoiseContext<'_>,
    _partial: &'a str,
) -> impl Iterator<Item = AutocompleteChoice> + 'a {
    let db = ctx.data().app_state.guild_db();
    let channel_type_repo = ChannelTypeRepository::new();

    let channel_types = channel_type_repo
        .get_all(db)
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "チャンネル種別の取得に失敗しました");
            vec![]
        });

    channel_types
        .into_iter()
        .map(|ct| AutocompleteChoice::new(ct.name.clone(), ct.id.to_string()))
        .collect::<Vec<_>>()
        .into_iter()
}

/// チャンネルを登録
///
/// ギルドの通知チャンネルを登録します。
#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    rename = "channel_register",
    name_localized("ja", "チャンネル登録"),
    description_localized("ja", "ギルドの通知チャンネルを登録します。（gbf_bot_controlロール必須）"),
)]
pub async fn channel_register(
    ctx: PoiseContext<'_>,
    #[autocomplete = "channel_type_autocomplete"]
    #[name_localized("ja", "チャンネル種別")]
    #[description = "Channel type"]
    #[description_localized("ja", "チャンネル種別")]
    channel_type: String,

    #[name_localized("ja", "チャンネル")]
    #[description = "Channel"]
    #[description_localized("ja", "登録するチャンネル")]
    channel: Channel,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます。".to_string(),
        }
    })?;

    // channel_typeをi32に変換
    let channel_type_id: i32 = channel_type.parse().map_err(|_| {
        crate::types::AppError::Validation {
            field: "チャンネル種別".to_string(),
        }
    })?;

    // チャンネルIDを取得
    let channel_id = channel.id().get();

    info!(
        guild_id = %guild_id,
        channel_type = channel_type_id,
        channel_id = channel_id,
        "チャンネル登録を開始します"
    );

    let app_state = &ctx.data().app_state;
    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id.get() as i64).await?;

    let result = async {
        let guild_repo = GuildRepository::new();
        let channel_type_repo = ChannelTypeRepository::new();
        let guild_channel_repo = GuildChannelRepository::new();

        // ギルドが存在しない場合は自動登録
        let guild_name = ctx
            .guild()
            .map(|g| g.name.clone())
            .unwrap_or_else(|| "Unknown Guild".to_string());

        guild_repo
            .upsert_with_txn(&txn, guild_id.get() as i64, guild_name)
            .await?;

        // チャンネル種別が存在するか確認
        let channel_type_model = channel_type_repo
            .get_by_id(&txn, channel_type_id)
            .await?
            .ok_or_else(|| crate::types::AppError::NotFound(format!(
                "チャンネル種別ID {} が見つかりませんでした",
                channel_type_id
            )))?;

        // ギルドチャンネルを登録または更新
        guild_channel_repo
            .upsert_with_txn(&txn, guild_id.get() as i64, channel_type_id, channel_id as i64)
            .await?;

        info!(
            guild_id = %guild_id,
            channel_type = channel_type_id,
            channel_id = channel_id,
            "チャンネル登録が完了しました"
        );

        // コミット前に、全チャンネル種別の設定状況を取得（トランザクション内で実行）
        let all_channel_types = channel_type_repo.get_all(&txn).await?;
        let mut status_lines = Vec::new();

        for ct in all_channel_types {
            let guild_channel = guild_channel_repo
                .get_by_guild_and_type_with_txn(&txn, guild_id.get() as i64, ct.id)
                .await?;

            if let Some(gc) = guild_channel {
                status_lines.push(format!("• **{}**: <#{}>\n", ct.name, gc.channel_id));
            } else {
                status_lines.push(format!("• **{}**: 未設定\n", ct.name));
            }
        }

        // トランザクションをコミット（ここで確定させる）
        txn.commit().await?;

        // 登録後、設定状況を表示
        let mut status_message = format!(
            "✅ チャンネルを登録しました。\n\n**種別:** {}\n**チャンネル:** <#{}>\n\n**現在の設定状況:**\n",
            channel_type_model.name, channel_id
        );

        for line in status_lines {
            status_message.push_str(&line);
        }

        ctx.send(
            poise::CreateReply::default()
                .content(status_message)
                .ephemeral(true),
        )
        .await?;

        Ok::<(), crate::types::AppError>(())
    }
    .await;

    result
}
