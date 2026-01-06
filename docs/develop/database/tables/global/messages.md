# メッセージ定義（message_texts）

## 概要

**テーブル物理名**: `message_texts`
**スキーマ名**: `master`
**テーブルタイプ**: Reference
**テーブルスコープ**: All（全ギルド共通）
**実装状況**: ✅ 実装済み

## 用途

Bot応答メッセージのテンプレートを定義します。統一的なメッセージ管理と多言語対応を実現します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| id | TEXT | PK, NOT NULL | メッセージ定義ID（例: RECRUITMENT_START、EVENT_REMINDER） |
| message_jp | TEXT | NOT NULL | 日本語のメッセージテンプレート |
| message_en | TEXT | NULLABLE | 英語のメッセージテンプレート（多言語対応） |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `id`

### 外部キー
なし

### UNIQUE制約
なし

### NOT NULL制約
- `id`, `message_jp`, `created_at`, `updated_at`

## インデックス

- **プライマリキーインデックス**: `id`（自動作成）

## データサンプル

| id | message_jp | message_en |
|----|-----------|-----------|
| RECRUITMENT_START | マルチバトル募集を開始しました | Multi-battle recruitment started |
| RECRUITMENT_FULL | 募集が満員になりました | Recruitment is full |
| EVENT_REMINDER | イベント終了まであと1時間です | Event ends in 1 hour |

## 関連テーブル

### 参照元テーブル

- **master.event_schedule_details**: `message_text_id` で参照（1対多）

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/master/message_texts.rs`
- **マイグレーションファイル**: `migration/src/m*_create_message_texts.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "master", table_name = "message_texts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub message_jp: String,
    pub message_en: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

## 備考

- **テーブル名変更**: 以前の設計では `messages` という名称でしたが、実装では `message_texts` に変更されました
- **カラム名変更**: `message_id` → `id` に変更
- **カラム削除**: `reactions`, `memo` カラムは実装に存在しないため削除されました
- **カラム追加**: `message_en` が追加され、多言語対応が強化されました
- グローバルメッセージテンプレートとして全ギルドで使用
- 多言語対応のため日本語と英語の両方を保持可能
