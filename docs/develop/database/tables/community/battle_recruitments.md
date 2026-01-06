# マルチバトル募集情報（battle_recruitments）

## 概要

**テーブル物理名**: `battle_recruitments`
**スキーマ名**: `worker`
**テーブルタイプ**: Transaction
**テーブルスコープ**: Community（ギルド内のユーザー活動データ）
**実装状況**: ✅ 実装済み

## 用途

ユーザーが作成したマルチバトル募集を管理します。Discord上のメッセージと1対1で紐づき、募集状態、クエスト情報、出発時刻などを保持します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| id | SERIAL | PK, NOT NULL | 募集ID（主キー、自動採番） |
| guild_id | BIGINT | NOT NULL, FK | ギルドID（Discord Guild ID） |
| channel_id | BIGINT | NOT NULL | チャンネルID（Discord Channel ID） |
| message_id | BIGINT | NOT NULL | メッセージID（Discord Message ID） |
| quest_id | INTEGER | NOT NULL, FK | 対象クエストID |
| battle_style_id | INTEGER | NOT NULL, FK | バトル戦術ID |
| quest_start_at | TIMESTAMPTZ | NOT NULL | クエスト出発時刻（UTC） |
| is_recruiting | BOOLEAN | NOT NULL, DEFAULT true | 募集中フラグ |
| is_canceled | BOOLEAN | NOT NULL, DEFAULT false | キャンセルフラグ |
| recruit_end_message_id | BIGINT | NULLABLE | 募集終了メッセージID |
| full_notification_sent | BOOLEAN | NOT NULL, DEFAULT false | 満員通知送信済みフラグ |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `id`

### 外部キー
- `quest_id` → `master.quests(id)`
- `battle_style_id` → `master.battle_styles(id)`
- `guild_id` → `guild_master.guilds(guild_id)`

### UNIQUE制約
なし（以前の設計では guild_id, channel_id, message_id の複合UNIQUEがあったが、実装では削除）

### NOT NULL制約
- `id`, `guild_id`, `channel_id`, `message_id`, `quest_id`, `battle_style_id`, `quest_start_at`, `is_recruiting`, `is_canceled`, `full_notification_sent`, `created_at`, `updated_at`

## インデックス

- **プライマリキーインデックス**: `id`（自動作成）
- **外部キーインデックス**: `quest_id`, `battle_style_id`, `guild_id`（外部キー制約で自動作成）

## データサンプル

| id | guild_id | channel_id | message_id | quest_id | battle_style_id | quest_start_at | is_recruiting | is_canceled |
|----|----------|-----------|-----------|----------|----------------|---------------|--------------|------------|
| 1 | 123456789 | 987654321 | 1234567890123456 | 1 | 1 | 2025-10-24 12:00:00+00 | true | false |
| 2 | 123456789 | 987654321 | 2345678901234567 | 2 | 2 | 2025-10-24 15:00:00+00 | false | false |

## 関連テーブル

### 参照先テーブル

- **master.quests**: `quest_id` で参照（多対1）
- **master.battle_styles**: `battle_style_id` で参照（多対1）
- **guild_master.guilds**: `guild_id` で参照（多対1）

### 参照元テーブル

- **worker.recruitment_participants**: 参加者情報（1対多）
- **worker.notification_rel_battle_recruitments**: 通知との関連（1対多）
- **worker.scheduled_task_dismissals**: 解散タスク（1対多）
- **worker.scheduled_task_dissolutions**: 強制解散タスク（1対多）

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/worker/battle_recruitments.rs`
- **マイグレーションファイル**: `migration/src/m*_create_battle_recruitments.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "battle_recruitments", schema_name = "worker")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_id: i64,
    pub quest_id: i32,
    pub battle_style_id: i32,
    pub quest_start_at: DateTimeUtc,
    pub is_recruiting: bool,
    pub is_canceled: bool,
    pub recruit_end_message_id: Option<i64>,
    pub full_notification_sent: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

## 備考

- **設計変更**: 以前の設計から大幅に変更されました:
  - 主キー: UUID (`row_id`) → SERIAL (`id`)
  - 削除されたカラム: `room_id`, `expiry_date`
  - 追加されたカラム: `quest_start_at`, `is_recruiting`, `is_canceled`, `full_notification_sent`
  - UNIQUE制約の削除: `(guild_id, channel_id, message_id)` の複合UNIQUE制約は実装では存在しない

- **募集状態管理**:
  - `is_recruiting`: 募集中かどうか（true=募集中、false=締め切り）
  - `is_canceled`: キャンセルされたかどうか
  - `full_notification_sent`: 満員通知を送信済みかどうか

- **Discord メッセージとの紐づけ**:
  - `guild_id`, `channel_id`, `message_id` の組み合わせでDiscordメッセージを特定
  - メッセージ削除時に自動的に募集がキャンセルされる仕組みあり

- **有効期限管理**:
  - 以前の設計にあった `expiry_date` は削除され、代わりに `quest_start_at`（クエスト出発時刻）で管理
  - クエスト出発時刻を過ぎた募集は、scheduled_task_dissolutions（強制解散タスク）により自動的にクローズ

- **参加者管理**:
  - 別テーブル `worker.recruitment_participants` で参加者情報を管理
  - Discordリアクションによる参加/離脱を記録

## データクリーンアップ

古い募集データは定期的にクリーンアップされます。詳細は [data_cleanup_system.md](../../design/features/data_cleanup_system.md) を参照してください。
