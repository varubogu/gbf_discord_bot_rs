# スプレッドシート テーブル固有処理の分離 設計書

## 概要

現在、`src/services/spreadsheet/schema_extractor_service.rs`にハードコードされているテーブル固有処理を、
`src/services/spreadsheet/tables/`配下の個別ファイルに分離し、保守性と拡張性を向上させる。

## 現在の問題点

1. **SchemaExtractorServiceのハードコード**
   - 新しいテーブル追加時に2箇所の手動修正が必要
   - テーブル一覧が一元管理されていない
   - 別名対応（messages/message_texts等）が散在

2. **テーブル固有情報の散逸**
   - 列マッピング情報の定義場所が不明確
   - 読み込み専用列・書き込み専用列の区別がない
   - エンティティとスプレッドシートの対応関係が暗黙的

## 設計方針

### 1. ディレクトリ構造

```
src/services/spreadsheet/
├── tables/                          # 新規作成
│   ├── mod.rs                       # TableConfigトレイト定義、全テーブル登録
│   ├── battle_types.rs              # battle_typesテーブル固有処理
│   ├── environments.rs              # environmentsテーブル固有処理
│   ├── event_schedules.rs
│   ├── event_schedule_details.rs
│   ├── message_texts.rs
│   ├── quests.rs
│   └── quest_aliases.rs
├── schema_extractor_service.rs      # 変更: TableConfigを使用
├── spreadsheet_reader_service.rs    # 変更なし（共通処理）
├── spreadsheet_writer_service.rs    # 変更なし（共通処理）
├── data_converter_service.rs        # 変更なし（共通処理）
└── ...
```

### 2. TableConfigトレイト設計

各テーブルファイルで実装するトレイト：

```rust
/// テーブル固有設定トレイト
pub trait TableConfig: Send + Sync {
    /// エンティティ型（型パラメータで指定）
    type Entity: EntityTrait;

    /// テーブル名（データベース側）
    fn table_name() -> &'static str;

    /// スプレッドシート上の別名一覧（オプション）
    fn table_aliases() -> Vec<&'static str> {
        vec![]
    }

    /// 読み込み対象列の取得（自動抽出）
    fn read_columns(&self) -> Vec<ColumnSchema> {
        extract_entity_schema::<Self::Entity>(Self::table_name())
    }

    /// 書き込み対象列の取得（自動抽出、デフォルトは読み込みと同じ）
    fn write_columns(&self) -> Vec<ColumnSchema> {
        self.read_columns()
    }

    /// 除外列の指定（created_at, updated_at等）
    fn excluded_columns_for_read() -> Vec<&'static str> {
        vec![]
    }

    fn excluded_columns_for_write() -> Vec<&'static str> {
        vec!["created_at", "updated_at"]  // デフォルト除外
    }
}
```

### 3. 個別テーブルファイルの実装例

#### battle_types.rs

```rust
use sea_orm::EntityTrait;
use crate::models::entities;
use crate::services::spreadsheet::{ColumnSchema, PostgresType};
use super::TableConfig;

/// battle_typesテーブル設定
pub struct BattleTypesTable;

impl TableConfig for BattleTypesTable {
    type Entity = entities::battle_types::Entity;

    fn table_name() -> &'static str {
        "battle_types"
    }

    // デフォルト実装で十分な場合は省略可能
}
```

#### message_texts.rs（別名対応の例）

```rust
use sea_orm::EntityTrait;
use crate::models::entities;
use crate::services::spreadsheet::{ColumnSchema, PostgresType};
use super::TableConfig;

/// message_textsテーブル設定
pub struct MessageTextsTable;

impl TableConfig for MessageTextsTable {
    type Entity = entities::message_texts::Entity;

    fn table_name() -> &'static str {
        "message_texts"
    }

    /// スプレッドシート側で"messages"という別名を許可
    fn table_aliases() -> Vec<&'static str> {
        vec!["messages"]
    }
}
```

#### quests.rs（列除外の例）

```rust
use sea_orm::EntityTrait;
use crate::models::entities;
use crate::services::spreadsheet::{ColumnSchema, PostgresType};
use super::TableConfig;

/// questsテーブル設定
pub struct QuestsTable;

impl TableConfig for QuestsTable {
    type Entity = entities::quests::Entity;

    fn table_name() -> &'static str {
        "quests"
    }

    /// 読み込み時に除外する列（自動生成される列など）
    fn excluded_columns_for_read() -> Vec<&'static str> {
        vec!["id", "created_at", "updated_at"]
    }
}
```

### 4. tables/mod.rs でのテーブル登録

```rust
use std::collections::HashMap;
use sea_orm::EntityTrait;

mod battle_types;
mod environments;
mod event_schedules;
mod event_schedule_details;
mod message_texts;
mod quests;
mod quest_aliases;

pub use battle_types::BattleTypesTable;
pub use environments::EnvironmentsTable;
pub use event_schedules::EventSchedulesTable;
pub use event_schedule_details::EventScheduleDetailsTable;
pub use message_texts::MessageTextsTable;
pub use quests::QuestsTable;
pub use quest_aliases::QuestAliasesTable;

/// TableConfigトレイト定義
pub trait TableConfig: Send + Sync {
    type Entity: EntityTrait;
    fn table_name() -> &'static str;
    fn table_aliases() -> Vec<&'static str> { vec![] }
    fn read_columns(&self) -> Vec<ColumnSchema>;
    fn write_columns(&self) -> Vec<ColumnSchema>;
    fn excluded_columns_for_read() -> Vec<&'static str> { vec![] }
    fn excluded_columns_for_write() -> Vec<&'static str> { vec!["created_at", "updated_at"] }
}

/// 全テーブル設定を登録
pub fn register_all_tables() -> HashMap<String, Box<dyn TableConfig>> {
    let mut tables: HashMap<String, Box<dyn TableConfig>> = HashMap::new();

    // 各テーブルを登録
    register_table::<BattleTypesTable>(&mut tables);
    register_table::<EnvironmentsTable>(&mut tables);
    register_table::<EventSchedulesTable>(&mut tables);
    register_table::<EventScheduleDetailsTable>(&mut tables);
    register_table::<MessageTextsTable>(&mut tables);
    register_table::<QuestsTable>(&mut tables);
    register_table::<QuestAliasesTable>(&mut tables);

    tables
}

/// テーブル登録ヘルパー（本名と別名の両方を登録）
fn register_table<T: TableConfig + Default + 'static>(
    tables: &mut HashMap<String, Box<dyn TableConfig>>
) {
    let table = Box::new(T::default());

    // 本名で登録
    tables.insert(T::table_name().to_string(), table);

    // 別名で登録
    for alias in T::table_aliases() {
        tables.insert(alias.to_string(), Box::new(T::default()));
    }
}

/// テーブル名（または別名）からTableConfigを取得
pub fn get_table_config(table_name: &str) -> Option<Box<dyn TableConfig>> {
    let all_tables = register_all_tables();
    all_tables.get(table_name).cloned()
}
```

### 5. SchemaExtractorServiceのリファクタリング

```rust
use crate::services::spreadsheet::tables;

impl SchemaExtractorService {
    /// 全テーブルのスキーマをHashMapで取得
    pub fn extract_all_schemas(&self) -> HashMap<String, Vec<ColumnSchema>> {
        let all_tables = tables::register_all_tables();

        all_tables.into_iter()
            .map(|(table_name, config)| {
                (table_name, config.read_columns())
            })
            .collect()
    }

    /// 特定のテーブルのスキーマを取得
    pub fn extract_schema(&self, table_name: &str) -> Option<Vec<ColumnSchema>> {
        tables::get_table_config(table_name)
            .map(|config| config.read_columns())
    }
}
```

**変更点:**
- ハードコードされたmatch文を完全削除
- `tables::register_all_tables()`を使用した動的登録
- 新しいテーブル追加時は`tables/xxx.rs`を作成し、`tables/mod.rs`に1行追加するだけ

### 6. メリット

1. **保守性向上**
   - テーブル固有処理が1ファイルに集約
   - 新規テーブル追加時の変更箇所が明確（tables/mod.rsの1箇所のみ）
   - 別名対応がテーブル定義内に含まれる

2. **拡張性向上**
   - 列の読み書き分離が容易
   - 除外列の指定が簡単
   - テーブル固有のバリデーションルール追加が可能

3. **型安全性**
   - EntityTraitの型パラメータで静的チェック
   - コンパイル時にエンティティとテーブル設定の対応を保証

4. **テスト容易性**
   - 各テーブル設定を個別にテスト可能
   - モックの作成が容易

### 7. マイグレーション手順

#### Phase 1: 基盤構築
1. `src/services/spreadsheet/tables/mod.rs`作成
2. `TableConfig`トレイト定義
3. `register_all_tables()`実装

#### Phase 2: 個別テーブル実装
4. `tables/battle_types.rs`作成
5. `tables/environments.rs`作成
6. ... (全7テーブル)

#### Phase 3: 統合
7. `schema_extractor_service.rs`をリファクタリング
8. テスト実行・修正
9. 既存の動作確認

#### Phase 4: クリーンアップ
10. 未使用コードの削除
11. ドキュメント更新

### 8. 互換性維持

- **既存のAPIは変更なし**
  - `extract_all_schemas()` シグネチャ変更なし
  - `extract_schema(table_name)` シグネチャ変更なし
- **Facade層の変更不要**
  - `SpreadsheetImportFacade`はそのまま使用可能
- **既存のテストは全て通過するはず**

### 9. 今後の拡張ポイント

#### カスタムバリデーション
```rust
impl TableConfig for QuestsTable {
    fn validate_row(&self, row: &RowData) -> Result<(), ValidationError> {
        // テーブル固有のバリデーション
        // 例: recruit_countは1以上30以下
        Ok(())
    }
}
```

#### カラムレベルのカスタマイズ
```rust
impl TableConfig for BattleTypesTable {
    fn read_columns(&self) -> Vec<ColumnSchema> {
        let mut cols = extract_entity_schema::<Self::Entity>(Self::table_name());

        // emojiカラムはオプショナルにする
        for col in &mut cols {
            if col.column_name == "emoji" {
                col.nullable = true;
            }
        }

        cols
    }
}
```

#### スプレッドシート列名のカスタマイズ
```rust
pub trait TableConfig {
    /// スプレッドシート上の列名マッピング（DB列名 → スプレッドシート列名）
    fn column_name_mapping() -> HashMap<&'static str, &'static str> {
        HashMap::new()  // デフォルトは同一名
    }
}
```

## 実装スケジュール

| Phase | タスク | 見積もり |
|-------|--------|----------|
| 1 | 基盤構築（mod.rs、トレイト定義） | 30分 |
| 2 | 個別テーブル実装（7ファイル） | 1時間 |
| 3 | SchemaExtractorService統合 | 30分 |
| 4 | テスト・動作確認 | 30分 |
| 5 | クリーンアップ・ドキュメント | 20分 |
| **合計** | | **約3時間** |

## リスク管理

### リスク
1. トレイトオブジェクトの制約によるコンパイルエラー
2. 既存テストの破損

### 対策
1. 最小限のテーブル（1つ）で先行実装して動作確認
2. 各Phase後にテスト実行して早期検出

## 参考資料

- [Rustのトレイトオブジェクト](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
- [SeaORMのEntityTrait](https://docs.rs/sea-orm/latest/sea_orm/entity/trait.EntityTrait.html)
- クリーンアーキテクチャ原則（CLAUDE.md参照）
