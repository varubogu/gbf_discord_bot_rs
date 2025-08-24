# セキュリティルール

## 基本方針

- **プレゼンテーション層での必須入力検証**: ユーザー入力は必ずプレゼンテーション層で検証する
- **SQLインジェクション対策**: ORMを使用し、生SQL実行時は準備済みステートメントを使用
- **適切なサニタイゼーション**: ユーザー入力のサニタイゼーションを実施
- **Discord権限の適切な確認**: コマンド実行前にDiscordサーバー権限を確認
- **サーバー固有権限の実装**: アプリケーション独自の権限システムを実装

## 入力検証

### プレゼンテーション層での入力検証

```rust
use regex::Regex;
use std::collections::HashSet;

// ✅ 推奨: 包括的な入力検証
#[derive(Debug)]
pub struct InputValidator;

impl InputValidator {
    // 英数字とハイフンのみ許可（クエスト名用）
    pub fn validate_quest_alias(alias: &str) -> Result<String, ValidationError> {
        if alias.is_empty() {
            return Err(ValidationError::RequiredFieldMissing {
                field: "quest_alias".to_string()
            });
        }

        if alias.len() > 50 {
            return Err(ValidationError::ValueOutOfRange {
                field: "quest_alias".to_string(),
                value: format!("長さ: {}", alias.len())
            });
        }

        let valid_pattern = Regex::new(r"^[a-zA-Z0-9\-_]+$").unwrap();
        if !valid_pattern.is_match(alias) {
            return Err(ValidationError::InvalidFormat {
                field: "quest_alias".to_string()
            });
        }

        Ok(alias.to_lowercase())
    }

    // Discord User IDの検証
    pub fn validate_discord_user_id(user_id: &str) -> Result<u64, ValidationError> {
        user_id.parse::<u64>()
            .map_err(|_| ValidationError::InvalidFormat {
                field: "user_id".to_string()
            })
    }

    // 参加者数の検証
    pub fn validate_participant_count(count: i32) -> Result<i32, ValidationError> {
        if count < 1 || count > 30 {
            return Err(ValidationError::ValueOutOfRange {
                field: "participant_count".to_string(),
                value: count.to_string()
            });
        }
        Ok(count)
    }
}

// ❌ 危険: 検証なしの入力受け取り
pub async fn dangerous_command(ctx: PoiseContext, raw_input: String) -> Result<(), PoiseError> {
    // 危険: 入力検証なしでそのまま使用
    let query = format!("SELECT * FROM quests WHERE name = '{}'", raw_input);
    // SQLインジェクションの脆弱性あり
}
```

### 許可リストベースの検証

```rust
// ✅ 推奨: 許可リストによる検証
pub struct AllowedValues;

impl AllowedValues {
    pub const BATTLE_TYPES: &'static [&'static str] = &[
        "raid", "trial", "event", "guild_war"
    ];

    pub const ROLES: &'static [&'static str] = &[
        "attacker", "healer", "support", "tank"
    ];

    pub fn validate_battle_type(battle_type: &str) -> Result<String, ValidationError> {
        if Self::BATTLE_TYPES.contains(&battle_type) {
            Ok(battle_type.to_string())
        } else {
            Err(ValidationError::InvalidFormat {
                field: "battle_type".to_string()
            })
        }
    }

    pub fn validate_role(role: &str) -> Result<String, ValidationError> {
        if Self::ROLES.contains(&role) {
            Ok(role.to_string())
        } else {
            Err(ValidationError::InvalidFormat {
                field: "role".to_string()
            })
        }
    }
}
```

## SQLインジェクション対策

### 安全なデータベースクエリ

```rust
use sea_orm::*;

// ✅ 推奨: ORMを使用した安全なクエリ
impl BattleRecruitmentRepository {
    pub async fn find_by_quest_name_safe(
        &self,
        quest_name: &str,
        tx: &DatabaseTransaction
    ) -> Result<Vec<BattleRecruitment>, DataAccessError> {
        // ORMによる自動エスケープ
        let recruitments = battle_recruitment::Entity::find()
            .filter(battle_recruitment::Column::QuestName.eq(quest_name))  // 自動エスケープ
            .all(tx)
            .await?;

        Ok(recruitments.into_iter().map(Into::into).collect())
    }

    pub async fn find_by_complex_condition(
        &self,
        quest_name: &str,
        min_participants: i32,
        max_participants: i32,
        tx: &DatabaseTransaction
    ) -> Result<Vec<BattleRecruitment>, DataAccessError> {
        // 複数条件も安全
        let recruitments = battle_recruitment::Entity::find()
            .filter(
                Condition::all()
                    .add(battle_recruitment::Column::QuestName.eq(quest_name))
                    .add(battle_recruitment::Column::CurrentParticipants.gte(min_participants))
                    .add(battle_recruitment::Column::MaxParticipants.lte(max_participants))
            )
            .all(tx)
            .await?;

        Ok(recruitments.into_iter().map(Into::into).collect())
    }
}

// ❌ 危険: 生SQLによる脆弱性
impl DangerousRepository {
    pub async fn dangerous_find(&self, user_input: &str, tx: &DatabaseTransaction) -> Result<Vec<BattleRecruitment>, DataAccessError> {
        // 危険: SQLインジェクション脆弱性
        let query = format!(
            "SELECT * FROM battle_recruitments WHERE quest_name = '{}'",
            user_input  // エスケープなし
        );

        // この実装は使用禁止
        unimplemented!("SQLインジェクション脆弱性があるため実装禁止")
    }
}
```

### 準備済みステートメントの使用（必要な場合）

```rust
// ✅ 推奨: 準備済みステートメントによる安全な生SQL実行
impl AdvancedQueryRepository {
    pub async fn execute_safe_raw_query(
        &self,
        quest_name: &str,
        tx: &DatabaseTransaction
    ) -> Result<Vec<QueryResult>, DataAccessError> {
        // 準備済みステートメントを使用
        let results = tx
            .query_all(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "SELECT id, quest_name, current_participants FROM battle_recruitments WHERE quest_name = $1",
                vec![quest_name.into()],  // パラメータバインディング
            ))
            .await?;

        Ok(results)
    }
}
```

## 適切なサニタイゼーション

### Discord メッセージのサニタイゼーション

```rust
use regex::Regex;

// ✅ 推奨: メッセージサニタイゼーション
pub struct MessageSanitizer;

impl MessageSanitizer {
    pub fn sanitize_user_message(message: &str) -> String {
        // Discord固有の制御文字を除去
        let discord_controls = Regex::new(r"@(everyone|here|&\d+|!\d+)").unwrap();
        let sanitized = discord_controls.replace_all(message, "[メンション]");

        // マークダウンの無効化
        let markdown_chars = Regex::new(r"[`*_~|]").unwrap();
        let sanitized = markdown_chars.replace_all(&sanitized, "\\$0");

        // 長すぎるメッセージの切り詰め
        if sanitized.len() > 2000 {
            format!("{}...", &sanitized[..1997])
        } else {
            sanitized.to_string()
        }
    }

    pub fn sanitize_quest_name(quest_name: &str) -> String {
        // クエスト名の正規化
        quest_name
            .trim()
            .replace('\n', " ")
            .replace('\r', " ")
            .replace('\t', " ")
    }

    pub fn sanitize_for_log(data: &str) -> String {
        // ログ出力用のサニタイゼーション（改行コードなどを除去）
        data
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
}

// 使用例
impl RecruitmentMessageService {
    pub async fn send_recruitment_message(
        &self,
        ctx: &PoiseContext,
        quest_name: &str,
        description: &str,
    ) -> Result<(), PoiseError> {
        let safe_quest_name = MessageSanitizer::sanitize_quest_name(quest_name);
        let safe_description = MessageSanitizer::sanitize_user_message(description);

        let message = format!(
            "**募集**: {}\n**説明**: {}",
            safe_quest_name,
            safe_description
        );

        ctx.say(message).await?;
        Ok(())
    }
}
```

## Discord権限チェック

### 基本権限の確認

```rust
use serenity::model::permissions::Permissions;

// ✅ 推奨: 段階的権限チェック
pub struct PermissionChecker;

impl PermissionChecker {
    // 基本的なDiscord権限チェック
    pub fn check_basic_permissions(ctx: &PoiseContext) -> Result<(), PermissionError> {
        let guild_id = ctx.guild_id()
            .ok_or(PermissionError::GuildRequired)?;

        let member = ctx.author_member()
            .ok_or(PermissionError::MemberInfoRequired)?;

        // メッセージ送信権限の確認
        if !member.permissions(ctx.cache())?.contains(Permissions::SEND_MESSAGES) {
            return Err(PermissionError::InsufficientDiscordPermission {
                required: "SEND_MESSAGES".to_string()
            });
        }

        Ok(())
    }

    // 管理者権限が必要なコマンド用
    pub fn check_admin_permissions(ctx: &PoiseContext) -> Result<(), PermissionError> {
        Self::check_basic_permissions(ctx)?;

        let member = ctx.author_member().unwrap();  // 上記で確認済み

        if !member.permissions(ctx.cache())?.contains(Permissions::ADMINISTRATOR) {
            return Err(PermissionError::AdminRequired);
        }

        Ok(())
    }

    // モデレーター権限（複数権限のいずれかが必要）
    pub fn check_moderator_permissions(ctx: &PoiseContext) -> Result<(), PermissionError> {
        Self::check_basic_permissions(ctx)?;

        let member = ctx.author_member().unwrap();
        let permissions = member.permissions(ctx.cache())?;

        let moderator_permissions = [
            Permissions::ADMINISTRATOR,
            Permissions::MANAGE_MESSAGES,
            Permissions::MANAGE_CHANNELS,
            Permissions::KICK_MEMBERS,
        ];

        if !moderator_permissions.iter().any(|&perm| permissions.contains(perm)) {
            return Err(PermissionError::ModeratorRequired);
        }

        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PermissionError {
    #[error("このコマンドはサーバー内でのみ実行できます")]
    GuildRequired,

    #[error("メンバー情報を取得できません")]
    MemberInfoRequired,

    #[error("必要な権限が不足しています: {required}")]
    InsufficientDiscordPermission { required: String },

    #[error("このコマンドには管理者権限が必要です")]
    AdminRequired,

    #[error("このコマンドにはモデレーター権限が必要です")]
    ModeratorRequired,
}
```

### アプリケーション固有権限システム

```rust
// ✅ 推奨: アプリケーション独自権限システム
#[derive(Debug, Clone)]
pub enum ApplicationRole {
    BattleMaster,      // バトル募集管理者
    EventOrganizer,    // イベント主催者
    Moderator,         // モデレーター
    Member,            // 一般メンバー
}

pub struct ApplicationPermissionService {
    user_roles: Arc<dyn UserRoleRepository>,
}

impl ApplicationPermissionService {
    pub async fn check_battle_management_permission(
        &self,
        user_id: &UserId,
        guild_id: &GuildId,
        tx: &Transaction,
    ) -> Result<(), PermissionError> {
        let user_roles = self.user_roles.find_by_user_and_guild(user_id, guild_id, tx).await?;

        let has_permission = user_roles.iter().any(|role| {
            matches!(role, ApplicationRole::BattleMaster | ApplicationRole::Moderator)
        });

        if !has_permission {
            return Err(PermissionError::InsufficientApplicationPermission {
                required: "BattleMaster or Moderator".to_string(),
            });
        }

        Ok(())
    }

    pub async fn can_manage_recruitment(
        &self,
        user_id: &UserId,
        recruitment: &BattleRecruitment,
        guild_id: &GuildId,
        tx: &Transaction,
    ) -> Result<bool, PermissionError> {
        // 作成者本人は常に管理可能
        if recruitment.creator_id() == user_id {
            return Ok(true);
        }

        // アプリケーション権限をチェック
        match self.check_battle_management_permission(user_id, guild_id, tx).await {
            Ok(()) => Ok(true),
            Err(PermissionError::InsufficientApplicationPermission { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }
}
```

## 機密情報の保護

### 環境変数の安全な管理

```rust
// ✅ 推奨: 安全な環境変数管理
pub struct SecureConfig {
    discord_token: String,
    database_url: String,
    encryption_key: String,
}

impl SecureConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let discord_token = std::env::var("DISCORD_TOKEN")
            .map_err(|_| ConfigError::MissingRequiredEnv("DISCORD_TOKEN".to_string()))?;

        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingRequiredEnv("DATABASE_URL".to_string()))?;

        let encryption_key = std::env::var("ENCRYPTION_KEY")
            .map_err(|_| ConfigError::MissingRequiredEnv("ENCRYPTION_KEY".to_string()))?;

        // 基本的な検証
        if discord_token.len() < 50 {
            return Err(ConfigError::InvalidTokenFormat);
        }

        Ok(Self {
            discord_token,
            database_url,
            encryption_key,
        })
    }

    pub fn discord_token(&self) -> &str {
        &self.discord_token
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }
}

// Debugトレイトの実装で機密情報を隠蔽
impl std::fmt::Debug for SecureConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecureConfig")
            .field("discord_token", &"***")  // 機密情報を隠蔽
            .field("database_url", &"***")
            .field("encryption_key", &"***")
            .finish()
    }
}
```

### ログでの機密情報漏洩防止

```rust
use tracing::info;

// ✅ 推奨: 安全なログ出力
impl UserService {
    pub async fn authenticate_user(&self, user_id: &UserId, token: &str) -> Result<User, AuthError> {
        // ❌ 危険: トークンをログに出力
        // info!("Authenticating user {} with token: {}", user_id, token);

        // ✅ 推奨: 機密情報を除外したログ
        info!(
            user_id = %user_id,
            token_length = token.len(),
            "ユーザー認証を開始"
        );

        let result = self.verify_token(user_id, token).await;

        info!(
            user_id = %user_id,
            success = result.is_ok(),
            "ユーザー認証完了"
        );

        result
    }
}
```

## セキュリティテストの指針

```rust
#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_input_validation_prevents_injection() {
        // SQLインジェクション攻撃文字列のテスト
        let malicious_inputs = [
            "'; DROP TABLE battle_recruitments; --",
            "' OR '1'='1",
            "'; DELETE FROM users WHERE '1'='1'; --",
            "<script>alert('xss')</script>",
            "@everyone @here",
        ];

        for input in &malicious_inputs {
            assert!(
                InputValidator::validate_quest_alias(input).is_err(),
                "悪意のある入力が拒否されませんでした: {}",
                input
            );
        }
    }

    #[test]
    fn test_message_sanitization() {
        let dangerous_message = "@everyone Hello <script>alert('xss')</script> @here";
        let sanitized = MessageSanitizer::sanitize_user_message(dangerous_message);

        assert!(!sanitized.contains("@everyone"));
        assert!(!sanitized.contains("@here"));
        assert!(!sanitized.contains("<script>"));
    }

    #[tokio::test]
    async fn test_permission_enforcement() {
        let ctx = create_test_context_without_admin().await;

        let result = PermissionChecker::check_admin_permissions(&ctx);
        assert!(result.is_err(), "管理者権限のないユーザーでも成功してしまいました");
    }

    #[test]
    fn test_secure_config_debug_hiding() {
        let config = SecureConfig {
            discord_token: "very_secret_token".to_string(),
            database_url: "postgres://secret".to_string(),
            encryption_key: "secret_key".to_string(),
        };

        let debug_output = format!("{:?}", config);
        assert!(!debug_output.contains("very_secret_token"));
        assert!(!debug_output.contains("postgres://secret"));
        assert!(debug_output.contains("***"));
    }
}
```

## セキュリティチェックリスト

### 開発時のチェックポイント

1. **入力検証**
    - [ ] すべてのユーザー入力が検証されている
    - [ ] 許可リストベースの検証が実装されている
    - [ ] 入力長制限が適切に設定されている

2. **SQLインジェクション対策**
    - [ ] ORMを使用している
    - [ ] 生SQLを使用する場合は準備済みステートメントを使用
    - [ ] 動的クエリ生成を避けている

3. **権限管理**
    - [ ] Discord権限チェックが実装されている
    - [ ] アプリケーション固有権限が適切に管理されている
    - [ ] 権限昇格攻撃に対する保護がある

4. **機密情報保護**
    - [ ] 環境変数が適切に管理されている
    - [ ] ログに機密情報が出力されていない
    - [ ] Debugトレイトで機密情報が隠蔽されている

5. **エラーハンドリング**
    - [ ] エラーメッセージに機密情報が含まれていない
    - [ ] 適切なエラーレスポンスが実装されている
    - [ ] スタックトレースが本番環境で無効化されている