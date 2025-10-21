# ロール権限システム設計書

## 概要

GBF Discord Botの権限管理システムの設計書です。Discordのロール機能を活用し、Botの機能に応じた適切な権限管理を実現します。

## 設計方針

### 基本原則
1. **最小権限の原則**: 必要最小限の権限のみ付与
2. **階層的権限管理**: ロールの階層構造による権限継承
3. **明確な責務分離**: 各ロールの責務を明確に定義
4. **拡張性の確保**: 将来の機能追加に対応可能な設計

### セキュリティ要件
- 機密情報へのアクセス制御
- システム設定変更の権限管理
- データ操作の監査可能性
- 権限昇格の防止

## ロール設計

### 1. システム管理者ロール

#### `gbf_bot_control`
- **目的**: Botのシステム管理・設定変更
- **権限レベル**: 最高権限
- **責務**:
  - 環境変数の管理
  - スプレッドシート連携の管理
  - システム設定の変更
  - 緊急時のBot操作

#### 実装済み機能
```rust
// 現在実装済みの権限チェック
pub async fn has_bot_control_permission(
    ctx: &PoiseContext<'_>,
    member: &Member,
) -> Result<(), String>
```

#### 対象コマンド
- `/environ_load` - 環境変数リロード
- `/gspread_load` - スプレッドシート読み込み
- `/gspread_push` - スプレッドシート書き込み

### 2. イベント管理者ロール（将来実装予定）

#### `gbf_event_manager`
- **目的**: イベント・スケジュール管理
- **権限レベル**: 高権限
- **責務**:
  - イベントスケジュールの作成・編集
  - イベント通知の管理
  - イベント参加者の管理
  - イベント統計の確認

#### 対象機能（将来実装）
- イベントスケジュール管理
- イベント通知システム
- イベント統計レポート

### 3. 募集管理者ロール（将来実装予定）

#### `gbf_recruitment_manager`
- **目的**: 募集システムの管理
- **権限レベル**: 中権限
- **責務**:
  - 募集の強制終了
  - 募集ルールの管理
  - 募集統計の確認
  - 問題のある募集の対応

#### 対象機能（将来実装）
- 募集の強制終了
- 募集ルール設定
- 募集統計レポート

### 4. 一般ユーザー

#### デフォルト権限
- **目的**: 基本的なBot機能の利用
- **権限レベル**: 基本権限
- **責務**:
  - 募集の作成・参加・キャンセル
  - ヘルプの確認
  - 基本的な情報取得

#### 対象コマンド
- `/recruit` - 募集作成
- `/recruit_cancel` - 募集キャンセル
- `/recruit_change` - 募集内容変更
- `/help` - ヘルプ表示

## 権限チェック実装

### 現在の実装

#### 基本権限チェック関数
```rust
// src/services/permission/mod.rs
pub async fn has_role(
    ctx: &PoiseContext<'_>,
    member: &Member,
    role_name: &str,
) -> Result<(), String>
```

#### システム管理者権限チェック
```rust
pub async fn has_bot_control_permission(
    ctx: &PoiseContext<'_>,
    member: &Member,
) -> Result<(), String>
```

### 将来の拡張実装

#### 階層的権限チェック
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionLevel {
    SystemAdmin,    // gbf_bot_control
    EventManager,   // gbf_event_manager
    RecruitmentManager, // gbf_recruitment_manager
    User,           // 一般ユーザー
}

impl PermissionLevel {
    pub fn from_role(role_name: &str) -> Option<Self> {
        match role_name {
            "gbf_bot_control" => Some(PermissionLevel::SystemAdmin),
            "gbf_event_manager" => Some(PermissionLevel::EventManager),
            "gbf_recruitment_manager" => Some(PermissionLevel::RecruitmentManager),
            _ => None,
        }
    }

    pub fn has_permission(&self, required: &PermissionLevel) -> bool {
        match (self, required) {
            (PermissionLevel::SystemAdmin, _) => true,
            (PermissionLevel::EventManager, PermissionLevel::EventManager) => true,
            (PermissionLevel::EventManager, PermissionLevel::RecruitmentManager) => true,
            (PermissionLevel::EventManager, PermissionLevel::User) => true,
            (PermissionLevel::RecruitmentManager, PermissionLevel::RecruitmentManager) => true,
            (PermissionLevel::RecruitmentManager, PermissionLevel::User) => true,
            (PermissionLevel::User, PermissionLevel::User) => true,
            _ => false,
        }
    }
}
```

#### 複数権限チェック
```rust
pub async fn has_any_role(
    ctx: &PoiseContext<'_>,
    member: &Member,
    role_names: &[&str],
) -> Result<(), String> {
    for role_name in role_names {
        if has_role(ctx, member, role_name).await.is_ok() {
            return Ok(());
        }
    }
    Err("必要な権限がありません".to_string())
}

pub async fn has_all_roles(
    ctx: &PoiseContext<'_>,
    member: &Member,
    role_names: &[&str],
) -> Result<(), String> {
    for role_name in role_names {
        has_role(ctx, member, role_name).await?;
    }
    Ok(())
}
```

## コマンド権限マトリックス

### 現在実装済み

| コマンド | 権限要件 | 実装状況 |
|---------|---------|---------|
| `/help` | なし | ✅ 実装済み |
| `/recruit` | なし | ✅ 実装済み |
| `/recruit_cancel` | 作成者本人 | ✅ 実装済み |
| `/recruit_change` | 作成者本人 | ✅ 実装済み |
| `/environ_load` | `gbf_bot_control` | ✅ 実装済み |
| `/gspread_load` | `gbf_bot_control` | ✅ 実装済み |
| `/gspread_push` | `gbf_bot_control` | ✅ 実装済み |

### 将来実装予定

| コマンド | 権限要件 | 実装予定 |
|---------|---------|---------|
| `/event_create` | `gbf_event_manager` | 将来実装 |
| `/event_edit` | `gbf_event_manager` | 将来実装 |
| `/recruit_force_cancel` | `gbf_recruitment_manager` | 将来実装 |
| `/recruit_stats` | `gbf_recruitment_manager` | 将来実装 |

## エラーハンドリング

### 権限エラーの種類

```rust
#[derive(thiserror::Error, Debug)]
pub enum PermissionError {
    #[error("ロールが見つかりません: {role_name}")]
    RoleNotFound { role_name: String },
    
    #[error("権限が不足しています: {user_name} は {role_name} ロールを持っていません")]
    InsufficientPermission { user_name: String, role_name: String },
    
    #[error("Guild情報を取得できません")]
    GuildInfoRequired,
    
    #[error("メンバー情報を取得できません")]
    MemberInfoRequired,
}
```

### エラーメッセージの標準化

```rust
impl PermissionError {
    pub fn user_friendly_message(&self) -> String {
        match self {
            PermissionError::RoleNotFound { role_name } => {
                format!("ロール '{}' がサーバーに存在しません。管理者に確認してください。", role_name)
            }
            PermissionError::InsufficientPermission { user_name, role_name } => {
                format!("{} さんは '{}' ロールを持っていないため、このコマンドを実行できません。", user_name, role_name)
            }
            PermissionError::GuildInfoRequired => {
                "サーバー情報を取得できませんでした。".to_string()
            }
            PermissionError::MemberInfoRequired => {
                "メンバー情報を取得できませんでした。".to_string()
            }
        }
    }
}
```

## セキュリティ考慮事項

### 1. 権限昇格の防止
- ロールの階層構造を明確に定義
- 権限チェックの二重化
- ログによる監査証跡の確保

### 2. 機密情報の保護
- システム設定へのアクセス制限
- データベース操作の権限管理
- 外部API連携の権限制御

### 3. 監査ログ
```rust
pub struct PermissionAuditLog {
    pub user_id: u64,
    pub guild_id: u64,
    pub command: String,
    pub required_role: String,
    pub result: PermissionResult,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub enum PermissionResult {
    Granted,
    Denied { reason: String },
}
```

## 実装ガイドライン

### 1. 権限チェックの実装パターン

#### コマンドレベルでの権限チェック
```rust
#[poise::command(slash_command)]
pub async fn admin_command(ctx: PoiseContext<'_>) -> Result<()> {
    // 権限チェック
    let member = ctx.author_member().await
        .ok_or("メンバー情報を取得できません")?;
    
    has_bot_control_permission(ctx, &member).await?;
    
    // コマンド実行
    // ...
}
```

#### Facade層での権限チェック
```rust
pub async fn admin_operation(
    ctx: &PoiseContext<'_>,
    operation: AdminOperation,
) -> Result<()> {
    // 権限チェック
    let member = ctx.author_member().await
        .ok_or("メンバー情報を取得できません")?;
    
    has_bot_control_permission(ctx, &member).await?;
    
    // 操作実行
    // ...
}
```

### 2. 権限の段階的導入

権限システムは、基本ロールによる管理を基点とし、階層化や細粒度化へ段階的に拡張できるよう責務を分離して設計しています。Facade層での権限判定や監査ログの集約ポイントを確保することで、追加要件に対しても既存構造を保ったまま拡張しやすい構成としています。

## テスト戦略

### 1. 単体テスト
```rust
#[tokio::test]
async fn test_has_bot_control_permission_success() {
    // 権限を持つユーザーのテスト
}

#[tokio::test]
async fn test_has_bot_control_permission_denied() {
    // 権限を持たないユーザーのテスト
}
```

### 2. 統合テスト
```rust
#[tokio::test]
async fn test_admin_command_permission_check() {
    // 管理者コマンドの権限チェック統合テスト
}
```

### 3. セキュリティテスト
- 権限昇格のテスト
- 不正アクセスのテスト
- エラーケースのテスト

## 運用ガイドライン

### 1. ロール設定手順
1. Discordサーバーでロールを作成
2. 適切なメンバーにロールを付与
3. Botの権限設定を確認
4. 動作テストを実行

### 2. 権限監査
- 定期的な権限確認
- 不要な権限の削除
- ログの監視

### 3. トラブルシューティング
- 権限エラーの診断手順
- ロール設定の確認方法
- エラーログの確認方法

## まとめ

この設計書は、GBF Discord Botの権限管理システムの現在の実装状況と将来の拡張計画を示しています。現在は`gbf_bot_control`ロールによる基本的な権限管理が実装されており、将来的には階層的な権限システムへの拡張を予定しています。

セキュリティと使いやすさのバランスを保ちながら、Botの機能拡張に合わせて権限システムも段階的に発展させていく方針です。
