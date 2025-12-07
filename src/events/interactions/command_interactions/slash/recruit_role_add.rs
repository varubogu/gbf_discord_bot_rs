use crate::facades::recruitment::role_management;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::Role;

use super::autocomplete::quest_auto_complete;

#[poise::command(
    slash_command,
    name_localized("ja", "募集ロール追加"),
    description_localized("ja", "マルチバトル募集の通知ロールを追加します（gbf_bot_controlロール必須）"),
    required_permissions = "ADMINISTRATOR"
)]
pub async fn recruit_role_add(
    ctx: PoiseContext<'_>,

    #[description = "quest name or alias (use 'すべて' for all recruitments)"]
    #[description_localized("ja", "クエスト名またはクエスト別名（全ての募集の場合は「すべて」を入力）")]
    #[autocomplete = "quest_auto_complete"]
    quest: String,

    #[description = "role 1"]
    #[description_localized("ja", "ロール1")]
    role1: Role,

    #[description = "role 2"]
    #[description_localized("ja", "ロール2")]
    role2: Option<Role>,

    #[description = "role 3"]
    #[description_localized("ja", "ロール3")]
    role3: Option<Role>,

    #[description = "role 4"]
    #[description_localized("ja", "ロール4")]
    role4: Option<Role>,

    #[description = "role 5"]
    #[description_localized("ja", "ロール5")]
    role5: Option<Role>,

    #[description = "role 6"]
    #[description_localized("ja", "ロール6")]
    role6: Option<Role>,
) -> Result<()> {
    ctx.defer().await?;

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
    let added_count = role_management::add_recruitment_notification_roles(
        &ctx,
        &quest,
        role_ids,
    ).await?;

    // 結果をユーザーに通知
    ctx.say(format!(
        "{}個のロールを募集通知ロールとして登録しました。",
        added_count
    ))
    .await?;

    Ok(())
}
