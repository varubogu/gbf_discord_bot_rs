# ignoredテスト実行戦略

最終更新: 2026-03-03

## 目的
- `#[ignore]` が多い統合テストを分類し、定期実行レーンを分離する。
- 「通常テストでは重いが、回帰検知としては重要」なケースをCIで継続監視する。

## 現状分類（2026-03-03時点）
- 総数: `145` 件（`rg -n "#\\[ignore\\]" tests src | wc -l`）

| 区分 | 件数 | 代表パス | 実行レーン |
|---|---:|---|---|
| DB必須 | 138 | `tests/integration/facades/*`, `tests/data_cleanup_test.rs` | `ignored-db-tests` |
| DB接続は必要だが外部依存なし | 3 | `tests/integration/facades/guild_settings_test.rs` の一部 | `ignored-db-tests` |
| 外部API/トークン必須 | 2 | `tests/system/bot_startup_test.rs` | 手動実行のみ |
| その他 | 2 | 将来分類（新規追加時に理由コメント必須） | 追加時に決定 |

通常レーンへ昇格済み（DB未設定時はテスト内で明示スキップ）:
- `integration::facades::recruitment_new_test::test_update_message_id_not_found`
- `integration::facades::recruitment_new_test::test_new_recruitment_quest_not_found`
- `integration::facades::recruitment_schedule_test::test_create_schedule_basic`
- `integration::facades::recruitment_schedule_test::test_create_schedule_quest_not_found`
- `integration::facades::recruitment_schedule_test::test_create_schedule_invalid_time_format`

## 運用ルール
1. `#[ignore]` を付ける場合、必ず理由コメントを同一行に書く。
2. DB必須テストは `ignored-db-tests` レーンで定期実行する。
3. Discordトークン等の外部秘密情報が必要なテストは、CI通常レーンから除外し手動実行に限定する。
4. 安定化できた ignored テストは通常レーンへ昇格する（`#[ignore]` を外す）。

## CIレーン
- 通常レーン: `.github/workflows/ci.yml`
  - `cargo fmt -- --check`
  - `cargo clippy -j 1`
  - `cargo test -j 1`
- ワークフロー: `.github/workflows/ignored-db-tests.yml`
- 前提:
  - PostgreSQL service コンテナ
  - `cargo run -j 1 -- migrate-only` 実行済み
- 実行タイミング:
  - 毎日 02:00 UTC（schedule）
  - 手動実行（workflow_dispatch）
- 現在の実行対象（Facade統合の代表）:
  - `integration::facades::spreadsheet_test`
  - `integration::facades::guild_settings_test`

## ローカル実行例
```bash
# 通常レーン相当
cargo fmt -- --check
cargo clippy -j 1
cargo test -j 1

# DB必須 ignored テスト（対象フィルタ）
cargo test -j 1 --test mod integration::facades::spreadsheet_test -- --ignored
cargo test -j 1 --test mod integration::facades::guild_settings_test -- --ignored
```
