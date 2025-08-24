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

## 技術スタック・アーキテクチャ

- Rust
- poise (discord bot)
- Postgres (database)
- sea-orm (ORM)

## プロジェクト構成について

```
(root)
├── .junie   # junieの設定フォルダ
├── .tmp     # 一時的な作業用。この中身はコミットしないので検証用に自由に使って良い。
├── docs/    # ドキュメント格納先
├── locales/  # 翻訳ファイルを格納
│   ├── ja.json  # SvelteコンポーネントのE2Eテスト
│   ├── design/    # 設計書
│   └── features/  # 一般の人向けの説明書。使用できるコマンドなどもここに含む
├── src
│   ├── events/            # discord botのイベント・インタラクション全般
│   │   ├── handlers/      # discord botのイベント（on_messageやon_reaction_add）を定義
│   │   ├── interactions/  # discord botのインタラクション全般
│   │   │   ├── command_interactions/   # インタラクションのうちユーザーのコマンド依存 
│   │   │   │   └── slash/              # スラッシュコマンド定義
│   │   │   ├── components/             # コンポーネントインタラクション
│   │   │   └── modal/                  # モーダルインタラクション
│   │   └── handler.rs     # discord botのイベント・インタラクション全般
│   ├── facades/        # Facade層。主にユーザーの何かしらのアクション（events、インタラクション、スケジュール起動）から呼び出される
│   ├── models/         # facades,servicesで使用する構造体を定義
│   │   └── entities/   # DBのテーブル定義・マッピング（src/models/直下のモデルとは相互変換が可能）
│   ├── repository/     # データの永続化（保存・読み込み）
│   │   └── database/   # DBへのCRUD操作の処理（sea-ormベース）
│   ├── services/                   # 単一の処理を提供するサービス層
│   │   ├── battle_recruitment/     # マルチバトル募集・通知サービス
│   │   ├── environment/            # 環境変数読み込み
│   │   ├── message/                # メッセージの抽象化層（rust-i8n＋サーバーごとに独自で上書き可能なメッセージ）
│   │   └── permission/             # discordパーミッションに関するサービス
│   ├── types/       # type/enumの定義（※ドメインオブジェクトはsrc/models/に配置する
│   ├── utils/       # プロジェクト内で使う汎用処理
│   ├── lib.rs       # ライブラリクレートとしての最上位。主にテストの時のモジュール宣言用
│   └── main.rs      # エントリポイント
├── target       # cargoのビルド等の出力先
├── tests        # 結合テスト
├── .env         # 開発時の環境変数定義（重要な認証情報が含まれるため読み込み禁止）
├── .env.example # 開発時の環境変数定義の見本
├── Cargo.lock   # Cargoインストール済みクレート
└── Cargo.toml   # Cargo設定ファイル
```

## 開発ルールについて

開発時に従うべき詳細なルールは、テーマごとに以下のファイルに分割されています：

- **[アーキテクチャルール](../docs/develop/rules/architecture.md)**: クリーンアーキテクチャの層間責務とRustらしい設計原則
- **[依存性注入ルール](../docs/develop/rules/dependency_injection.md)**: DIパターンとDB接続管理
- **[エラーハンドリングルール](../docs/develop/rules/error_handling.md)**: 構造化エラーと層別エラーハンドリング戦略
- **[パフォーマンスルール](../docs/develop/rules/performance.md)**: DB最適化、メモリ管理、非同期処理
- **[セキュリティルール](../docs/develop/rules/security.md)**: 入力検証、SQLインジェクション対策、権限管理
- **[テストルール](../docs/develop/rules/testing.md)**: 単体・結合テスト戦略とテストダブル使用指針
- **[ログ・監視ルール](../docs/develop/rules/logging.md)**: 構造化ログとメトリクス収集

これらのルールファイルは必ず参照し、遵守してください。

