# PR06: Spreadsheet/Startup/Environment機能のリファクタリング

## 対象機能
- スプレッドシート連携
- 起動時初期化/環境値管理
- 旧 `Database` ラッパーコード（`models_database`, `db_compat`）

## 目的
- 残存する旧DBアクセスポイントを整理し、接続・セッション責務を `infrastructure/database` に一本化する。

## 設計書修正
- `docs/en/developer/06_feature_specifications/spreadsheet_integration.md`
- `docs/en/developer/06_feature_specifications/startup_validation.md`
- `docs/en/developer/05_database/connections_and_transactions.md`
  - 接続管理と旧ラッパーコード廃止方針を明記

## コード修正
- `guild_spreadsheet_config_repository` の配置/責務見直し（traitと実装分離）
- `src/services/environment/**` の `repository::database::models_database::Database` 依存解消
- `src/models/**` の `db_compat::Database` 依存を段階削除（必要なら専用serviceへ移譲）
- `db_helper` を `infrastructure/database/session` へ寄せ、責務名を明確化

## テスト修正
- spreadsheet facade/service テスト
- startup validation テスト
- environmentサービステスト

## 実行手順
1. 旧DatabaseラッパーAPIの利用箇所を洗い出し
2. 依存をservice/repository trait経由へ置換
3. 旧ラッパーコードを削除

## 完了条件
- `models_database` / `db_compat` の直接利用が解消されている
- spreadsheet/startup/environment機能で回帰なし
- 対象テストが通る
