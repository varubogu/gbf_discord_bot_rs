use crate::services::message::MessageTextId;
use crate::types::AppState;
use poise::serenity_prelude::{
    ButtonStyle, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption,
};
use std::collections::HashMap;

pub const HELP_NAV_CUSTOM_ID_PREFIX: &str = "help_nav";
pub const HELP_NAV_JUMP_CUSTOM_ID: &str = "help_nav:jump";
pub const HELP_NAV_TO_INDEX_CUSTOM_ID: &str = "help_nav:to_index";

/// ヘルプのページ種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpPage {
    Index,
    MultiRecruitment,
    ScheduledRecruitment,
    AutoRecruitment,
    Utility,
    AdminManagement,
    AdminServer,
}

impl HelpPage {
    pub fn id(&self) -> &'static str {
        match self {
            Self::Index => "index",
            Self::MultiRecruitment => "multi_recruitment",
            Self::ScheduledRecruitment => "scheduled_recruitment",
            Self::AutoRecruitment => "auto_recruitment",
            Self::Utility => "utility",
            Self::AdminManagement => "admin_management",
            Self::AdminServer => "admin_server",
        }
    }

    pub fn from_id(value: &str) -> Option<Self> {
        match value {
            "index" => Some(Self::Index),
            "multi_recruitment" => Some(Self::MultiRecruitment),
            "scheduled_recruitment" => Some(Self::ScheduledRecruitment),
            "auto_recruitment" => Some(Self::AutoRecruitment),
            "utility" => Some(Self::Utility),
            "admin_management" => Some(Self::AdminManagement),
            "admin_server" => Some(Self::AdminServer),
            _ => None,
        }
    }

    fn title_message_id(&self) -> MessageTextId {
        match self {
            Self::Index => MessageTextId::HelpNavigatorIndexTitle,
            Self::MultiRecruitment => MessageTextId::HelpNavigatorPageMultiRecruitmentTitle,
            Self::ScheduledRecruitment => MessageTextId::HelpNavigatorPageScheduledRecruitmentTitle,
            Self::AutoRecruitment => MessageTextId::HelpNavigatorPageAutoRecruitmentTitle,
            Self::Utility => MessageTextId::HelpNavigatorPageUtilityTitle,
            Self::AdminManagement => MessageTextId::HelpNavigatorPageAdminManagementTitle,
            Self::AdminServer => MessageTextId::HelpNavigatorPageAdminServerTitle,
        }
    }

    fn description_message_id(&self) -> MessageTextId {
        match self {
            Self::Index => MessageTextId::HelpNavigatorIndexDescription,
            Self::MultiRecruitment => MessageTextId::HelpNavigatorPageMultiRecruitmentDescription,
            Self::ScheduledRecruitment => {
                MessageTextId::HelpNavigatorPageScheduledRecruitmentDescription
            }
            Self::AutoRecruitment => MessageTextId::HelpNavigatorPageAutoRecruitmentDescription,
            Self::Utility => MessageTextId::HelpNavigatorPageUtilityDescription,
            Self::AdminManagement => MessageTextId::HelpNavigatorPageAdminManagementDescription,
            Self::AdminServer => MessageTextId::HelpNavigatorPageAdminServerDescription,
        }
    }
}

/// ヘルプページ遷移方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpDirection {
    Prev,
    Next,
}

impl HelpDirection {
    fn id(&self) -> &'static str {
        match self {
            Self::Prev => "prev",
            Self::Next => "next",
        }
    }

    fn from_id(value: &str) -> Option<Self> {
        match value {
            "prev" => Some(Self::Prev),
            "next" => Some(Self::Next),
            _ => None,
        }
    }
}

/// ヘルプ遷移カスタムIDの分解結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HelpNavInput {
    pub current_page: HelpPage,
    pub direction: HelpDirection,
}

/// カスタムIDを生成する
pub fn build_help_nav_custom_id(current_page: HelpPage, direction: HelpDirection) -> String {
    format!(
        "{HELP_NAV_CUSTOM_ID_PREFIX}:{}:{}",
        current_page.id(),
        direction.id()
    )
}

/// カスタムIDを解析する
pub fn parse_help_nav_custom_id(custom_id: &str) -> Option<HelpNavInput> {
    let mut parts = custom_id.split(':');
    let prefix = parts.next()?;
    if prefix != HELP_NAV_CUSTOM_ID_PREFIX {
        return None;
    }

    let current_page = HelpPage::from_id(parts.next()?)?;
    let direction = HelpDirection::from_id(parts.next()?)?;

    if parts.next().is_some() {
        return None;
    }

    Some(HelpNavInput {
        current_page,
        direction,
    })
}

/// 実行者の権限に応じて表示可能なページを構成する
pub fn build_visible_pages(has_bot_control: bool, is_admin_server: bool) -> Vec<HelpPage> {
    let mut pages = vec![
        HelpPage::Index,
        HelpPage::MultiRecruitment,
        HelpPage::ScheduledRecruitment,
        HelpPage::AutoRecruitment,
        HelpPage::Utility,
    ];

    if has_bot_control {
        pages.push(HelpPage::AdminManagement);
        if is_admin_server {
            pages.push(HelpPage::AdminServer);
        }
    }

    pages
}

/// 現在ページと方向から次ページを解決する（ループ遷移）
pub fn resolve_next_page(
    visible_pages: &[HelpPage],
    current_page: HelpPage,
    direction: HelpDirection,
) -> HelpPage {
    if visible_pages.is_empty() {
        return HelpPage::Index;
    }

    let current_index = visible_pages
        .iter()
        .position(|p| *p == current_page)
        .unwrap_or(0);

    let next_index = match direction {
        HelpDirection::Prev => {
            if current_index == 0 {
                visible_pages.len() - 1
            } else {
                current_index - 1
            }
        }
        HelpDirection::Next => {
            if current_index + 1 >= visible_pages.len() {
                0
            } else {
                current_index + 1
            }
        }
    };

    visible_pages[next_index]
}

/// ヘルプ描画結果
pub struct HelpView {
    pub embed: CreateEmbed,
    pub components: Vec<CreateActionRow>,
}

/// ページ内容と操作ボタンを構築する
pub async fn build_help_view(
    app_state: &AppState,
    guild_id: Option<i64>,
    locale: &str,
    requested_page: HelpPage,
    visible_pages: &[HelpPage],
) -> HelpView {
    let pages = if visible_pages.is_empty() {
        vec![HelpPage::Index]
    } else {
        visible_pages.to_vec()
    };

    let current_page = if pages.contains(&requested_page) {
        requested_page
    } else {
        HelpPage::Index
    };

    let current_page_number = pages
        .iter()
        .position(|p| *p == current_page)
        .map(|index| index + 1)
        .unwrap_or(1);
    let total_pages = pages.len();

    let title = get_message_or_key(
        app_state,
        guild_id,
        locale,
        current_page.title_message_id(),
        HashMap::new(),
    )
    .await;

    let description = build_description(app_state, guild_id, locale, current_page, &pages).await;

    let mut footer_params = HashMap::new();
    footer_params.insert("current".to_string(), current_page_number.to_string());
    footer_params.insert("total".to_string(), total_pages.to_string());
    let footer = get_message_or_key(
        app_state,
        guild_id,
        locale,
        MessageTextId::HelpNavigatorFooter,
        footer_params,
    )
    .await;

    let prev_label = get_message_or_key(
        app_state,
        guild_id,
        locale,
        MessageTextId::HelpNavigatorButtonPrev,
        HashMap::new(),
    )
    .await;
    let next_label = get_message_or_key(
        app_state,
        guild_id,
        locale,
        MessageTextId::HelpNavigatorButtonNext,
        HashMap::new(),
    )
    .await;

    let prev_button =
        CreateButton::new(build_help_nav_custom_id(current_page, HelpDirection::Prev))
            .style(ButtonStyle::Secondary)
            .label(prev_label);
    let next_button =
        CreateButton::new(build_help_nav_custom_id(current_page, HelpDirection::Next))
            .style(ButtonStyle::Primary)
            .label(next_label);

    let embed = CreateEmbed::new()
        .title(title)
        .description(description)
        .footer(CreateEmbedFooter::new(footer));

    let mut components = Vec::new();
    if current_page == HelpPage::Index {
        let mut jump_options = Vec::new();
        for page in pages.iter().copied().filter(|p| *p != HelpPage::Index) {
            let page_title = get_message_or_key(
                app_state,
                guild_id,
                locale,
                page.title_message_id(),
                HashMap::new(),
            )
            .await;
            jump_options.push(CreateSelectMenuOption::new(page_title, page.id()));
        }

        if !jump_options.is_empty() {
            let jump_placeholder = get_message_or_key(
                app_state,
                guild_id,
                locale,
                MessageTextId::HelpNavigatorJumpPlaceholder,
                HashMap::new(),
            )
            .await;
            let jump_select = CreateSelectMenu::new(
                HELP_NAV_JUMP_CUSTOM_ID,
                CreateSelectMenuKind::String {
                    options: jump_options,
                },
            )
            .placeholder(jump_placeholder);
            components.push(CreateActionRow::SelectMenu(jump_select));
        }

        components.push(CreateActionRow::Buttons(vec![prev_button, next_button]));
    } else {
        let to_index_label = get_message_or_key(
            app_state,
            guild_id,
            locale,
            MessageTextId::HelpNavigatorButtonToIndex,
            HashMap::new(),
        )
        .await;
        let to_index_button = CreateButton::new(HELP_NAV_TO_INDEX_CUSTOM_ID)
            .style(ButtonStyle::Secondary)
            .label(to_index_label);
        components.push(CreateActionRow::Buttons(vec![
            prev_button,
            to_index_button,
            next_button,
        ]));
    }

    HelpView { embed, components }
}

async fn build_description(
    app_state: &AppState,
    guild_id: Option<i64>,
    locale: &str,
    current_page: HelpPage,
    visible_pages: &[HelpPage],
) -> String {
    if current_page != HelpPage::Index {
        return get_message_or_key(
            app_state,
            guild_id,
            locale,
            current_page.description_message_id(),
            HashMap::new(),
        )
        .await;
    }

    let index_intro = get_message_or_key(
        app_state,
        guild_id,
        locale,
        MessageTextId::HelpNavigatorIndexDescription,
        HashMap::new(),
    )
    .await;

    let mut page_lines: Vec<String> = Vec::new();
    for page in visible_pages {
        if *page == HelpPage::Index {
            continue;
        }
        let page_title = get_message_or_key(
            app_state,
            guild_id,
            locale,
            page.title_message_id(),
            HashMap::new(),
        )
        .await;
        page_lines.push(format!("- {page_title}"));
    }

    if page_lines.is_empty() {
        return index_intro;
    }

    format!("{index_intro}\n\n{}", page_lines.join("\n"))
}

async fn get_message_or_key(
    app_state: &AppState,
    guild_id: Option<i64>,
    locale: &str,
    message_id: MessageTextId,
    params: HashMap<String, String>,
) -> String {
    app_state
        .message_service()
        .get_message(
            app_state.guild_db(),
            message_id.as_str(),
            params,
            guild_id,
            Some(locale),
        )
        .await
        .unwrap_or_else(|_| message_id.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_help_nav_custom_id_正常系() {
        let parsed = parse_help_nav_custom_id("help_nav:index:next").unwrap();
        assert_eq!(parsed.current_page, HelpPage::Index);
        assert_eq!(parsed.direction, HelpDirection::Next);
    }

    #[test]
    fn parse_help_nav_custom_id_不正系() {
        assert!(parse_help_nav_custom_id("help_nav:index").is_none());
        assert!(parse_help_nav_custom_id("help_nav:unknown:next").is_none());
        assert!(parse_help_nav_custom_id("help_nav:index:unknown").is_none());
        assert!(parse_help_nav_custom_id("other:index:next").is_none());
    }

    #[test]
    fn resolve_next_page_ループ遷移_先頭で戻る() {
        let pages = vec![
            HelpPage::Index,
            HelpPage::MultiRecruitment,
            HelpPage::ScheduledRecruitment,
        ];

        let next = resolve_next_page(&pages, HelpPage::Index, HelpDirection::Prev);
        assert_eq!(next, HelpPage::ScheduledRecruitment);
    }

    #[test]
    fn resolve_next_page_ループ遷移_末尾で進む() {
        let pages = vec![
            HelpPage::Index,
            HelpPage::MultiRecruitment,
            HelpPage::ScheduledRecruitment,
        ];

        let next = resolve_next_page(&pages, HelpPage::ScheduledRecruitment, HelpDirection::Next);
        assert_eq!(next, HelpPage::Index);
    }

    #[test]
    fn build_visible_pages_権限別に変化する() {
        let general = build_visible_pages(false, false);
        assert_eq!(
            general,
            vec![
                HelpPage::Index,
                HelpPage::MultiRecruitment,
                HelpPage::ScheduledRecruitment,
                HelpPage::AutoRecruitment,
                HelpPage::Utility
            ]
        );

        let bot_control = build_visible_pages(true, false);
        assert!(bot_control.contains(&HelpPage::AdminManagement));
        assert!(!bot_control.contains(&HelpPage::AdminServer));

        let admin_server = build_visible_pages(false, true);
        assert!(!admin_server.contains(&HelpPage::AdminManagement));
        assert!(!admin_server.contains(&HelpPage::AdminServer));

        let bot_admin = build_visible_pages(true, true);
        assert!(bot_admin.contains(&HelpPage::AdminManagement));
        assert!(bot_admin.contains(&HelpPage::AdminServer));
    }
}
