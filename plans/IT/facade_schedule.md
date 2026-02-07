# スケジュールファサード 結合テスト計画書

## 対象ファイル

- `src/facades/schedule/notification_schedule_facade.rs`
- `src/facades/schedule/schedule_query_facade.rs`

## テスト方針

- 実際のPostgreSQLデータベースを使用する結合テスト（`#[ignore]`付き）
- Gateway依存なし（DB操作のみ）
- テストデータ（notifications, scheduled_tasks等）はDBに事前登録が必要
- テストデータは各テスト内で作成・テスト後にクリーンアップ

---

## 対象関数とテストケース

### 1. `NotificationScheduleFacade::get_future_notifications_formatted`（今後の通知一覧）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 1-1 | 正常系：今後の通知が存在する | 未来日時のnotificationsデータあり | フォーマットされた通知一覧文字列が返る |
| 1-2 | 正常系：今後の通知が存在しない | 未来日時のnotificationsデータなし | 空または「通知なし」のメッセージが返る |
| 1-3 | 正常系：limit制限の確認 | 通知が多数存在、limit=5 | 5件以下の通知が返る |

### 2. `NotificationScheduleFacade::get_notification_history_formatted`（通知履歴）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 2-1 | 正常系：通知履歴取得 | 過去の送信済みnotificationsあり | フォーマットされた履歴文字列と`ScheduleStats`のタプルが返る |
| 2-2 | 正常系：履歴なし | 指定期間内のnotificationsなし | 空の履歴と統計が返る |

### 3. `ScheduleQueryFacade::get_stats`（通知統計取得）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 3-1 | 正常系：統計データ取得 | scheduled_tasks, notificationsデータあり | `ScheduleStats`に正しい統計値が含まれる |
| 3-2 | 正常系：データなし | 対象ギルドのデータなし | ゼロ値の統計が返る |
| 3-3 | 正常系：期間指定の確認 | days=7で指定 | 過去7日分の統計のみが集計される |
