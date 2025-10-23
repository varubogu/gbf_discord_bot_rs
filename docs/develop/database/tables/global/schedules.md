# 通知スケジュール（schedules）

## 概要

**テーブル物理名**: `schedules`
**テーブルタイプ**: Transaction
**テーブルスコープ**: All

## 用途

実際に通知を送信するスケジュールを管理します。event_schedule_detailsから具体的な日時に展開されたスケジュールで、通知処理の実行対象となります。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| row_id | UUID | PK, NOT NULL, default=uuid_v4 | 行ID（プライマリキー） |
| parent_schedule_id | UUID | NULLABLE, FK(event_schedules.row_id) | 作成元イベントスケジュールID |
| parent_schedule_detail_id | UUID | NULLABLE, FK(event_schedule_details.row_id) | 作成元詳細スケジュールID |
| schedule_datetime | DateTime | NOT NULL | 通知送信日時 |
| guild_id | BigInteger | NOT NULL | 対象ギルドID |
| channel_id | BigInteger | NOT NULL | 通知先チャンネルID |
| message_id | String | NULLABLE, FK(messages.message_id) | 送信メッセージID |

## 制約

### プライマリキー
- `row_id`

### 外部キー
- `parent_schedule_id` → `event_schedules(row_id)`
- `parent_schedule_detail_id` → `event_schedule_details(row_id)`
- `message_id` → `messages(message_id)`

### UNIQUE制約
なし

## インデックス

- PK: `row_id`（自動作成）
- FK: `parent_schedule_id`（外部キー制約で自動作成）
- FK: `parent_schedule_detail_id`（外部キー制約で自動作成）
- FK: `message_id`（外部キー制約で自動作成）
- 推奨追加インデックス: `schedule_datetime`, `guild_id`（検索性能向上）

## データサンプル

| row_id | parent_schedule_id | parent_schedule_detail_id | schedule_datetime | guild_id | channel_id | message_id |
|--------|-------------------|--------------------------|------------------|----------|-----------|-----------|
| uuid-1 | uuid-s1 | uuid-d1 | 2025-10-15 05:00:00 | 123456789 | 987654321 | DAILY_MISSION |
| uuid-2 | uuid-s1 | uuid-d2 | 2025-10-15 23:59:00 | 123456789 | 987654321 | BORDER_UPDATE |

## 関連テーブル

- **参照元**: `battle_recruitment_schedules`（parent_idで参照）
- **参照先**: `event_schedules`（parent_schedule_idで参照）
- **参照先**: `event_schedule_details`（parent_schedule_detail_idで参照）
- **参照先**: `messages`（message_idで参照）

## 備考

- row_idはUUID v4で自動生成
- event_schedule_detailsの相対日時を具体的な日時に展開したもの
- 通知バッチ処理がschedule_datetimeを基準に実行
- parent_schedule_idとparent_schedule_detail_idで元のイベント情報を追跡可能
- 通知処理後も履歴として保持される

## Rust実装

- **エンティティ**: `src/models/entities/notifications.rs`（schedulesから移行）
- **実装状況**: 実装済み（notificationsとして）
