# 技術スタック・アーキテクチャ

## 技術スタック

### コア技術
- **Rust** - edition 2024
- **poise 0.6.1** - Discord Bot フレームワーク
- **PostgreSQL** - データベース
- **SeaORM 1.1** - ORM（Object-Relational Mapping）

### 主要ライブラリ
- **tokio 1.47** - 非同期ランタイム
- **tracing 0.1 / tracing-subscriber 0.3** - 構造化ログ
- **thiserror 1.0** - エラーハンドリング
- **chrono 0.4 / chrono-tz 0.10** - 日時処理・タイムゾーン
- **google-sheets4 5.0** - Google Sheets API連携
- **tokio-cron-scheduler 0.15** - スケジューラー
- **uuid 1.0** - UUID生成
- **serde 1.0 / serde_json 1.0** - シリアライズ・デシリアライズ
- **regex 1.11** - 正規表現
- **async-trait 0.1** - 非同期トレイト
- **mockall 0.13** - テスト用モック生成

## アーキテクチャ概要

このアプリケーションはDiscord上でグランブルーファンタジー（以下、グラブル）のサポートをしてくれるBotです。

クリーンアーキテクチャを採用し、以下の層構造で設計されています：

1. **プレゼンテーション層** - Discord イベント・インタラクションの処理
2. **アプリケーション層** - ビジネスロジック（Facade、Service）
3. **データアクセス層** - データ永続化（Repository、Infrastructure）

各層間の依存関係は一方向に保たれ、外部依存関係は依存性注入パターンによって管理されています。