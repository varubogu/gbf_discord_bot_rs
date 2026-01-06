# 定期募集スケジュール（battle_recruitment_schedules）

## 概要

**テーブル物理名**: `battle_recruitment_schedules`
**スキーマ名**: `guild_master`
**テーブルタイプ**: Reference
**テーブルスコープ**: Guild（ギルド固有）
**実装状況**: ✅ 実装済み

## 用途

定期的に自動作成されるマルチバトル募集のスケジュールを管理します。毎週特定の曜日・時刻に自動的に募集メッセージを作成する機能で、ギルド単位で設定可能です。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| id | SERIAL | PK, NOT NULL | 定期募集ID（主キー、自動採番） |
| name | TEXT | NOT NULL | 定期募集の名前（例: 毎週火曜HL、日曜日課） |
| guild_id | BIGINT | NOT NULL, FK | ギルドID（Discord Guild ID） |
| channel_id | BIGINT | NOT NULL | 募集メッセージを投稿するチャンネルID |
| quest_id | INTEGER | NOT NULL, FK | 対象クエストID（quests.idを参照） |
| battle_style_id | INTEGER | NOT NULL, FK | バトル戦術ID（battle_styles.idを参照） |
| quest_start_time | TIME | NOT NULL | クエスト出発時刻（時分秒） |
| recruit_start_day_offset | INTEGER | NOT NULL | 募集開始日のオフセット（日数、負の値で開始日より前） |
| recruit_start_time | TIME | NULLABLE | 募集開始時刻（時分秒） |
| max_participants | INTEGER | NULLABLE | 最大参加人数（NULLの場合はクエストのデフォルト値を使用） |
| note | TEXT | NULLABLE | 備考・メモ |
| is_enabled | BOOLEAN | NOT NULL, DEFAULT true | 有効フラグ（false=無効化） |
| created_by | BIGINT | NOT NULL | 作成者のユーザーID（Discord User ID） |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `id`

### 外部キー
- `guild_id` → `guild_master.guilds(guild_id)`
- `quest_id` → `master.quests(id)`
- `battle_style_id` → `master.battle_styles(id)`

### UNIQUE制約
なし

### NOT NULL制約
- `id`, `name`, `guild_id`, `channel_id`, `quest_id`, `battle_style_id`, `quest_start_time`, `recruit_start_day_offset`, `is_enabled`, `created_by`, `created_at`, `updated_at`

## インデックス

- **プライマリキーインデックス**: `id`（自動作成）
- **外部キーインデックス**: `guild_id`, `quest_id`, `battle_style_id`（外部キー制約で自動作成）

## データサンプル

| id | name | guild_id | channel_id | quest_id | battle_style_id | quest_start_time | recruit_start_day_offset | recruit_start_time | is_enabled |
|----|------|----------|-----------|----------|----------------|------------------|------------------------|-------------------|-----------|
| 1 | 毎週火曜HL | 123456789 | 987654321 | 2 | 1 | 21:00:00 | -1 | 20:00:00 | true |
| 2 | 日曜日課 | 123456789 | 987654321 | 1 | 2 | 12:00:00 | 0 | 10:00:00 | true |

## 関連テーブル

### 参照先テーブル

- **guild_master.guilds**: `guild_id` で参照（多対1）
- **master.quests**: `quest_id` で参照（多対1）
- **master.battle_styles**: `battle_style_id` で参照（多対1）

### 参照元テーブル

- **guild_master.battle_recruitment_schedule_days**: 定期募集の曜日設定（1対多）

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/guild_master/battle_recruitment_schedules.rs`
- **マイグレーションファイル**: `migration/src/m*_create_battle_recruitment_schedules.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "guild_master", table_name = "battle_recruitment_schedules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub guild_id: i64,
    pub channel_id: i64,
    pub quest_id: i32,
    pub battle_style_id: i32,
    pub quest_start_time: TimeTime,
    pub recruit_start_day_offset: i32,
    pub recruit_start_time: Option<TimeTime>,
    pub max_participants: Option<i32>,
    pub note: Option<String>,
    pub is_enabled: bool,
    pub created_by: i64,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

## 備考

- **定期募集の仕組み**:
  - `battle_recruitment_schedule_days` テーブルで曜日を設定（例: 毎週火曜日、金曜日）
  - 設定された曜日と時刻に自動的に募集メッセージが作成される
  - `quest_start_time` がクエスト出発時刻、`recruit_start_time` が募集開始時刻

- **募集開始タイミング**:
  - `recruit_start_day_offset` が負の値の場合、クエスト開始日より前に募集開始
  - 例: `recruit_start_day_offset = -1` で `recruit_start_time = 20:00:00` の場合、クエスト開始日の前日20時に募集開始

- **有効/無効の切り替え**:
  - `is_enabled = false` にすることで、削除せずに一時的に無効化できる
  - 無効化された定期募集は、募集メッセージが自動作成されない

- **ギルド固有データ**:
  - スキーマ名は `guild_master` で、ギルド単位で管理される
  - 各ギルドが独自に定期募集スケジュールを設定可能

- **関連コマンド**:
  - `/定期募集作成`: 新しい定期募集スケジュールを作成
  - `/定期募集削除`: 定期募集スケジュールを削除
  - `/定期募集一覧`: 現在設定されている定期募集の一覧を表示

## 参照

定期募集機能の詳細は [quest_recruitment.md](../../features/quest_recruitment.md) を参照してください。
