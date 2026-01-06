# ギルドイベントスケジュール（guild_event_schedules）

## 概要

**テーブル物理名**: `guild_event_schedules`
**スキーマ名**: `guild_master`
**テーブルタイプ**: Reference
**テーブルスコープ**: Guild（ギルド固有）
**実装状況**: ✅ 実装済み

## 用途

ギルド固有のイベントスケジュールを定義します。グローバルのevent_schedulesテーブルをギルド単位で上書き可能にし、ギルド独自のイベント期間設定を実現します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| row_id | UUID | PK, NOT NULL, default=uuid_v4 | 行ID（プライマリキー） |
| guild_id | BigInteger | NOT NULL, UNIQUE(event_type, event_count, guild_id) | ギルドID（Discord Guild ID） |
| event_type | String | NOT NULL, UNIQUE(event_type, event_count, guild_id) | イベント種類 |
| event_count | BigInteger | NOT NULL, UNIQUE(event_type, event_count, guild_id) | イベント開催回数（第N回） |
| profile | String | NULLABLE | ギルド詳細スケジュールとの紐づけプロファイル |
| weak_attribute | Integer | NULLABLE, FK(elements.element_id) | 有利属性ID |
| start_at | DateTime | NOT NULL | 開始日時 |
| end_at | DateTime | NOT NULL | 終了日時 |

## 制約

### プライマリキー
- `row_id`

### 外部キー
- `weak_attribute` → `elements(element_id)`

### UNIQUE制約
- UNIQUE(`event_type`, `event_count`, `guild_id`) - 制約名: unique_guild_event_schedule

## インデックス
- PK: `row_id`（自動作成）
- UNIQUE: `event_type`, `event_count`, `guild_id`（自動作成）
- FK: `weak_attribute`（外部キー制約で自動作成）
- 推奨追加インデックス: `guild_id`（検索性能向上）

## データサンプル
| row_id | guild_id | event_type | event_count | profile | weak_attribute | start_at | end_at |
|--------|----------|-----------|------------|---------|----------------|----------|--------|
| uuid-1 | 123456789 | 古戦場 | 65 | guild_unite_fight | 1 | 2025-10-15 20:00:00 | 2025-10-22 22:00:00 |
| uuid-2 | 987654321 | ゼノ撃滅戦 | 30 | guild_xeno_clash | 2 | 2025-10-20 18:00:00 | 2025-10-27 17:00:00 |

## 関連テーブル
- **参照先**: `elements`（weak_attributeで参照）
- **関連**: `event_schedules`（グローバルイベントスケジュール）
- **関連**: `guild_event_schedule_details`（profileで論理的に紐づけ）

## 備考
- row_idはUUID v4で自動生成
- グローバルのevent_schedulesをギルド単位で上書き
- データ参照時は guild_event_schedules → event_schedules の順で検索
- event_type、event_count、guild_idの組み合わせで一意に識別
- profileはguild_event_schedule_detailsとの紐づけに使用
- ギルドごとに独自のイベント期間や有利属性を設定可能

## Rust実装
- **エンティティファイル**: `src/models/entities/guild_master/guild_event_schedules.rs`
- **マイグレーションファイル**: `migration/src/m*_create_guild_event_schedules.rs`
- **実装状況**: ✅ 実装済み
