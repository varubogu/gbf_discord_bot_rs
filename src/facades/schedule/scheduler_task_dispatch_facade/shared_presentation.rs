use crate::presenter::RecruitmentPresenter;
use crate::services::guild_environment_service::ElementEmojis;
use crate::types::discord::{ActionRowContent, EmbedContent};

/// ボタン版募集の表示内容を組み立てる。
///
/// UIモデルへの変換はFacadeからPresenterへ委譲し、Service層を表示依存から分離する。
/// 定期募集・自動マッチング募集の両方から共有される。
pub(super) fn build_v2_recruitment_embed_and_components(
    battle_style_name: &str,
    element_emojis: &ElementEmojis,
) -> (EmbedContent, Vec<ActionRowContent>) {
    let initial_text =
        RecruitmentPresenter::create_initial_participants_text(battle_style_name, element_emojis);
    let components = if battle_style_name == "6属性" {
        RecruitmentPresenter::create_six_element_full_components(element_emojis)
    } else {
        RecruitmentPresenter::create_recruitment_buttons(battle_style_name, element_emojis)
    };
    let embed = RecruitmentPresenter::create_participants_embed(&initial_text, Some(0));

    (embed, components)
}
