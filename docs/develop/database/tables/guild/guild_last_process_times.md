# ギルド最終処理実行日時（guild_last_process_times）

## 概要

**テーブル物理名**: `guild_last_process_times`
**テーブルタイプ**: History
**テーブルスコープ**: Guild

## 用途

ギルド単位のバッチ処理最終実行時刻を記録します。ギルド固有の定期処理の重複防止や、障害復旧時の基準時刻として使用されます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| guild_id | BigInteger | PK, NOT NULL | ギルドID（Discord Guild ID） |
| process_type | Integer | PK, NOT NULL | 処理種類ID |
| execute_time | DateTime | NULLABLE | 処理実行日時 |
| memo | String | NULLABLE | メモ（処理の説明） |

## 制約

### プライマリキー
- `guild_id`, `process_type`（複合キー）

### 外部キー
なし

### UNIQUE制約
なし

## インデックス
- PK: `guild_id`, `process_type`（自動作成）

## データサンプル
| guild_id | process_type | execute_time | memo |
|----------|-------------|-------------|------|
| 123456789 | 1 | 2025-10-23 12:00:00 | ギルド通知処理 |
| 123456789 | 2 | 2025-10-23 11:55:00 | ギルド募集期限チェック |
| 987654321 | 1 | 2025-10-23 12:05:00 | ギルド通知処理 |

## 関連テーブル
- **関連**: `last_process_times`（グローバルの処理実行時刻）

## 備考
- ギルド固有の定期処理の実行時刻を記録
- execute_timeがNULLの場合は未実行を示す
- ギルド単位のバッチ処理の重複実行を防止
- 障害復旧時にギルドごとの最後の正常実行時刻を確認可能
- グローバルのlast_process_timesと独立して管理

## Rust実装
- **エンティティ**: `src/models/entities/guild_last_process_times.rs`
- **実装状況**: 未実装
