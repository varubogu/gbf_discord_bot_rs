# データクリーンアップ実行方法

データクリーンアップは外部バッチ経由でメンテナンスコンテナを起動して実行します。

## 概要

- 30日以上前の古いデータを自動削除
- Bot本体とは完全に独立したメンテナンスコンテナで実行
- デッドロックの心配なし（手動実行時も1つのコンテナのみ起動）
- **専用データベースロール（`gbf_bot_cleanup`）**を使用し、最小権限の原則を適用

## 開発環境での実行

### 前提条件

- `.env.maintenance` ファイルにDB接続情報が設定されている
- PostgreSQLが起動している

### 実行方法

```bash
# .envファイルから環境変数を自動読み込み
cargo run --bin cleanup
```

### 環境変数を直接指定して実行

```bash
DB_HOST=localhost \
DB_PORT=5432 \
DB_USER=postgres \
DB_PASSWORD=your_password \
DB_NAME=gbf_bot_db \
CLEANUP_RETENTION_DAYS=30 \
cargo run --bin cleanup
```

## 前提条件

### データベースロールの作成

初回のみ、マイグレーションを実行してCleanupロールを作成する必要があります：

```bash
# マイグレーション実行
cargo run -- migrate

# または本番環境で
docker compose exec app /app/gbf_discord_bot_rs migrate
```

これにより、以下が自動的に作成されます：
- `gbf_bot_cleanup` ロール
- workerスキーマの特定テーブルに対するDELETE + SELECT権限
- RLS BYPASS権限（全ギルドのデータを対象とするため）

詳細は [データベースロール設計書](../develop/design/features/data_cleanup_system_database_role.md) を参照してください。

## 手動実行方法

### 本番サーバーでの実行

SSHで本番サーバーに入り、以下のコマンドを実行：

```bash
# プロジェクトディレクトリに移動
cd /path/to/gbf_discord_bot_rs

# メンテナンスコンテナを実行
docker compose run --rm maintenance
```

### 実行結果の確認

```bash
# 標準出力に以下のようなログが表示されます
[INFO] データクリーンアップを開始します
[INFO] cleanup_before=2025-11-26T18:00:00Z retention_days=30 削除基準日時を計算しました
[INFO] deleted_count=2345 battle_recruitmentsを削除しました
[INFO] deleted_count=1234 notificationsを削除しました
[INFO] deleted_count=3456 scheduled_tasksを削除しました
[INFO] データクリーンアップが正常に完了しました
```

## cron定期実行（推奨）

本番サーバーでcronを設定し、毎日自動実行することを推奨します。

### cron設定例

```bash
# crontab -e で編集
# 毎日AM3時（JST）に実行
0 3 * * * cd /path/to/gbf_discord_bot_rs && docker compose run --rm maintenance >> /var/log/gbf-cleanup.log 2>&1
```

### ログローテーション設定

`/etc/logrotate.d/gbf-cleanup` を作成：

```
/var/log/gbf-cleanup.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
}
```

## 削除対象データ

以下のテーブルから30日以上前のデータを削除：

### 1. battle_recruitments（マルチ募集）
- **削除条件**: クエスト開始日時が30日以上前 AND 募集終了済み
- **CASCADE削除**: recruitment_participants, battle_recruitment_dismissals, notification_rel_battle_recruitments, scheduled_task_dissolutions, scheduled_task_dismissals

### 2. notifications（通知）
- **削除条件**: 通知予定日時が30日以上前 AND 送信済み
- **CASCADE削除**: notification_rel_battle_recruitments, notification_rel_event_schedules, scheduled_task_notifications

### 3. scheduled_tasks（スケジュールタスク）
- **削除条件**: 実行予定日時が30日以上前 AND 実行済み AND DataCleanupタスク以外
- **CASCADE削除**: scheduled_task_notifications, scheduled_task_dissolutions, scheduled_task_dismissals, scheduled_task_recurring_recruitments

## 保持期間の変更

デフォルトは30日ですが、環境変数で変更可能です。

### 一時的に変更して実行

```bash
docker compose run --rm -e CLEANUP_RETENTION_DAYS=60 maintenance
```

### 恒久的に変更

`.env.maintenance` ファイルを編集：

```bash
CLEANUP_RETENTION_DAYS=60
```

## トラブルシューティング

### エラー時の対応

クリーンアップ中にエラーが発生した場合、トランザクションが自動的にロールバックされます。データの一貫性は保たれます。

### ログ確認

```bash
# cronログを確認
tail -f /var/log/gbf-cleanup.log

# docker composeログを確認
docker compose logs maintenance
```

### デバッグモード

より詳細なログを出力する場合：

```bash
docker compose run --rm -e RUST_LOG=debug maintenance
```

## 注意事項

- **複数同時実行は不要**: cronと手動実行が重なってもデッドロックの心配はありませんが、無駄なので避けてください
- **実行時間帯**: ユーザーが少ない深夜帯（AM3時推奨）に実行
- **バックアップ**: 削除されたデータは復元できません。定期的なDBバックアップを推奨
