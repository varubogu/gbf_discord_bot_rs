use crate::errors::ServiceError;
use crate::repository::{GuildMessageTextRepository, MessageTextRepository};
use regex::Regex;
use sea_orm::ConnectionTrait;
use std::collections::HashMap;
use std::sync::OnceLock;
use tracing::{debug, warn};

use super::yaml_loader;

/// メッセージ取得サービス
///
/// 優先順位:
/// 1. Guild固有メッセージ (guild_master.guild_message_texts)
/// 2. グローバルマスターメッセージ (master.message_texts)
/// 3. YAMLファイルから読み込んだデフォルトメッセージ
/// 4. システムエラー
#[derive(Debug, Clone)]
pub struct MessageService<G, M>
where
    G: GuildMessageTextRepository,
    M: MessageTextRepository,
{
    guild_message_repo: G,
    message_repo: M,
}

impl<G, M> MessageService<G, M>
where
    G: GuildMessageTextRepository,
    M: MessageTextRepository,
{
    /// 新しいメッセージサービスインスタンスを作成
    pub fn new(guild_message_repo: G, message_repo: M) -> Self {
        Self {
            guild_message_repo,
            message_repo,
        }
    }

    /// メッセージを取得
    ///
    /// # 引数
    /// * `db` - データベース接続（DatabaseConnection または DatabaseTransaction）
    /// * `message_id` - メッセージID（DB検索キー兼YAMLキー）
    /// * `params` - テキスト埋め込み用パラメータ
    /// * `guild_id` - ギルドID (オプション)
    /// * `locale` - ユーザーロケール (オプション)
    ///
    /// # 戻り値
    /// パラメータ置換済みのメッセージ文字列
    ///
    /// # エラー
    /// メッセージが見つからない場合（通常はYAMLに存在するため発生しないはず）
    pub async fn get_message<C>(
        &self,
        db: &C,
        message_id: &str,
        params: HashMap<String, String>,
        guild_id: Option<i64>,
        locale: Option<&str>,
    ) -> Result<String, ServiceError>
    where
        C: ConnectionTrait,
    {
        // ロケールを決定 (ja or en)
        let locale = self.determine_locale(locale);

        debug!(
            message_id = %message_id,
            guild_id = ?guild_id,
            locale = %locale,
            "メッセージ取得を開始"
        );

        // 1. Guild固有メッセージを試行
        if let Some(gid) = guild_id
            && let Some(message) = self.get_guild_message(db, gid, message_id, &locale).await?
        {
            debug!(
                message_id = %message_id,
                guild_id = %gid,
                locale = %locale,
                "Guild固有メッセージを使用"
            );
            return Ok(self.replace_parameters(&message, &params));
        }

        // 2. グローバルマスターメッセージを試行
        if let Some(message) = self.get_master_message(db, message_id, &locale).await? {
            debug!(
                message_id = %message_id,
                locale = %locale,
                "グローバルマスターメッセージを使用"
            );
            return Ok(self.replace_parameters(&message, &params));
        }

        // 3. YAMLメッセージを試行（yaml_loaderを使用して動的にメッセージを取得）
        if let Some(yaml_message) = yaml_loader::get_yaml_message(message_id, &locale) {
            debug!(
                message_id = %message_id,
                locale = %locale,
                "YAMLメッセージを使用"
            );
            return Ok(self.replace_parameters(&yaml_message, &params));
        }

        // 4. すべて失敗した場合はエラー（通常はYAMLに存在するため発生しないはず）
        warn!(
            message_id = %message_id,
            "メッセージが見つかりませんでした（DB・YAML共に存在しません）"
        );
        Err(ServiceError::NotFound(format!(
            "メッセージが見つかりません: message_id={message_id}"
        )))
    }

    /// Guild固有メッセージを取得
    async fn get_guild_message<C>(
        &self,
        db: &C,
        guild_id: i64,
        message_id: &str,
        locale: &str,
    ) -> Result<Option<String>, ServiceError>
    where
        C: ConnectionTrait,
    {
        match self
            .guild_message_repo
            .get_by_guild_and_id(db, guild_id, message_id)
            .await
        {
            Ok(Some(model)) => {
                let message = if locale == "ja" {
                    model.message_jp
                } else if let Some(en_msg) = model.message_en {
                    en_msg
                } else {
                    model.message_jp
                };
                Ok(Some(message))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                warn!(
                    error = %e,
                    guild_id = %guild_id,
                    message_id = %message_id,
                    "Guild固有メッセージ取得中にエラーが発生"
                );
                Ok(None) // エラーが発生してもフォールバックを続行
            }
        }
    }

    /// グローバルマスターメッセージを取得
    async fn get_master_message<C>(
        &self,
        db: &C,
        message_id: &str,
        locale: &str,
    ) -> Result<Option<String>, ServiceError>
    where
        C: ConnectionTrait,
    {
        match self.message_repo.get_by_id(db, message_id).await {
            Ok(Some(model)) => {
                let message = if locale == "ja" {
                    model.message_jp
                } else if let Some(en_msg) = model.message_en {
                    en_msg
                } else {
                    model.message_jp
                };
                Ok(Some(message))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                warn!(
                    error = %e,
                    message_id = %message_id,
                    "グローバルマスターメッセージ取得中にエラーが発生"
                );
                Ok(None) // エラーが発生してもフォールバックを続行
            }
        }
    }

    /// ロケールを決定
    ///
    /// ユーザーロケールから ja または en を決定
    /// jaでない場合は全てenにフォールバック
    fn determine_locale(&self, locale: Option<&str>) -> String {
        match locale {
            Some(l) if l.starts_with("ja") => "ja".to_string(),
            _ => "en".to_string(),
        }
    }

    /// パラメータを置換
    ///
    /// `{{variable}}` 形式の文字列を置換
    /// エスケープシーケンス対応:
    /// - `\{{variable}}` -> `{{variable}}` (置換されない)
    /// - `\\{{variable}}` -> `\xyz` (置換される)
    fn replace_parameters(&self, template: &str, params: &HashMap<String, String>) -> String {
        static PARAM_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = PARAM_REGEX.get_or_init(|| {
            // `(?<!\\)((?:\\\\)*)\\?\{\{(\w+)\}\}` のようなパターンで
            // エスケープを考慮した置換を行う
            // 簡易実装として、まずエスケープ処理を先に行う
            Regex::new(r"\{\{(\w+)\}\}").unwrap()
        });

        // エスケープシーケンス処理
        // 1. まず \\ を一時プレースホルダーに置換
        let temp_backslash = "\x00BACKSLASH\x00";
        let temp_open_brace = "\x00OPEN_BRACE\x00";
        let temp_close_brace = "\x00CLOSE_BRACE\x00";

        let mut result = template.to_string();

        // \\ -> 一時プレースホルダー
        result = result.replace("\\\\", temp_backslash);
        // \{ -> 一時プレースホルダー
        result = result.replace("\\{", temp_open_brace);
        // \} -> 一時プレースホルダー
        result = result.replace("\\}", temp_close_brace);

        // パラメータ置換
        result = regex
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                params
                    .get(var_name)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| caps[0].to_string())
            })
            .to_string();

        // 一時プレースホルダーを元に戻す
        result = result.replace(temp_backslash, "\\");
        result = result.replace(temp_open_brace, "{");
        result = result.replace(temp_close_brace, "}");

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::entities::guild_master::guild_message_texts;
    use crate::models::message_texts::MessageTexts;
    use crate::repository::{GuildMessageTextRepository, MessageTextRepository};
    use async_trait::async_trait;
    use sea_orm::DbErr;

    #[derive(Debug, Clone, Copy)]
    struct DummyGuildMessageTextRepository;

    #[async_trait]
    impl GuildMessageTextRepository for DummyGuildMessageTextRepository {
        async fn get_by_guild_and_id<'c, C>(
            &self,
            _db: &'c C,
            _guild_id: i64,
            _message_id: &str,
        ) -> Result<Option<guild_message_texts::Model>, DbErr>
        where
            C: sea_orm::ConnectionTrait,
        {
            Ok(None)
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct DummyMessageTextRepository;

    #[async_trait]
    impl MessageTextRepository for DummyMessageTextRepository {
        async fn get_by_id<'c, C>(
            &self,
            _db: &'c C,
            _id: &str,
        ) -> Result<Option<MessageTexts>, DbErr>
        where
            C: sea_orm::ConnectionTrait,
        {
            Ok(None)
        }
    }

    fn create_test_service()
    -> MessageService<DummyGuildMessageTextRepository, DummyMessageTextRepository> {
        MessageService::new(DummyGuildMessageTextRepository, DummyMessageTextRepository)
    }

    #[test]
    fn test_determine_locale() {
        let service = create_test_service();

        assert_eq!(service.determine_locale(Some("ja")), "ja");
        assert_eq!(service.determine_locale(Some("ja-JP")), "ja");
        assert_eq!(service.determine_locale(Some("en")), "en");
        assert_eq!(service.determine_locale(Some("en-US")), "en");
        assert_eq!(service.determine_locale(Some("fr")), "en");
        assert_eq!(service.determine_locale(None), "en");
    }

    #[test]
    fn test_replace_parameters_basic() {
        let service = create_test_service();
        let mut params = HashMap::new();
        params.insert("name".to_string(), "テスト".to_string());
        params.insert("value".to_string(), "123".to_string());

        let template = "こんにちは、{{name}}さん！値: {{value}}";
        let result = service.replace_parameters(template, &params);
        assert_eq!(result, "こんにちは、テストさん！値: 123");
    }

    #[test]
    fn test_replace_parameters_missing() {
        let service = create_test_service();
        let params = HashMap::new();

        let template = "値: {{missing}}";
        let result = service.replace_parameters(template, &params);
        // パラメータが存在しない場合はそのまま
        assert_eq!(result, "値: {{missing}}");
    }

    #[test]
    fn test_replace_parameters_escaped() {
        let service = create_test_service();
        let mut params = HashMap::new();
        params.insert("var".to_string(), "置換".to_string());

        // エスケープされた場合
        let template = r"エスケープ: \{{var}}、通常: {{var}}";
        let result = service.replace_parameters(template, &params);
        assert_eq!(result, "エスケープ: {{var}}、通常: 置換");
    }

    #[test]
    fn test_replace_parameters_double_backslash() {
        let service = create_test_service();
        let mut params = HashMap::new();
        params.insert("var".to_string(), "値".to_string());

        // \\ の場合
        let template = r"バックスラッシュ: \\{{var}}";
        let result = service.replace_parameters(template, &params);
        assert_eq!(result, r"バックスラッシュ: \値");
    }

    #[test]
    fn test_replace_parameters_complex() {
        let service = create_test_service();
        let mut params = HashMap::new();
        params.insert("quest".to_string(), "ドラゴンクエスト".to_string());
        params.insert("count".to_string(), "5".to_string());

        let template =
            r"クエスト「{{quest}}」に{{count}}人参加しています。\{{escaped}}は置換されません。";
        let result = service.replace_parameters(template, &params);
        assert_eq!(
            result,
            "クエスト「ドラゴンクエスト」に5人参加しています。{{escaped}}は置換されません。"
        );
    }
}
