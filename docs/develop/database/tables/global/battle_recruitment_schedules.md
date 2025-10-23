# マルチ募集スケジュール（battle_recruitment_schedules）

## 概要

**テーブル物理名**: `battle_recruitment_schedules`
**テーブルタイプ**: Transaction
**テーブルスコープ**: All

## 用途

マルチバトル募集に関連する通知スケジュールを管理します。schedulesテーブルと1対1で紐づき、マルチ募集固有の情報（対象メッセージID）を保持します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| row_id | UUID | PK, NOT NULL, default=uuid_v4 | 行ID（プライマリキー） |
| parent_id | UUID | NOT NULL, FK(schedules.row_id) | スケジュール行ID |
| message_id | BigInteger | NOT NULL | 通知対象のマルチ募集メッセージID |

## 制約

### プライマリキー
- `row_id`

### 外部キー
- `parent_id` → `schedules(row_id)`

### UNIQUE制約
なし

## インデックス

- PK: `row_id`（自動作成）
- FK: `parent_id`（外部キー制約で自動作成）
- 推奨追加インデックス: `message_id`（検索性能向上）

## データサンプル

| row_id | parent_id | message_id |
|--------|----------|-----------|
| uuid-1 | uuid-s1 | 1234567890123456 |
| uuid-2 | uuid-s2 | 2345678901234567 |

## 関連テーブル

- **参照先**: `schedules`（parent_idで参照）
- **関連**: `battle_recruitments`（message_idで論理的に関連）

## 備考

- row_idはUUID v4で自動生成
- schedulesテーブルと1対1の関係
- message_idはDiscordのメッセージIDを示し、battle_recruitmentsテーブルのメッセージと対応
- マルチ募集のリマインダー通知や期限切れ通知に使用

## Rust実装

- **エンティティ**: `src/models/entities/notification_rel_battle_recruitments.rs`
- **実装状況**: 実装済み（notification_rel_battle_recruitmentsとして）
