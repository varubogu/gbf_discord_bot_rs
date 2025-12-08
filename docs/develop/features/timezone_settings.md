# タイムゾーン設定機能 設計書

## 概要

Discordサーバー（Guild）ごとにタイムゾーンを設定し、日時入力および表示を現地時間で行う機能です。ユーザーがスラッシュコマンドでタイムゾーンを設定すると、以降の募集コマンドやスプレッドシート読み込みで指定したタイムゾーンが適用されます。

## 機能要件

### 基本機能
- スラッシュコマンド `/タイムゾーン設定` によるタイムゾーン設定
- スラッシュコマンド `/タイムゾーン確認` による現在設定の確認
- IANAタイムゾーンデータベース準拠のタイムゾーン名指定（例: `Asia/Tokyo`, `America/New_York`）
- デフォルトタイムゾーン: `Asia/Tokyo`
- `gbf_bot_control` ロール保持者のみ設定変更可能

### タイムゾーン設定の影響範囲

#### 影響する処理
1. **ユーザー入力の解釈**
   - `/マルチバトル募集` コマンドの日時入力
   - `/マルチバトル募集内容変更` コマンドの日時入力
   - スプレッドシート読み込み時の日時解釈

2. **表示処理**
   - 募集メッセージの開催日時表示
   - 通知メッセージの日時表示
   - スケジュール表示

#### 影響しない処理
- DB保存データ（常にUTC）
- 内部計算処理（常にUTC）
- タイムゾーン変更前に作成された募集（既存データはUTCのまま保存されており、表示時のみタイムゾーン変換）

### 制約事項
- タイムゾーン設定変更は即座に反映される
- 既存の募集データのメッセージやDBは自動的に再計算されない
- タイムゾーンは真っ先に設定すべき設定項目として位置づけられる

## アーキテクチャ設計

### 層別責務

#### プレゼンテーション層（events/）
```
src/events/interactions/command_interactions/slash/timezone_set.rs
src/events/interactions/command_interactions/slash/timezone_show.rs
```
- Discord API操作の実装
- スラッシュコマンドの定義
- 権限チェック（`gbf_bot_control` ロール）
- エラーハンドリング
- 設定完了メッセージ表示

#### Facade層（facades/）
```
src/facades/timezone/set_timezone.rs
src/facades/timezone/show_timezone.rs
```
- サービス層の協調
- トランザクション境界管理
- 設定結果の集約

#### Service層（services/）
```
src/services/timezone_service.rs
```
- タイムゾーン取得・設定のビジネスロジック
- タイムゾーン名のバリデーション（chrono-tzでパース可能か）
- デフォルトタイムゾーンの返却（未設定時）

#### Repository層（repository/）
```
src/repository/database/guild_timezone_repository.rs
```
- タイムゾーン設定の永続化
- ギルドIDによる検索
- UPSERT処理

## データモデル

### 主要エンティティ

#### GuildTimezones
```rust
pub struct Model {
    pub guild_id: i64,           // Discord Guild ID
    pub timezone: String,        // IANAタイムゾーン名
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## コマンド仕様

### `/タイムゾーン設定` コマンド

#### パラメータ
| パラメータ名 | 型 | 必須 | 説明 | 例 |
|-------------|-----|------|------|-----|
| timezone | String | ✅ | IANAタイムゾーン名 | `Asia/Tokyo`, `America/New_York`, `Europe/London` |

#### 動作フロー
1. コマンド実行者の権限チェック（`gbf_bot_control` ロール保持確認）
2. タイムゾーン名のバリデーション（chrono-tzでパース可能か）
3. DB保存（UPSERT）
4. 完了メッセージ表示

#### 成功時の表示例
```
タイムゾーンを Asia/Tokyo (JST) に設定しました。
```

#### エラー例
- 権限不足: `このコマンドは管理者（gbf_bot_controlロール）のみ実行できます。`
- 無効なタイムゾーン: `無効なタイムゾーン名です。IANAタイムゾーン名を指定してください。（例: Asia/Tokyo, America/New_York）`

### `/タイムゾーン確認` コマンド

#### パラメータ
なし

#### 動作フロー
1. 現在のギルド設定を取得
2. タイムゾーン名と略称を表示

#### 表示例
```
現在のタイムゾーン: Asia/Tokyo (JST)
デフォルト設定: はい
```

または設定済みの場合：
```
現在のタイムゾーン: America/New_York (EST)
デフォルト設定: いいえ
```

## サービス層API設計

### TimezoneService

```rust
pub struct TimezoneService {
    repository: Arc<GuildTimezoneRepository>,
}

impl TimezoneService {
    /// ギルドのタイムゾーンを取得
    /// 未設定の場合はデフォルト（Asia/Tokyo）を返す
    pub async fn get_guild_timezone(&self, guild_id: GuildId) -> Result<Tz> {
        match self.repository.find_by_guild_id(guild_id).await? {
            Some(settings) => {
                settings.timezone.parse::<Tz>()
                    .map_err(|_| Error::InvalidTimezone)
            },
            None => {
                Ok(chrono_tz::Asia::Tokyo)
            }
        }
    }

    /// ギルドのタイムゾーンを設定
    pub async fn set_guild_timezone(
        &self,
        guild_id: GuildId,
        timezone: Tz,
    ) -> Result<()> {
        // バリデーション済みのTz型を受け取るため、そのまま保存
        self.repository.upsert(guild_id, timezone.name()).await
    }

    /// タイムゾーン名のバリデーション
    pub fn validate_timezone(timezone_str: &str) -> Result<Tz> {
        timezone_str.parse::<Tz>()
            .map_err(|_| Error::InvalidTimezone)
    }
}
```

## リポジトリ層API設計

### GuildTimezoneRepository

```rust
pub struct GuildTimezoneRepository {
    db: Arc<DatabaseConnection>,
}

impl GuildTimezoneRepository {
    /// ギルドIDでタイムゾーン設定を検索
    pub async fn find_by_guild_id(
        &self,
        guild_id: GuildId,
    ) -> Result<Option<guild_timezones::Model>> {
        guild_timezones::Entity::find_by_id(guild_id.get() as i64)
            .one(self.db.as_ref())
            .await
            .map_err(Into::into)
    }

    /// タイムゾーン設定を保存（UPSERT）
    pub async fn upsert(
        &self,
        guild_id: GuildId,
        timezone: &str,
    ) -> Result<()> {
        let now = Utc::now();
        let model = guild_timezones::ActiveModel {
            guild_id: Set(guild_id.get() as i64),
            timezone: Set(timezone.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        };

        guild_timezones::Entity::insert(model)
            .on_conflict(
                OnConflict::column(guild_timezones::Column::GuildId)
                    .update_column(guild_timezones::Column::Timezone)
                    .update_column(guild_timezones::Column::UpdatedAt)
                    .to_owned()
            )
            .exec(self.db.as_ref())
            .await?;

        Ok(())
    }
}
```

## エラーハンドリング

### エラー型定義

```rust
#[derive(Debug, thiserror::Error)]
pub enum TimezoneError {
    #[error("無効なタイムゾーン名です")]
    InvalidTimezone,

    #[error("権限がありません")]
    PermissionDenied,

    #[error("データベースエラー: {0}")]
    DatabaseError(#[from] DbErr),
}
```

## テスト計画

### 単体テスト

#### TimezoneService
- [ ] 有効なタイムゾーン名のバリデーション成功
- [ ] 無効なタイムゾーン名のバリデーション失敗
- [ ] デフォルトタイムゾーンの返却（未設定時）
- [ ] 設定済みタイムゾーンの取得

#### GuildTimezoneRepository
- [ ] UPSERT処理（新規作成）
- [ ] UPSERT処理（更新）
- [ ] ギルドIDによる検索成功
- [ ] 存在しないギルドIDの検索（None返却）

### 統合テスト

- [ ] `/タイムゾーン設定` コマンド実行成功
- [ ] `/タイムゾーン設定` コマンドでの無効なタイムゾーン入力
- [ ] `/タイムゾーン設定` コマンドでの権限不足
- [ ] `/タイムゾーン確認` コマンド実行（デフォルト状態）
- [ ] `/タイムゾーン確認` コマンド実行（設定済み状態）
- [ ] タイムゾーン設定後の募集コマンドでの日時解釈確認

## マイグレーション

### マイグレーションファイル
```
migration/src/m20251208_000000_create_guild_timezones.rs
```

### DDL
```sql
CREATE TABLE guild_timezones (
    guild_id BIGINT PRIMARY KEY,
    timezone VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
```

## 主要なタイムゾーン名一覧（参考）

| タイムゾーン名 | 説明 | 略称例 |
|--------------|------|--------|
| Asia/Tokyo | 日本標準時 | JST |
| America/New_York | 米国東部時間 | EST/EDT |
| America/Los_Angeles | 米国太平洋時間 | PST/PDT |
| Europe/London | グリニッジ標準時 | GMT/BST |
| Europe/Paris | 中央ヨーロッパ時間 | CET/CEST |
| Australia/Sydney | オーストラリア東部時間 | AEDT/AEST |

## 実装チェックリスト

設計書段階:
- [x] 機能要件定義
- [x] アーキテクチャ設計
- [x] データモデル設計
- [x] コマンド仕様定義
- [x] エラーハンドリング設計
- [x] テスト計画

実装段階:
- [ ] Cargo.toml に chrono-tz 追加
- [ ] マイグレーションファイル作成
- [ ] エンティティ定義（guild_timezones::Model）
- [ ] リポジトリ実装
- [ ] サービス実装
- [ ] Facade実装
- [ ] コマンド実装（/タイムゾーン設定）
- [ ] コマンド実装（/タイムゾーン確認）
- [ ] datetime_parserサービス修正
- [ ] 表示処理修正
- [ ] スプレッドシート読み込み修正
- [ ] テストコード作成
