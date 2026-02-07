# スプレッドシートファサード 結合テスト計画書

## 対象ファイル

- `src/facades/spreadsheet/core/import_facade.rs`
- `src/facades/spreadsheet/core/export_facade.rs`
- `src/facades/spreadsheet/global_load_facade.rs`
- `src/facades/spreadsheet/global_push_facade.rs`
- `src/facades/spreadsheet/guild_load_facade.rs`
- `src/facades/spreadsheet/guild_push_facade.rs`
- `src/facades/spreadsheet/guild_spreadsheet_registration_facade.rs`

## テスト方針

- **現時点ではテスト対象外とする**
- 理由：Google Sheets APIへの依存が強く、外部サービスのモック化が必要
- 将来的にGoogle Sheets APIクライアントのトレイト抽象化が完了した際にテスト計画を策定する

## 備考

スプレッドシート関連ファサードのテストには以下の前提条件が必要：
1. Google Sheets APIクライアントのトレイト化
2. モックAPIクライアントの実装
3. テスト用スプレッドシートの用意

これらの前提条件が整った段階でテスト計画を更新する。
