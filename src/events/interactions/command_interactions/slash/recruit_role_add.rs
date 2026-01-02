use crate::facades::recruitment::role_management;
use crate::services::message::MessageTextId;
use crate::services::message::helpers::get_message_from_context;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::Role;
use std::collections::HashMap;

use super::autocomplete::quest_auto_complete;

#[poise::command(
    slash_command,
    name_localized("ja", "募集ロール追加"),
    check = "check_bot_control_role",
    ephemeral = true,
    description_localized(
        "ja",
        "マルチバトル募集の通知ロールを追加します（gbf_bot_controlロール必須）"
    )
)]
#[allow(clippy::too_many_arguments)]
pub async fn recruit_role_add(
    ctx: PoiseContext<'_>,

    #[autocomplete = "quest_auto_complete"]
    #[name_localized("ja", "クエスト名")]
    #[description = "quest name or alias (use 'すべて' for all recruitments)"]
    #[description_localized(
        "ja",
        "クエスト名またはクエスト別名（全ての募集の場合は「すべて」を入力）"
    )]
    quest: String,

    #[name_localized("ja", "ロール1")]
    #[description = "role 1"]
    #[description_localized("ja", "ロール1")]
    role1: Role,

    #[name_localized("ja", "ロール2")]
    #[description = "role 2"]
    #[description_localized("ja", "ロール2")]
    role2: Option<Role>,

    #[name_localized("ja", "ロール3")]
    #[description = "role 3"]
    #[description_localized("ja", "ロール3")]
    role3: Option<Role>,

    #[name_localized("ja", "ロール4")]
    #[description = "role 4"]
    #[description_localized("ja", "ロール4")]
    role4: Option<Role>,

    #[name_localized("ja", "ロール5")]
    #[description = "role 5"]
    #[description_localized("ja", "ロール5")]
    role5: Option<Role>,

    #[name_localized("ja", "ロール6")]
    #[description = "role 6"]
    #[description_localized("ja", "ロール6")]
    role6: Option<Role>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    // ロールIDのリストを作成
    let mut role_ids = vec![role1.id.get()];
    if let Some(r) = role2 {
        role_ids.push(r.id.get());
    }
    if let Some(r) = role3 {
        role_ids.push(r.id.get());
    }
    if let Some(r) = role4 {
        role_ids.push(r.id.get());
    }
    if let Some(r) = role5 {
        role_ids.push(r.id.get());
    }
    if let Some(r) = role6 {
        role_ids.push(r.id.get());
    }

    // Facadeを呼び出し
    let added_count =
        role_management::add_recruitment_notification_roles(&ctx, &quest, role_ids).await?;

    // 結果をユーザーに通知
    let mut params = HashMap::new();
    params.insert("count".to_string(), added_count.to_string());

    let message = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::RecruitmentRoleAddSuccess,
        params,
    )
    .await
    .unwrap_or_else(|_| format!("{added_count}個のロールを募集通知ロールとして登録しました。"));

    ctx.say(&message).await?;

    Ok(())
}
