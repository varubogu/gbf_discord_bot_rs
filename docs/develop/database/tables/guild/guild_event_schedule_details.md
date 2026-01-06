# ギルドイベント詳細スケジュール（guild_event_schedule_details）

## 概要

**テーブル物理名**: `guild_event_schedule_details`
**テーブルタイプ**: Reference
**テーブルスコープ**: Guild

## 用途

ギルド固有のイベント詳細スケジュールを定義します。グローバルのevent_schedule_detailsテーブルをギルド単位で上書き可能にし、ギルド独自の通知スケジュールを設定できます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| row_id | UUID | PK, NOT NULL, default=uuid_v4 | 行ID（プライマリキー） |
| guild_id | BigInteger | NOT NULL | ギルドID（Discord Guild ID） |
| profile | String | NOT NULL | ギルドイベントスケジュールとの紐づけプロファイル |
| start_day_relative | String | NOT NULL | 開始日からの相対日（例: "1", "1-5", "final"） |
| time | String | NOT NULL | イベント時間（例: "23:59", "05:00"） |
| schedule_name | String | NOT NULL | スケジュール名 |
| message_id | String | NULLABLE, FK(guild_messages.message_id) | 通知メッセージID |
| channel_id | BigInteger | NOT NULL | 通知先チャンネルID |
| reactions | String | NULLABLE | 通知メッセージに付与するリアクション |

## 制約

### プライマリキー
- `row_id`

### 外部キー
- `message_id` → `guild_messages(message_id)`（論理的な外部キー、guild_idも含む）

### UNIQUE制約
なし

## インデックス
- PK: `row_id`（自動作成）
- 推奨追加インデックス: `guild_id`, `profile`（検索性能向上）

## データサンプル
| row_id | guild_id | profile | start_day_relative | time | schedule_name | message_id | channel_id | reactions |
|--------|----------|---------|-------------------|------|--------------|-----------|-----------|-----------|
| uuid-1 | 123456789 | guild_unite_fight | 1 | 06:00 | デイリーミッション | DAILY_MISSION | 987654321 | ✅ |
| uuid-2 | 123456789 | guild_unite_fight | 1-5 | 22:00 | ボーダー更新 | BORDER_UPDATE | 987654321 | 📊 |
| uuid-3 | 987654321 | guild_xeno_clash | final | 16:00 | イベント終了1時間前 | EVENT_ENDING | 123456789 | ⏰ |

## 関連テーブル
- **参照先**: `guild_messages`（message_idで参照）
- **関連**: `event_schedule_details`（グローバル詳細スケジュール）
- **関連**: `guild_event_schedules`（profileで論理的に紐づけ）

## 備考
- row_idはUUID v4で自動生成
- グローバルのevent_schedule_detailsをギルド単位で上書き
- データ参照時は guild_event_schedule_details → event_schedule_details の順で検索
- profileでguild_event_schedulesと紐づけ
- start_day_relativeは相対日を示し、"1"は初日、"1-5"は1日目から5日目まで、"final"は最終日
- timeは時刻を示し、"HH:MM"形式
- channel_idはギルド内のチャンネルIDを指定
- schedulesテーブルに展開される際に具体的な日時に変換される

## Rust実装
- **エンティティ**: `src/models/entities/guild_event_schedule_details.rs`
- **実装状況**: 未実装
