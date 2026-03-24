# ignore付きテスト実行戦略

最終更新: 2026-03-24

## 目的

- `#[ignore]` が付いたテストを分類し、専用の実行レーンへ分ける
- 通常レーンには重すぎるが、回帰検知として重要なケースを CI で継続監視する

## 現在の分類スナップショット

- 集計日時: 2026-03-03
- 総数: `145` 件（`rg -n "#\\[ignore\\]" tests src | wc -l`）

| 区分 | 件数 | 代表パス | 実行レーン |
|---|---:|---|---|
| 実DB必須 | 138 | `tests/integration/facades/*`, `tests/data_cleanup_test.rs` | `ignored-db-tests` |
| DB接続は必要だが外部依存なし | 3 | `tests/integration/facades/guild_settings_test.rs` の一部 | `ignored-db-tests` |
| 外部 API / トークン必須 | 2 | `tests/system/bot_startup_test.rs` | 手動実行のみ |
| その他 | 2 | 新規追加時に分類を決める | 追加時に決定 |

通常レーンへ昇格済み（DB 設定が不足している場合はテスト内で明示的にスキップ）:

- `integration::facades::recruitment_new_test::test_update_message_id_not_found`
- `integration::facades::recruitment_new_test::test_new_recruitment_quest_not_found`
- `integration::facades::recruitment_schedule_test::test_create_schedule_basic`
- `integration::facades::recruitment_schedule_test::test_create_schedule_quest_not_found`
- `integration::facades::recruitment_schedule_test::test_create_schedule_invalid_time_format`

## 運用ルール

1. `#[ignore]` を付ける場合は、理由コメントを同じ行に書く。
2. 実DB必須テストは `ignored-db-tests` レーンで定期実行する。
3. Discord トークンなどの外部秘密情報が必要なテストは、通常の CI レーンから外し、手動実行のみに限定する。
4. 安定化した ignore 付きテストは、`#[ignore]` を外して通常レーンへ昇格する。

## CI レーン

- 通常レーン: `.github/workflows/ci.yml`
  - `cargo fmt -- --check`
  - `cargo clippy -j 1`
  - `cargo test -j 1`
- ワークフロー: `.github/workflows/ignored-db-tests.yml`
- 前提:
  - PostgreSQL service コンテナが利用可能であること
  - `cargo run -j 1 -- migrate-only` が完了していること
  - テスト用接続はロール別認証情報（`SYSTEM_DB_*`, `GUILD_DB_*`, `GLOBAL_DB_*`, `ADMIN_DB_*`）を使用すること
  - ignore 付きテストは既定の `DB_USER` / `DB_PASSWORD` にフォールバックしないこと
- 実行タイミング:
  - 毎日 02:00 UTC
  - `workflow_dispatch` による手動実行
- 現在の代表的な対象:
  - `integration::facades::spreadsheet_test`
  - `integration::facades::guild_settings_test`

## ローカル実行例

```bash
# 通常レーン相当
cargo fmt -- --check
cargo clippy -j 1
cargo test -j 1

# 実DB必須の ignore 付きテスト
cargo test -j 1 --test mod integration::facades::spreadsheet_test -- --ignored
cargo test -j 1 --test mod integration::facades::guild_settings_test -- --ignored
```
