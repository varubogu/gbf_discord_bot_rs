# ギルドメッセージ定義（guild_messages）

## 概要

**テーブル物理名**: `guild_messages`
**テーブルタイプ**: Reference
**テーブルスコープ**: Guild

## 用途

ギルド固有のメッセージテンプレートを定義します。グローバルのmessagesテーブルをギルド単位で上書き可能にし、ギルド独自のメッセージ文言をカスタマイズできます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| guild_id | BigInteger | PK, NOT NULL | ギルドID（Discord Guild ID） |
| message_id | String | PK, NOT NULL | メッセージ定義ID |
| message_jp | String | NOT NULL | 日本語のメッセージテンプレート |
| reactions | String | NULLABLE | メッセージに付与するリアクション（絵文字） |
| memo | String | NULLABLE | メモ |

## 制約

### プライマリキー
- `guild_id`, `message_id`（複合キー）

### 外部キー
なし

### UNIQUE制約
なし

## インデックス
- PK: `guild_id`, `message_id`（自動作成）

## データサンプル
| guild_id | message_id | message_jp | reactions | memo |
|----------|-----------|-----------|-----------|------|
| 123456789 | RECRUITMENT_START | 【募集開始】マルチバトル募集を開始しました！ | ✅🎯 | 独自の募集開始メッセージ |
| 123456789 | EVENT_REMINDER | 【重要】イベント終了まであと1時間です！ | ⏰🚨 | 強調したリマインダー |

## 関連テーブル
- **参照元**: `guild_event_schedule_details`（message_idで参照）
- **関連**: `messages`（グローバルメッセージ定義）

## 備考
- ギルド固有のメッセージとしてグローバルメッセージを上書き
- データ参照時は guild_messages → messages の順で検索
- ギルド管理者（gbf_bot_controlロール）が設定可能
- message_idはmessagesテーブルと同じIDを使用
- ギルドごとに独自の表現や絵文字を使用可能

## Rust実装
- **エンティティ**: `src/models/entities/guild_messages.rs`
- **実装状況**: 未実装
