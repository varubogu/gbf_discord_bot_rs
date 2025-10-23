# マルチバトル募集情報（battle_recruitments）

## 概要

**テーブル物理名**: `battle_recruitments`
**テーブルタイプ**: Transaction
**テーブルスコープ**: Community

## 用途

ユーザーが作成したマルチバトル募集を管理します。Discord上のメッセージと1対1で紐づき、募集状態、クエスト情報、部屋ID、有効期限などを保持します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| row_id | UUID | PK, NOT NULL, default=uuid_v4 | 行ID（プライマリキー） |
| guild_id | BigInteger | NOT NULL, UNIQUE(guild_id, channel_id, message_id) | ギルドID（Discord Guild ID） |
| channel_id | BigInteger | NOT NULL, UNIQUE(guild_id, channel_id, message_id) | チャンネルID（Discord Channel ID） |
| message_id | BigInteger | NOT NULL, UNIQUE(guild_id, channel_id, message_id) | メッセージID（Discord Message ID） |
| target_id | Integer | NOT NULL, FK(quests.target_id) | 対象クエストID |
| battle_type_id | Integer | NOT NULL, FK(battle_types.type_id) | バトル種類ID |
| room_id | String | NULLABLE | 共闘部屋ID（グラブルのルームID） |
| expiry_date | DateTime | NOT NULL, default=現在+1日 | 有効期限 |
| recruit_end_message_id | BigInteger | NULLABLE | 募集終了メッセージID |

## 制約

### プライマリキー
- `row_id`

### 外部キー
- `target_id` → `quests(target_id)`
- `battle_type_id` → `battle_types(type_id)`

### UNIQUE制約
- UNIQUE(`guild_id`, `channel_id`, `message_id`) - 制約名: unique_battle_recruitment_message

## インデックス

- PK: `row_id`（自動作成）
- UNIQUE: `guild_id`, `channel_id`, `message_id`（自動作成）
- FK: `target_id`（外部キー制約で自動作成）
- FK: `battle_type_id`（外部キー制約で自動作成）
- 推奨追加インデックス: `expiry_date`（期限チェック用）
- 推奨追加インデックス: `guild_id`, `channel_id`（検索性能向上）

## データサンプル

| row_id | guild_id | channel_id | message_id | target_id | battle_type_id | room_id | expiry_date | recruit_end_message_id |
|--------|----------|-----------|-----------|----------|---------------|---------|-------------|----------------------|
| uuid-1 | 123456789 | 987654321 | 1234567890123456 | 1 | 1 | ABC123 | 2025-10-24 12:00:00 | NULL |
| uuid-2 | 123456789 | 987654321 | 2345678901234567 | 2 | 2 | DEF456 | 2025-10-24 15:00:00 | 3456789012345678 |

## 関連テーブル

- **参照先**: `quests`（target_idで参照）
- **参照先**: `battle_types`（battle_type_idで参照）
- **関連**: `battle_recruitment_schedules`（message_idで論理的に関連）

## 備考

- row_idはUUID v4で自動生成
- Discord上のメッセージと1対1で紐づく（guild_id、channel_id、message_idの組み合わせで一意）
- room_idはグラブルのマルチバトルルームIDを保持
- expiry_dateはデフォルトで作成時刻+24時間に設定
- recruit_end_message_idは募集終了時に送信されるメッセージIDを保持（募集完了時に設定）
- 有効期限切れの募集は定期バッチ処理で自動的にクローズ
- ユーザーのリアクションで参加者を管理（別途participant管理機能が必要な場合は拡張）

## Rust実装

- **エンティティ**: `src/models/entities/battle_recruitments.rs`
- **実装状況**: 実装済み
