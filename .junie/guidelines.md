# junie guidelines

## チャット応答について

- 常に日本語で応答する

## ファイル出力について

JunieやAIチャットが出力するmarkdownの設計書・説明書は
`.tmp/works/xxxxx.md`
に出力する。
このフォルダはコミットされないのでソースコードを汚す心配がないためである。

## アプリケーション概要

このアプリはDiscord上でグランブルーファンタジー（以下、グラブル）のサポートをしてくれるBot

## 技術仕様・設計について

プロジェクトの技術仕様と設計に関する詳細は、以下の設計書を参照してください：

- **[技術スタック・アーキテクチャ](../docs/develop/design/technology_stack_architecture.md)**: 使用技術とアーキテクチャ概要
- **[プロジェクト構成](../docs/develop/design/project_structure.md)**: ディレクトリ構造と各層の役割

## 開発ルールについて

開発時に従うべき詳細なルールは、テーマごとに以下のファイルに分割されています：

- **[アーキテクチャルール](../docs/develop/rules/architecture.md)**: クリーンアーキテクチャの層間責務とRustらしい設計原則
- **[依存性注入ルール](../docs/develop/rules/dependency_injection.md)**: DIパターンとDB接続管理
- **[エラーハンドリングルール](../docs/develop/rules/error_handling.md)**: 構造化エラーと層別エラーハンドリング戦略
- **[パフォーマンスルール](../docs/develop/rules/performance.md)**: DB最適化、メモリ管理、非同期処理
- **[セキュリティルール](../docs/develop/rules/security.md)**: 入力検証、SQLインジェクション対策、権限管理
- **[テストルール](../docs/develop/rules/testing.md)**: 単体・結合テスト戦略とテストダブル使用指針
- **[ログ・監視ルール](../docs/develop/rules/logging.md)**: 構造化ログとメトリクス収集
- **[ワークフロー](../docs/develop/rules/workflow.md)**: 遵守すべき作業手順

これらのルールファイルは必ず参照し、遵守してください。

