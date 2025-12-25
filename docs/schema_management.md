# スキーマ管理システム

このドキュメントでは、テーブル名とスキーマ名のマッピング管理について説明します。

## 概要

プロジェクトでは、PostgreSQLの複数のスキーマ(master, guild_master, worker)を使用しています。
テーブル名からスキーマ名を取得する必要がある場面で、以下の3つのアプローチを組み合わせて保守性を確保しています。

## 1. 共通モジュール化

スキーマ名取得ロジックは [`src/services/spreadsheet/schema_utils/`](../src/services/spreadsheet/schema_utils/) に集約されています。

### 使用方法

```rust
use crate::services::spreadsheet::{get_schema_name, get_entity_table_ref};

// テーブル名からスキーマ名を取得
let schema = get_schema_name("quests"); // "master"

// TableRef を取得(スキーマ修飾付き)
let table_ref = get_entity_table_ref("quests");
```

## 2. 自動生成

`get_schema_name` 関数の実装は **ビルド時に自動生成** されます。

### 仕組み

1. [`build.rs`](../build.rs) がエンティティファイル(`src/models/entities/*.rs`)を解析
2. `#[sea_orm(schema_name = "...", table_name = "...")]` 属性を抽出
3. マッピング関数を生成し、`$OUT_DIR/generated_schema_utils.rs` に出力
4. [`src/services/spreadsheet/schema_utils/generated.rs`](../src/services/spreadsheet/schema_utils/generated.rs) が生成されたコードを読み込み

### エンティティ追加時の対応

エンティティファイルに `schema_name` と `table_name` を正しく設定するだけで、自動的に `get_schema_name` に反映されます。

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "master", table_name = "new_table")]
pub struct Model {
    // ...
}
```

再ビルド時に自動的に新しいテーブルが認識されます:

```bash
cargo build
```

## 3. 整合性検証ツール

エンティティ定義とスキーマ配置の整合性を検証するlintツールを提供しています。

### 実行方法

```bash
cargo run --bin schema_lint
```

### 検証内容

1. **テーブル名の接頭辞による検証**
   - `notification_rel_*` → `worker` スキーマ
   - `guild_*` → `guild_master` スキーマ (例外: `guild_last_process_times`)
   - `scheduled_task_*` → `worker` スキーマ

2. **出力例**

```
=== スキーマ整合性検証ツール ===

✓ 28個のエンティティを検出しました

=== スキーマ別テーブル一覧 ===

📁 master スキーマ (9 テーブル):
   - battle_styles
   - channel_types
   ...

=== 整合性チェック ===

✓ 整合性チェック完了: 問題は見つかりませんでした
```

### CI統合

以下のコマンドをCIパイプラインに追加することを推奨します:

```bash
# スキーマ整合性チェック
cargo run --bin schema_lint
```

## スキーマ設計ルール

### master スキーマ
- グローバルなマスタデータ
- 例: quests, battle_styles, elements

### guild_master スキーマ
- ギルドごとの設定・マスタデータ
- 例: guilds, guild_channels, guild_settings
- 例外: `guild_last_process_times` はワーカー状態管理のため worker スキーマ

### worker スキーマ
- 実行時データ・ワーカー状態
- 例: battle_recruitments, notifications, scheduled_tasks

## トラブルシューティング

### ビルドエラー: スキーマ情報が見つからない

エンティティファイルに `schema_name` 属性が設定されているか確認してください:

```rust
#[sea_orm(schema_name = "master", table_name = "your_table")]
```

### lint エラー: スキーマが不正

1. エンティティの `schema_name` を確認
2. 設計上正しい場合は、[`tools/schema_lint.rs`](../tools/schema_lint.rs) に例外を追加

## 参考

- エンティティ定義: [`src/models/entities/`](../src/models/entities/)
- スキーマユーティリティ: [`src/services/spreadsheet/schema_utils/`](../src/services/spreadsheet/schema_utils/)
- ビルドスクリプト: [`build.rs`](../build.rs)
- Lintツール: [`tools/schema_lint.rs`](../tools/schema_lint.rs)
