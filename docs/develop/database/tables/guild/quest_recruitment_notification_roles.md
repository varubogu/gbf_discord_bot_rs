# クエスト別募集通知ロール（quest_recruitment_notification_roles）

## 概要

**テーブル物理名**: `quest_recruitment_notification_roles`
**スキーマ名**: `guild_master`
**テーブルタイプ**: Reference
**テーブルスコープ**: Guild（ギルド固有のマスターデータ）
**実装状況**: ✅ 実装済み

## 用途

特定のクエストに対するマルチバトル募集時のみ通知を送信するDiscordロールを管理します。このテーブルに登録されたロールは、対応するクエストの募集メッセージでのみメンションされます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| guild_id | BIGINT | PK, NOT NULL | ギルドID（Discord Guild ID） |
| quest_id | INTEGER | PK, NOT NULL, FK | クエストID |
| seq | SERIAL | PK, NOT NULL | 表示順序用の連番（SEQUENCE使用） |
| role_id | BIGINT | NOT NULL, UNIQUE(guild_id, quest_id, role_id) | ロールID（Discord Role ID） |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `guild_id`, `quest_id`, `seq`（三重複合キー）

### 外部キー
- `quest_id` → `master.quests(id)` - クエストマスタテーブルを参照

### UNIQUE制約
- UNIQUE(`guild_id`, `quest_id`, `role_id`) - 同じギルド内で同じクエストに同じロールを重複登録できない

### NOT NULL制約
- `guild_id`, `quest_id`, `seq`, `role_id`, `created_at`, `updated_at`

## インデックス

- **プライマリキーインデックス**: `(guild_id, quest_id, seq)`（自動作成）
- **UNIQUEインデックス**: `(guild_id, quest_id, role_id)`（自動作成）
- **外部キーインデックス**: `quest_id`（外部キー制約で自動作成）

## データサンプル

| guild_id | quest_id | seq | role_id | created_at | updated_at |
|----------|----------|-----|---------|------------|------------|
| 123456789 | 1 | 1 | 987654321 | 2025-10-24 10:00:00+00 | 2025-10-24 10:00:00+00 |
| 123456789 | 1 | 2 | 987654322 | 2025-10-24 10:05:00+00 | 2025-10-24 10:05:00+00 |
| 123456789 | 2 | 1 | 987654323 | 2025-10-24 10:10:00+00 | 2025-10-24 10:10:00+00 |
| 987654321 | 1 | 1 | 123456789 | 2025-10-24 11:00:00+00 | 2025-10-24 11:00:00+00 |

## 関連テーブル

### 参照先テーブル

- **master.quests**: `quest_id` で参照（多対1）

### 論理関連テーブル

- **all_recruitment_notification_roles**: 全募集通知ロールと併用
- **worker.battle_recruitments**: 募集メッセージ作成時にロールメンションを生成

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/guild_master/quest_recruitment_notification_roles.rs`
- **Repository**: `src/repository/database/recruitment/quest_recruitment_notification_roles_repository.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "quest_recruitment_notification_roles", schema_name = "guild_master")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub quest_id: i32,
    #[sea_orm(primary_key)]
    pub seq: i32,
    pub role_id: i64,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::master::quests::Entity",
        from = "Column::QuestId",
        to = "super::super::master::quests::Column::Id"
    )]
    Quests,
}
```

## 備考

- **自動採番**: `seq`はPostgreSQLのSEQUENCEにより自動採番され、登録順序を保証する（後から追加されたロールは後に表示）
- 同じギルド内で同じクエストに同じロールを重複登録することはできない（UNIQUE制約）
- Discord上でロールが削除された場合でも、このテーブルからは自動削除されない（手動削除が必要）
- `quest_id`は外部キー制約により`master.quests.id`を参照し、存在しないクエストIDは登録不可
- クエストが削除された場合の動作: 外部キー制約により、クエスト削除時に関連ロール設定も削除される可能性（マイグレーション設定による）
- Row Level Security（RLS）対象テーブル（`guild_id`による行レベルセキュリティ）

### 募集メッセージでの使用

- 募集メッセージ作成時、このテーブルと`all_recruitment_notification_roles`テーブルを結合して通知対象ロールを取得
- **メンション順序**: 全募集通知ロール（seq昇順） → クエスト別通知ロール（seq昇順）
- **メンション形式**: `<@&role_id>`（Discord標準形式）

### 管理コマンド

- `/recruit_role_add`: ロールを追加
- `/recruit_role_remove`: ロールを削除
- `/recruit_role_show`: 登録済みロールを表示
- **権限**: 設定変更は`gbf_bot_control`ロール保持者のみ実行可能
