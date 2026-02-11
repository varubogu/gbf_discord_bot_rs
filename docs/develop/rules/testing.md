# テストルール（総論）

## 目的

本ドキュメントは、テスト戦略の全体方針と参照先を定義する。
詳細な設計内容は `docs/develop/design/testing/` 配下の設計書で管理する。

## 基本方針

- テストは「単体」「結合」「総合（E2E）」の責務を分離する。
- アーキテクチャ依存方向（`events -> facades -> services -> repository`）を前提に設計する。
- テストは再実行可能で、順序に依存しないことを必須とする。
- 外部依存やDB状態はテスト目的に応じて明示的に制御する。

## ドキュメント構成

- 全体設計: `docs/develop/rules/testing/overview.md`
- 単体テスト方針: `docs/develop/rules/testing/unit_test.md`
- 結合テスト方針: `docs/develop/rules/testing/integration_test.md`
- 結合テスト個別設計書: `docs/develop/design/testing/integration/`

## 運用ルール

- 新規機能の追加時は、該当テスト種別の方針書と個別設計書を更新する。
- 個別テスト仕様（前提データ、ケース、期待結果）は総論へ書かず、機能別設計書に記載する。
- ルール間に矛盾がある場合は、より具体的な個別方針を優先し、必要に応じて総論を更新する。
