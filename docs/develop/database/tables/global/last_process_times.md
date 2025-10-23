# 最終処理実行日時（last_process_times）

## 概要

**テーブル物理名**: `last_process_times`
**テーブルタイプ**: History
**テーブルスコープ**: All

## 用途

バッチ処理の最終実行時刻を記録します。定期実行処理の重複防止や、障害復旧時の基準時刻として使用されます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| process_type | Integer | PK, NOT NULL | 処理種類ID（例: 1=通知処理、2=募集期限チェック） |
| execute_time | DateTime | NULLABLE | 処理実行日時 |
| memo | String | NULLABLE | メモ（処理の説明） |

## 制約

### プライマリキー
- `process_type`

### 外部キー
なし

### UNIQUE制約
なし

## インデックス

- PK: `process_type`（自動作成）

## データサンプル

| process_type | execute_time | memo |
|-------------|-------------|------|
| 1 | 2025-10-23 12:00:00 | 通知処理 |
| 2 | 2025-10-23 11:55:00 | 募集期限チェック |
| 3 | 2025-10-23 00:00:00 | イベントスケジュール展開 |

## 関連テーブル

- **関連**: `guild_last_process_times`（ギルド固有の処理実行時刻）

## 備考

- グローバルな定期処理の実行時刻を記録
- execute_timeがNULLの場合は未実行を示す
- バッチ処理の重複実行を防止するために使用
- 障害復旧時に最後の正常実行時刻を確認可能

## Rust実装

- **エンティティ**: `src/models/entities/last_process_times.rs`
- **実装状況**: 実装済み
