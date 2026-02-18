# PR02: Scheduling機能のリファクタリング

## 対象機能
- スケジューラ本体
- 通知・定期募集・解散・クリーンアップ関連

## 目的
- `schedule` 関連の「trait層」と「SeaORM実装層」を完全分離し、依存方向を明確化する。

## 設計書修正
- `docs/en/developer/06_feature_specifications/scheduling_feature.md`
- `docs/en/developer/06_feature_specifications/scheduling_feature/*.md`
  - 実装参照パスを `repository -> infrastructure/database/repositories/schedule` に更新
  - トランザクション責務（Facade開始）を追記

## コード修正
- `src/repository/schedule/**`: trait定義のみを維持
- `src/infrastructure/database/repositories/schedule/**`: SeaORM実装を集約
- `src/services/schedule/**`, `src/facades/schedule/**`:
  - 具象実装直接参照を除去
  - trait経由でDIから受け取る形に統一
- `src/bin/cleanup.rs`:
  - 実装型importを新パスへ変更

## テスト修正
- `services/schedule` のユニットテスト
- scheduling関連統合テスト
- cleanupバイナリの最小動作確認

## 実行手順
1. Scheduling配下の実装importを新パスへ切替
2. 対象領域のimportを新パスへ置換
3. スケジューラ起動とタスク取得の回帰確認

## 完了条件
- `src/repository/schedule` に実装コードが存在しない
- Scheduling機能で `repository::database::schedule` 直接依存が残らない
- 対象テストが通る
