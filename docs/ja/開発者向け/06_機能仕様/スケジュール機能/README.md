# スケジュール機能（タスク種別別）

## 概要

本ディレクトリは、`worker.scheduled_tasks.task_type` ごとの設計を整理した資料です。  
共通仕様（実行サイクル、実行状態、全体アーキテクチャ）は [../スケジュール機能.md](../スケジュール機能.md) を参照してください。

## タスク種別一覧

- `1: Notification`  
  - [タスク種別1_通知.md](タスク種別1_通知.md)
  - [イベントスケジュール通知.md](イベントスケジュール通知.md)
- `2: Dissolution`  
  - [タスク種別2_募集解散.md](タスク種別2_募集解散.md)
- `3: DataCleanup`  
  - [タスク種別3_データクリーンアップ.md](タスク種別3_データクリーンアップ.md)
- `4: RecurringRecruitment`  
  - [タスク種別4_定期募集実行.md](タスク種別4_定期募集実行.md)
- `5: Dismissal`  
  - [タスク種別5_人数不足解散.md](タスク種別5_人数不足解散.md)
- `6: AutoRecruitmentRotation`  
  - [タスク種別6_自動募集日付ローテーション.md](タスク種別6_自動募集日付ローテーション.md)
- `7: AutoMatching`  
  - [タスク種別7_自動マッチング.md](タスク種別7_自動マッチング.md)

## 補足

- `task_type=1` はイベント通知と募集通知の両方で利用されます。
- `task_type=3` はテーブルとRepositoryは存在しますが、スケジューラー経路は未実装です（詳細は種別3資料を参照）。

## 実装境界

- スケジュール機能のRepository traitは `src/repository/schedule/**` に定義する
- SeaORM実装は `src/infrastructure/database/repositories/schedule/**` に配置する
- 具象Repository型の配線は `src/di/repositories.rs` でのみ行う
- Facadeがトランザクション境界（begin/commit/rollback）を管理し、Serviceへトランザクションを引き渡す
