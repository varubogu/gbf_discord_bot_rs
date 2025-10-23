# メッセージ定義（messages）

## 概要

**テーブル物理名**: `messages`
**テーブルタイプ**: Reference
**テーブルスコープ**: All

## 用途

Bot応答メッセージのテンプレートを定義します。統一的なメッセージ管理と多言語対応を実現します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| message_id | String | PK, NOT NULL | メッセージ定義ID（例: RECRUITMENT_START、EVENT_REMINDER） |
| message_jp | String | NOT NULL | 日本語のメッセージテンプレート |
| reactions | String | NULLABLE | メッセージに付与するリアクション（絵文字） |
| memo | String | NULLABLE | メモ |

## 制約

### プライマリキー
- `message_id`

### 外部キー
なし

### UNIQUE制約
なし

## インデックス

- PK: `message_id`（自動作成）

## データサンプル

| message_id | message_jp | reactions | memo |
|-----------|-----------|-----------|------|
| RECRUITMENT_START | マルチバトル募集を開始しました | ✅ | 募集開始メッセージ |
| RECRUITMENT_FULL | 募集が満員になりました | 🎉 | 募集満員メッセージ |
| EVENT_REMINDER | イベント終了まであと1時間です | ⏰ | イベント終了リマインダー |

## 関連テーブル

- **参照元**: `event_schedule_details`（message_idで参照）
- **参照元**: `schedules`（message_idで参照）
- **関連**: `guild_messages`（ギルド固有のメッセージ定義）

## 備考

- グローバルメッセージテンプレートとして全ギルドで使用
- guild_messagesで上書き可能
- reactionsフィールドでメッセージに自動付与する絵文字を定義
- 将来的に多言語対応のため message_en などを追加可能

## Rust実装

- **エンティティ**: `src/models/entities/message_texts.rs`（messagesに相当）
- **実装状況**: 実装済み
