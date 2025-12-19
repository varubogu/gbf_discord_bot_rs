# ギルド環境変数による属性絵文字カスタマイズ機能

## 概要

サーバー（ギルド）ごとに、マルチ募集で使用する属性絵文字（火・水・土・風・光・闇）をカスタマイズ可能にする機能。

`guild_environments` テーブルに特定の環境変数を設定することで、デフォルトの絵文字を上書きできる。

## 対象範囲

### 適用箇所

- マルチバトル募集コマンド（v1）のリアクション
- マルチバトル募集（v2）のボタンと参加者一覧
- スケジュール募集からの自動作成
- 募集内容変更時の更新

### 環境変数キー

| キー | 説明 |
|------|------|
| `ELEMENT_FIRE` | 火属性の絵文字 |
| `ELEMENT_WATER` | 水属性の絵文字 |
| `ELEMENT_EARTH` | 土属性の絵文字 |
| `ELEMENT_WIND` | 風属性の絵文字 |
| `ELEMENT_LIGHT` | 光属性の絵文字 |
| `ELEMENT_DARK` | 闇属性の絵文字 |

## アーキテクチャ設計

### レイヤー構成

```
Events層
  ↓
Facades層
  ↓ (GuildEnvironmentService呼び出し)
Services層 (GuildEnvironmentService)
  ↓ (Discord API)
  ├─ サーバー絵文字一覧取得
  └─ 絵文字解決・変換
  ↓ (Repository呼び出し)
Repository層 (GuildEnvironmentRepository)
  ↓
Database (guild_environments)
```

### データフロー

1. **環境変数取得**: Repository層がDBから6属性の環境変数を一括取得
2. **サーバー絵文字取得**: Discord APIからサーバーの絵文字一覧を取得
3. **絵文字解決**: 環境変数の値を実際に使用可能な絵文字に変換
4. **フォールバック**: 使用不可の場合はデフォルト値を使用

## コンポーネント設計

### Repository層

#### GuildEnvironmentRepository (Trait)

**ファイル**: `src/repository/guild_environments_repository.rs`

```rust
#[async_trait]
pub trait GuildEnvironmentRepository: Send + Sync {
    /// 単一環境変数を取得
    async fn get_by_guild_and_key<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
        key: &str,
    ) -> Result<Option<GuildEnvironments>, DbErr>;

    /// 複数環境変数を一括取得（N+1問題回避）
    async fn get_multiple_by_guild<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
        keys: &[&str],
    ) -> Result<HashMap<String, String>, DbErr>;

    /// 環境変数を設定（Upsert）
    async fn set_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        key: &str,
        value: &str,
    ) -> Result<GuildEnvironments, DbErr>;
}
```

**パフォーマンス最適化**:
- `get_multiple_by_guild()`: 6属性を1クエリで取得
  ```sql
  SELECT * FROM guild_master.guild_environments
  WHERE guild_id = $1 AND key IN ('ELEMENT_FIRE', 'ELEMENT_WATER', ...)
  ```

#### SeaOrmGuildEnvironmentRepository

**ファイル**: `src/repository/database/guild_environment_repository.rs`

SeaORMを使用した具体的な実装。`set_with_txn()` では既存レコードの更新と新規作成を自動判定（Upsert）。

### Service層

#### GuildEnvironmentService

**ファイル**: `src/services/guild_environment_service.rs`

**主要構造体**:

```rust
pub struct ElementEmojis {
    pub fire: String,
    pub water: String,
    pub earth: String,
    pub wind: String,
    pub light: String,
    pub dark: String,
}
```

**主要メソッド**:

##### get_element_emojis()

6属性の絵文字を取得・解決する。

```rust
pub async fn get_element_emojis<'c, C>(
    &self,
    db: &'c C,
    http: &Http,
    guild_id: i64,
) -> Result<ElementEmojis>
```

**処理フロー**:
1. DB から環境変数を一括取得
2. Discord API からサーバー絵文字一覧を取得
3. 各属性について `resolve_emoji()` で解決
4. 解決できない場合はデフォルト値を使用

##### resolve_emoji()

環境変数の値を実際に使用可能な絵文字形式に変換する。

```rust
fn resolve_emoji(value: &str, guild_emojis: &HashMap<u64, Emoji>) -> Option<String>
```

**サポート形式**:

| 形式 | 例 | 処理 |
|------|-----|------|
| Unicode絵文字 | `🔥` | そのまま返す |
| カスタム絵文字（完全） | `<:fire:123456789>` | サーバーに存在するか確認 |
| カスタム絵文字（名前） | `:fire:` | サーバー絵文字から名前検索 → `<:fire:id>` に変換 |
| アニメーション絵文字 | `<a:fire:123456789>` | サーバーに存在するか確認 |

**重要な機能: 絵文字名の自動変換**

`:emoji_name:` 形式が指定された場合、サーバーの絵文字一覧から名前で検索し、`<:emoji_name:id>` 形式に自動変換する。

```rust
// サーバー絵文字から名前で検索
for (emoji_id, emoji) in guild_emojis {
    if emoji.name == emoji_name {
        let resolved = if emoji.animated {
            format!("<a:{}:{}>", emoji_name, emoji_id)
        } else {
            format!("<:{}:{}>", emoji_name, emoji_id)
        };
        return Some(resolved);
    }
}
```

**理由**: マルチ募集v1（リアクション）では、正しい絵文字形式でないとリアクションが押せないため、`:fire:` のような形式は文字列として表示されてしまう。これを防ぐために自動変換を行う。

##### fetch_guild_emojis()

サーバーの絵文字一覧を取得する。

```rust
async fn fetch_guild_emojis(http: &Http, guild_id: u64) -> HashMap<u64, Emoji>
```

- 成功時: サーバーの全絵文字を `HashMap<絵文字ID, Emoji>` で返す
- 失敗時: 空のHashMapを返す（一部検証をスキップするがエラーにはしない）

##### extract_custom_emoji_id()

カスタム絵文字形式からIDを抽出する。

```rust
fn extract_custom_emoji_id(value: &str) -> Option<u64>
```

- `<:fire:123456789>` → `Some(123456789)`
- `<a:fire:123456789>` → `Some(123456789)`
- その他 → `None`

### Facade層での使用

**ファイル**:
- `src/facades/recruitment/new_recruit.rs`
- `src/facades/recruitment/button_handler.rs`
- `src/facades/recruitment/change.rs`
- `src/services/recruitment/recruitment_creation_service.rs`

**使用例**:

```rust
// 属性絵文字を取得（ギルド固有設定 or デフォルト値）
let guild_env_repo = Arc::new(SeaOrmGuildEnvironmentRepository::new());
let guild_env_service = GuildEnvironmentService::new(guild_env_repo);
let element_emojis = guild_env_service
    .get_element_emojis(conn, http, guild_id as i64)
    .await?;

// 募集データ作成時に使用
let recruitment_data = new::create_recruitment_data_with_repos(
    conn,
    &element_emojis,  // カスタム絵文字を渡す
    quest_alias,
    // ...
).await?;
```

## エラーハンドリング設計

### フォールバック戦略

すべてのエラーケースでデフォルト値にフォールバックし、処理を継続する。

| ケース | 対応 | ログレベル | 影響 |
|--------|------|-----------|------|
| 環境変数が存在しない | デフォルト値使用 | DEBUG | なし（正常動作） |
| 絵文字形式が不正 | デフォルト値使用 | WARN | カスタム絵文字が使えないのみ |
| カスタム絵文字がサーバーに存在しない | デフォルト値使用 | WARN | カスタム絵文字が使えないのみ |
| カスタム絵文字名が見つからない | デフォルト値使用 | WARN | カスタム絵文字が使えないのみ |
| サーバー絵文字取得失敗 | 空のHashMapで続行 | WARN | 名前形式の変換ができないのみ |
| DB接続エラー | エラーを上位層へ伝播 | - | 募集作成全体が失敗 |

### ログ設計

#### 成功時

```
DEBUG guild_id=123456789 "属性絵文字設定を取得します"
DEBUG guild_id=123456789 emoji_count=50 "サーバー絵文字一覧を取得しました"
DEBUG emoji_name="water" "カスタム絵文字名として検索します"
DEBUG emoji_name="water" emoji_id=987654321 resolved="<:water:987654321>" "絵文字名をカスタム絵文字形式に変換しました"
DEBUG guild_id=123456789 custom_count=6 "カスタム属性絵文字を適用しました"
```

#### 失敗時

```
DEBUG emoji_name="water" "カスタム絵文字名として検索します"
DEBUG emoji_name="water" "指定された名前のカスタム絵文字がサーバーに見つかりませんでした"
WARN guild_id=123456789 key="ELEMENT_WATER" value=":water:" "絵文字が使用できないためデフォルト値を使用します（形式不正またはサーバーに存在しない）"
```

## データベース設計

### guild_environments テーブル

環境変数の保存に既存の `guild_environments` テーブルを使用。

**スキーマ**:
```sql
CREATE TABLE guild_master.guild_environments (
    guild_id BIGINT NOT NULL,
    key VARCHAR NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY (guild_id, key)
);
```

**設定例**:
```sql
INSERT INTO guild_master.guild_environments (guild_id, key, value, created_at, updated_at)
VALUES
  (123456789, 'ELEMENT_FIRE', ':fire:', NOW(), NOW()),
  (123456789, 'ELEMENT_WATER', ':water:', NOW(), NOW()),
  (123456789, 'ELEMENT_EARTH', ':earth:', NOW(), NOW()),
  (123456789, 'ELEMENT_WIND', ':wind:', NOW(), NOW()),
  (123456789, 'ELEMENT_LIGHT', ':light:', NOW(), NOW()),
  (123456789, 'ELEMENT_DARK', ':dark:', NOW(), NOW())
ON CONFLICT (guild_id, key)
DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();
```

## パフォーマンス考慮事項

### N+1問題の回避

**問題**: 6属性を個別に取得すると6回のクエリが発生

**解決**: `get_multiple_by_guild()` で1クエリに集約

```sql
-- 最適化前（6クエリ）
SELECT * FROM guild_environments WHERE guild_id = $1 AND key = 'ELEMENT_FIRE';
SELECT * FROM guild_environments WHERE guild_id = $1 AND key = 'ELEMENT_WATER';
-- ... (4回繰り返し)

-- 最適化後（1クエリ）
SELECT * FROM guild_environments
WHERE guild_id = $1
  AND key IN ('ELEMENT_FIRE', 'ELEMENT_WATER', 'ELEMENT_EARTH', 'ELEMENT_WIND', 'ELEMENT_LIGHT', 'ELEMENT_DARK');
```

### Discord API呼び出しの最小化

サーバー絵文字一覧の取得は募集作成ごとに1回のみ。

```rust
// 1回だけ取得
let guild_emojis = Self::fetch_guild_emojis(http, guild_id).await;

// 6属性すべてで再利用
for key in ELEMENT_KEYS {
    Self::resolve_emoji(value, &guild_emojis);  // 同じHashMapを使用
}
```

### キャッシュ不要の判断

以下の理由により、現時点ではキャッシュを実装していない：

1. **募集作成頻度が低い**: 高負荷にはならない
2. **読み取り専用で軽量**: 6レコード程度
3. **リアルタイム性が重要**: 設定変更の即時反映が望ましい

将来的にパフォーマンス問題が発生した場合、AppStateにTTL付きキャッシュを追加可能。

## セキュリティ考慮事項

### インジェクション対策

- SeaORMのパラメータバインディングを使用
- ユーザー入力（絵文字名）は検証のみで直接SQLに含まない

### 権限管理

- 環境変数の設定は管理者権限が必要（実装は別機能）
- 読み取りは全ユーザーが可能（募集作成時に自動取得）

## 制約事項

### マルチ募集v1（リアクション）での制約

リアクションとして使用するため、以下の形式のみが有効：

- ✅ 有効: `🔥` `<:fire:123456789>` `<a:fire:123456789>`
- ❌ 無効: `:fire:` （文字列として表示され、リアクション不可）

この制約により、`:fire:` のような形式は自動的に `<:fire:123456789>` に変換する機能を実装。

### 大文字小文字の区別

カスタム絵文字名の検索では大文字小文字を区別する：

- `ELEMENT_FIRE=:Fire:` → `:Fire:` という名前の絵文字を検索
- `ELEMENT_FIRE=:fire:` → `:fire:` という名前の絵文字を検索

サーバーに登録されている絵文字名と完全に一致する必要がある。

### サーバー間の互換性

同じ環境変数設定でも、サーバーごとに動作が異なる：

**例**: `ELEMENT_FIRE=:fire:`

- **サーバーA** (`:fire:` というカスタム絵文字が存在)
  - `<:fire:123456789>` に自動変換して使用

- **サーバーB** (`:fire:` というカスタム絵文字が存在しない)
  - デフォルト絵文字（🔥）にフォールバック

## テスト戦略

### Repository層テスト

- ✅ カスタム値の正常取得
- ✅ 環境変数未設定時の空HashMap返却
- ✅ トランザクション内でのUpsert

### Service層テスト（mockallでモッキング）

- ✅ 全カスタム値の適用
- ✅ 部分的なカスタム値（残りはデフォルト）
- ✅ 不正な絵文字のフォールバック
- ✅ 環境変数未設定時の全デフォルト値
- ✅ `:emoji_name:` 形式の自動変換

### 統合テスト

- ✅ カスタム絵文字での募集作成（リアクション版・ボタン版v2）
- ✅ 参加者一覧更新時のカスタム絵文字適用
- ✅ サーバー絵文字が存在しない場合のフォールバック

## 運用上の注意事項

### 設定変更の即時反映

環境変数の変更は次回の募集作成時から反映される（キャッシュなし）。

### 絵文字の削除

サーバーからカスタム絵文字を削除した場合：
1. 次回の募集作成時にWARNログが出力される
2. 自動的にデフォルト絵文字にフォールバックする
3. 既存の募集には影響しない（メッセージは変更されない）

### デバッグ方法

DEBUGレベルのログを有効にすると、絵文字解決の詳細が確認できる：

```bash
RUST_LOG=debug cargo run
```

```
DEBUG guild_environment_service: emoji_name="water" "カスタム絵文字名として検索します"
DEBUG guild_environment_service: emoji_name="water" emoji_id=987654321 resolved="<:water:987654321>" "絵文字名をカスタム絵文字形式に変換しました"
```

## 今後の拡張可能性

### キャッシュの追加

パフォーマンス問題が発生した場合、AppStateにTTL付きキャッシュを追加可能：

```rust
pub struct AppState {
    // ...
    emoji_cache: Arc<Mutex<HashMap<i64, (ElementEmojis, SystemTime)>>>,
}
```

### 他の要素へのカスタマイズ拡張

同じ仕組みを使って、以下の要素もカスタマイズ可能：

- 募集メッセージのテンプレート
- ボタンのラベル
- 通知メッセージの内容

### 管理コマンドの追加

環境変数を設定するための Discord スラッシュコマンドを追加可能：

```
/guild_env set ELEMENT_FIRE :fire:
/guild_env list
/guild_env delete ELEMENT_FIRE
```

## 関連ドキュメント

- [クリーンアーキテクチャ設計](../architecture/rust_optimized_architecture.md)
- [データベース設計](../database/db_connection_transaction.md)
- [エラーハンドリング](../error_types.md)
- [依存性注入](../../rules/dependency_injection.md)
