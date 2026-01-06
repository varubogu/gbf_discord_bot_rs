# 全募集通知ロール（all_recruitment_notification_roles）

## 概要

**テーブル物理名**: `all_recruitment_notification_roles`
**スキーマ名**: `guild_master`
**テーブルタイプ**: Reference
**テーブルスコープ**: Guild（ギルド固有のマスターデータ）
**実装状況**: ✅ 実装済み

## 用途

全てのマルチバトル募集に対して通知を送信するDiscordロールを管理します。このテーブルに登録されたロールは、クエストの種類に関係なく、全ての募集メッセージでメンションされます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| guild_id | BIGINT | PK, NOT NULL | ギルドID（Discord Guild ID） |
| seq | SERIAL | PK, NOT NULL | 表示順序用の連番（SEQUENCE使用） |
| role_id | BIGINT | NOT NULL, UNIQUE(guild_id, role_id) | ロールID（Discord Role ID） |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `guild_id`, `seq`（複合キー）

### 外部キー
なし（Discord上のロールIDは外部キー制約不可）

### UNIQUE制約
- UNIQUE(`guild_id`, `role_id`) - 同じギルド内で同じロールを重複登録できない

## インデックス

- PK: `guild_id`, `seq`（自動作成）
- UNIQUE: `guild_id`, `role_id`（自動作成）
- 推奨追加インデックス: `guild_id`（検索性能向上）

## データサンプル

| guild_id | seq | role_id | created_at | updated_at |
|----------|-----|---------|------------|------------|
| 123456789 | 1 | 987654321 | 2025-10-24 10:00:00 | 2025-10-24 10:00:00 |
| 123456789 | 2 | 987654322 | 2025-10-24 10:05:00 | 2025-10-24 10:05:00 |
| 987654321 | 1 | 123456789 | 2025-10-24 11:00:00 | 2025-10-24 11:00:00 |

## 関連テーブル

- **論理関連**: `quest_recruitment_notification_roles`（クエスト別通知ロールと併用）
- **論理関連**: `battle_recruitments`（募集メッセージ作成時にロールメンションを生成）

## 備考

- **自動採番**: `seq`はPostgreSQLのSEQUENCEにより自動採番され、登録順序を保証する（後から追加されたロールは後に表示）
- 同じギルド内で同じロールを重複登録することはできない（UNIQUE制約）
- Discord上でロールが削除された場合でも、このテーブルからは自動削除されない（手動削除が必要）
- Row Level Security（RLS）対象テーブル（`guild_id`による行レベルセキュリティ）
- 募集メッセージ作成時、このテーブルと`quest_recruitment_notification_roles`テーブルを結合して通知対象ロールを取得
- メンション形式: `<@&role_id>`（Discord標準形式）
- 管理コマンド`/recruit_role_add`、`/recruit_role_remove`で設定変更可能
- 設定変更は`gbf_bot_control`ロール保持者のみ実行可能

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/guild_master/all_recruitment_notification_roles.rs`
- **Repository**: `src/repository/database/recruitment/all_recruitment_notification_roles_repository.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "all_recruitment_notification_roles", schema_name = "guild_master")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    #[sea_orm(primary_key)]
    pub seq: i32,
    pub role_id: i64,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```
