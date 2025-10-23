# イベント詳細スケジュール（event_schedule_details）

## 概要

**テーブル物理名**: `event_schedule_details`
**テーブルタイプ**: Reference
**テーブルスコープ**: All

## 用途

イベント期間内の詳細スケジュール（デイリーミッション、ボーダー更新など）を定義します。相対日時とprofileでイベントスケジュールと紐づけ、具体的な通知日時に展開されます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| row_id | UUID | PK, NOT NULL, default=uuid_v4 | 行ID（プライマリキー） |
| profile | String | NOT NULL | イベントスケジュールとの紐づけプロファイル |
| start_day_relative | String | NOT NULL | 開始日からの相対日（例: "1", "1-5", "final"） |
| time | String | NOT NULL | イベント時間（例: "23:59", "05:00"） |
| schedule_name | String | NOT NULL | スケジュール名（例: デイリーミッション、ボーダー更新） |
| message_id | String | NULLABLE, FK(messages.message_id) | 通知メッセージID |
| guild_id | BigInteger | NULLABLE | 対象ギルドID（グローバルの場合NULL） |
| channel_id | BigInteger | NULLABLE | 通知先チャンネルID |
| reactions | String | NULLABLE | 通知メッセージに付与するリアクション |

## 制約

### プライマリキー
- `row_id`

### 外部キー
- `message_id` → `messages(message_id)`

### UNIQUE制約
なし

## インデックス

- PK: `row_id`（自動作成）
- FK: `message_id`（外部キー制約で自動作成）

## データサンプル

| row_id | profile | start_day_relative | time | schedule_name | message_id | guild_id | channel_id | reactions |
|--------|---------|-------------------|------|--------------|-----------|----------|-----------|-----------|
| uuid-1 | unite_fight | 1 | 05:00 | デイリーミッション | DAILY_MISSION | NULL | NULL | ✅ |
| uuid-2 | unite_fight | 1-5 | 23:59 | ボーダー更新 | BORDER_UPDATE | NULL | NULL | 📊 |
| uuid-3 | xeno_clash | 1 | 00:00 | イベント開始 | EVENT_START | NULL | NULL | 🎉 |

## 関連テーブル

- **参照元**: `schedules`（parent_schedule_detail_idで参照）
- **参照先**: `messages`（message_idで参照）
- **関連**: `event_schedules`（profileで論理的に紐づけ）
- **関連**: `guild_event_schedule_details`（ギルド固有の詳細スケジュール）

## 備考

- row_idはUUID v4で自動生成
- profileでevent_schedulesと紐づけ
- start_day_relativeは相対日を示し、"1"は初日、"1-5"は1日目から5日目まで、"final"は最終日
- timeは時刻を示し、"HH:MM"形式
- guild_idがNULLの場合は全ギルド共通、値がある場合は特定ギルド専用
- schedulesテーブルに展開される際に具体的な日時に変換される

## Rust実装

- **エンティティ**: `src/models/entities/event_schedule_details.rs`
- **実装状況**: 実装済み
