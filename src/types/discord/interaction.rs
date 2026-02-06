//! インタラクション関連のドメイン型
//!
//! Discordインタラクション（ボタン、セレクトメニュー等）への応答に必要なデータを定義する。

use super::message::{ActionRowContent, EmbedContent};

/// インタラクション応答データ
#[derive(Debug, Clone, Default)]
pub struct InteractionResponse {
    /// テキストコンテンツ
    pub content: Option<String>,
    /// Embed一覧
    pub embeds: Vec<EmbedContent>,
    /// アクションロー一覧
    pub components: Vec<ActionRowContent>,
    /// エフェメラル（本人のみ表示）フラグ
    pub ephemeral: bool,
}

impl InteractionResponse {
    /// 新しいInteractionResponseを作成する
    pub fn new() -> Self {
        Self::default()
    }

    /// テキストのみの応答を作成する
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            ..Default::default()
        }
    }

    /// エフェメラルなテキスト応答を作成する
    pub fn ephemeral_text(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            ephemeral: true,
            ..Default::default()
        }
    }

    /// テキストを設定する
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Embedを追加する
    pub fn with_embed(mut self, embed: EmbedContent) -> Self {
        self.embeds.push(embed);
        self
    }

    /// 複数のEmbedを追加する
    pub fn with_embeds(mut self, embeds: Vec<EmbedContent>) -> Self {
        self.embeds.extend(embeds);
        self
    }

    /// アクションローを追加する
    pub fn with_component(mut self, component: ActionRowContent) -> Self {
        self.components.push(component);
        self
    }

    /// 複数のアクションローを追加する
    pub fn with_components(mut self, components: Vec<ActionRowContent>) -> Self {
        self.components.extend(components);
        self
    }

    /// エフェメラルフラグを設定する
    pub fn with_ephemeral(mut self, ephemeral: bool) -> Self {
        self.ephemeral = ephemeral;
        self
    }
}

/// インタラクションデータ
///
/// Gateway経由で取得したインタラクション情報を保持する。
/// ComponentInteractionの代替として使用する。
#[derive(Debug, Clone)]
pub struct InteractionData {
    /// インタラクションID
    pub id: u64,
    /// インタラクショントークン
    pub token: String,
    /// カスタムID（ボタン、セレクトメニュー等のID）
    pub custom_id: String,
    /// 選択された値（セレクトメニューの場合）
    pub values: Vec<String>,
    /// インタラクション種別
    pub kind: InteractionKind,
}

/// インタラクション種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionKind {
    /// ボタンクリック
    Button,
    /// セレクトメニュー選択
    SelectMenu,
    /// モーダル送信
    ModalSubmit,
    /// 不明
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_response_builder() {
        let response = InteractionResponse::new()
            .with_content("Hello!")
            .with_ephemeral(true);

        assert_eq!(response.content, Some("Hello!".to_string()));
        assert!(response.ephemeral);
    }

    #[test]
    fn test_ephemeral_text_shorthand() {
        let response = InteractionResponse::ephemeral_text("Secret message");

        assert_eq!(response.content, Some("Secret message".to_string()));
        assert!(response.ephemeral);
    }
}
