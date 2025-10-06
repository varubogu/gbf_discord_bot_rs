# AGENTS.md

## 目的
本ドキュメントは、GBF Discord Bot (Rust) リポジトリで活動するAIエージェント向けの行動指針です。`CLAUDE.md`、`.cursor/rules/rules.mdc`、および`docs/develop`配下の設計書を統合し、役割横断で守るべき原則と各エージェントの期待値を整理します。

## 参照ドキュメント
- `CLAUDE.md`: 開発全体の基本方針と代表的なコマンド
- `.cursor/rules/rules.mdc`: ルール集へのエントリーポイント
- `docs/develop/architecture/`: プロジェクト構造とアーキテクチャ指針
- `docs/develop/rules/`: コーディング、テスト、セキュリティ等の詳細ルール
- `docs/develop/design/database/`: DB接続・トランザクション設計
- `docs/develop/features/`: 機能別の要件と設計（例: `quest_recruitment.md`, `schedule_notification.md`）

## プロジェクト概要
- Rust + poise + SeaORM + PostgreSQL で実装されたGranblue Fantasy向けDiscord Bot
- クリーンアーキテクチャをベースに、`events → facades → services → repository`の単方向依存を徹底
- `AppState`パターンで依存性を注入し、単一DB接続を共有
- ドキュメント・コメント・エラーメッセージは日本語、コードは英語命名が原則

## 共通原則
### 言語とスタイル
- コメント、ドキュメント、エラーメッセージは日本語で記述
- 命名規則: 構造体/列挙体はPascalCase、関数/変数はsnake_case、定数はSCREAMING_SNAKE_CASE
- `unwrap()`禁止、`panic!()`は非回復例外のみ

### アーキテクチャと責務
- 層を跨いだ直接呼び出し禁止（例: Facade→Repositoryは不可、Service経由）
- Facade層のみがトランザクションを begin/commit/rollback する
- Service層は引数で受け取ったトランザクションをそのままRepositoryに渡す
- Repository層はビジネスロジックを持たず、永続化と取得に専念
- `AppState`で依存を組み立て、各層で独自にDB接続を生成しない

### エラーハンドリングとログ
- `thiserror`で層ごとのエラー型を定義し、`#[from]`で変換
- `tracing`を用いた構造化ログを出力（error/warn/info/debugの使い分け）
- ログに機密情報を含めない。業務例外はwarn、システム障害はerror

### パフォーマンスと設計
- 不要な`clone()`を避け、借用/参照で処理する
- `Arc<T>`の多用を避け、必要な箇所に限定
- 並行処理可能箇所は`try_join_all`等を活用しつつ、トランザクション長期化は避ける
- Builderパターンなど、Rustらしいゼロコスト抽象化を優先

### セキュリティ
- 入力検証は必ずプレゼンテーション層で実施（Regex・許可リスト・型変換）
- Discord権限チェックとアプリ固有権限の確認を徹底
- SQLインジェクション対策としてSeaORMのクエリビルダを利用し、生SQLは準備済みステートメントで保護
- Discordメッセージへ出力する際は必要に応じてサニタイゼーション

### テスト
- 各層に単体テストを用意し、`mockall`等で依存を分離
- Facade層で結合テスト、Repository層は実DB（テスト用）で検証
- Arrange-Act-Assertパターンで可読性を維持
- `cargo test`, `cargo clippy`, `cargo fmt`での検証を前提

### ワークフロー
- 変更は必ずテスト・ドキュメント更新を伴う（`docs/`と`locales/`を含め確認）
- 影響範囲を設計書で確認し、該当する`docs/develop/features/`や`docs/develop/architecture/`を更新
- ブランチ命名: `feature/`, `fix/`, `refactor/`, `remove/`, `docs/`
- 追加→修正→削除の各フェーズでチェックリスト（設計→実装→ドキュメント→検証）を順守

## エージェント別ガイド
### 実装エージェント（例: Coding, Fixer）
1. 要件・影響範囲を`docs/develop/features/`とルール群で調査
2. 層の責務とトランザクション境界を再確認してから実装
3. 新規コードには日本語コメント/エラーメッセージ、`thiserror`エラー、`tracing`ログを適用
4. 単体テスト・結合テストを追加/更新し、必要に応じてモックを整備
5. 実装後に`cargo fmt`, `cargo clippy`, `cargo test`を実行し、結果を報告

### レビューエージェント
1. 変更がクリーンアーキテクチャとトランザクション規約に従っているか確認
2. エラー型、ログ、入力検証、セキュリティ対策が仕様通りかをチェック
3. テスト網羅性（層別単体テスト、Facade結合テスト、DBテスト）とドキュメント更新の有無を確認
4. パフォーマンス影響（不要な`clone`やArcなど）と長時間トランザクションを警告
5. 問題があれば具体的な修正提案と参照ドキュメントを提示

### ドキュメントエージェント
1. 変更対象機能の設計書を特定し、`docs/develop/architecture/`・`docs/develop/features/`を更新
2. 抽象度を保ち、具体的なコード例ではなく責務・フロー・制約を説明
3. ドキュメント更新時は関連するルールファイルとの整合性を確認
4. ユーザー向け資料（`docs/user/`や`locales/`）への影響もレビュー
5. 必要に応じて図表（Mermaid等）で処理フローを追記

## 作業チェックリスト
- [ ] 対象機能の設計書・ルールを事前に確認した
- [ ] 層別責務・トランザクション規約を守っている
- [ ] `thiserror`エラーと`tracing`ログを適切に実装した
- [ ] `cargo fmt`, `cargo clippy`, `cargo test`を実行した
- [ ] docs・ローカライズ・マイグレーション等の関連成果物を更新した
- [ ] セキュリティ／パフォーマンス影響を評価し、懸念を記録した

## 判断に迷った場合
- 設計意図は`docs/develop/architecture/`と`docs/develop/design/database/`を優先参照
- 機能仕様は`docs/develop/features/`を参照し、実装との差異をドキュメント側で補正
- ルールの競合が起きたら最も具体的な設計書を優先し、必要ならドキュメントを更新
- 未定義の挙動や外部依存が関係する場合は、GitHub Issueや設計書への追記を提案
