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

- 外部API通信が必須のケースは当面`#[ignore]`で運用する
- 外部API通信なしで検証できるケース（初期化・入力検証・DB整合性）は先行して結合テスト対象に含める
- 将来的にGoogle Sheets APIクライアントの抽象化完了後、外部API依存ケースを通常運用の結合テストへ昇格する

## 当面の対象ケース

### 1. `GuildSpreadsheetRegistrationFacade::new`

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 1-1 | 異常系：必須環境変数なし | `GOOGLE_SERVICE_ACCOUNT_KEY_FILE`未設定 | `FacadeError::Initialization`が返る |

### 2. `GuildSpreadsheetRegistrationFacade::register_guild_spreadsheets`

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 2-1 | 異常系：URL形式不正 | 不正なスプレッドシートURLを指定 | URL抽出時点でエラーが返り、DBに設定レコードが作成されない |
| 2-2 | 異常系：片方のみ不正URL | load/pushの一方のみ不正 | エラーが返り、DBに設定レコードが作成されない（部分登録なし） |

## 備考（将来拡張）

スプレッドシート関連ファサードのテストには以下の前提条件が必要：
1. Google Sheets APIクライアントのトレイト化
2. モックAPIクライアントの実装
3. テスト用スプレッドシートの用意

これらの前提条件が整った段階でテスト計画を更新する。
