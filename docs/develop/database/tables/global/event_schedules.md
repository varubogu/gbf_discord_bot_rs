# イベントスケジュール（event_schedules）

## 概要

**テーブル物理名**: `event_schedules`
**テーブルタイプ**: Reference
**テーブルスコープ**: All

## 用途

グラブルイベントの開催スケジュールを定義します。イベント種類、開催回数、期間、有利属性などを管理し、詳細スケジュールとの紐づけを行います。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| row_id | UUID | PK, NOT NULL, default=uuid_v4 | 行ID（プライマリキー） |
| event_type | String | NOT NULL, UNIQUE(event_type, event_count) | イベント種類（例: 古戦場、ゼノ撃滅戦） |
| event_count | BigInteger | NOT NULL, UNIQUE(event_type, event_count) | イベント開催回数（第N回） |
| profile | String | NULLABLE | イベント詳細スケジュールとの紐づけプロファイル |
| weak_attribute | Integer | NULLABLE, FK(elements.element_id) | 有利属性ID |
| start_at | DateTime | NOT NULL | 開始日時 |
| end_at | DateTime | NOT NULL | 終了日時 |

## 制約

### プライマリキー
- `row_id`

### 外部キー
- `weak_attribute` → `elements(element_id)`

### UNIQUE制約
- UNIQUE(`event_type`, `event_count`) - 制約名: unique_event_schedule

## インデックス

- PK: `row_id`（自動作成）
- UNIQUE: `event_type`, `event_count`（自動作成）
- FK: `weak_attribute`（外部キー制約で自動作成）

## データサンプル

| row_id | event_type | event_count | profile | weak_attribute | start_at | end_at |
|--------|-----------|------------|---------|----------------|----------|--------|
| uuid-1 | 古戦場 | 65 | unite_fight | 1 | 2025-10-15 19:00:00 | 2025-10-22 23:59:00 |
| uuid-2 | ゼノ撃滅戦 | 30 | xeno_clash | 2 | 2025-10-20 17:00:00 | 2025-10-27 16:59:00 |

## 関連テーブル

- **参照元**: `schedules`（parent_schedule_idで参照）
- **参照先**: `elements`（weak_attributeで参照）
- **関連**: `event_schedule_details`（profileで論理的に紐づけ）
- **関連**: `guild_event_schedules`（ギルド固有のイベントスケジュール）

## 備考

- row_idはUUID v4で自動生成
- event_typeとevent_countの組み合わせで一意に識別
- profileは event_schedule_details との紐づけに使用
- weak_attributeでイベントの有利属性を管理
- guild_event_schedulesで上書き可能

## Rust実装

- **エンティティ**: `src/models/entities/event_schedules.rs`
- **実装状況**: 実装済み
