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
-

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
├── Cargo.lock   # Cargoインストール済みクレート
└── Cargo.toml   # Cargo設定ファイル
```

## コーディングスタイル

クリーンアーキテクチャを遵守。
そのため下記の1方向の流れは遵守してください。

プレゼンテーション層（event,commandなど）>アプリケーション層（facade,service）>データアクセス層（repository）

### プレゼンテーション層（event, command, controllerなど）

ユーザーのアクションを受け取る層。
今回でいうと下記が対象。

- イベント発生（src/events/handler, src/events/handlers/*）
- インタラクション発生（src/events/interactions/*）
- スケジュールによる起動

主に責務は以下の通り

- facade層を呼び出す
- facade層を呼び出すためのパラメータを作成（イベントなどで起こった情報を収集し、facade_parameterにする）
- facadeとは1対1の関係であり、1対多にはならない
- アプリケーション層、データアクセス層をこの層から触ってはいけない。

### アプリケーション層（facade）

クリーンアーキテクチャに従った純粋なビジネスロジックのうち１つのオペレーションを担当。
アプリケーション層の外側の層で、主にプレゼンテーション層から呼び出される。

主な責務は以下の通り

- service層の総括
- service層を呼び出す
- service層を呼び出すためのパラメータを作成（service側が特定の構造体などを求めている場合のみ）
- DBトランザクションを利用する場合、この階層でやる必要があるため、抽象化されたトランザクションの使用は許可される
- 上記以外の層については触れてはいけない

### アプリケーション層（service）

クリーンアーキテクチャに従った純粋なビジネスロジックを担当。

### データアクセス層

### その他ユースケース別

- DB接続: エントリポイント時点で作成してグローバル変数的な環境であるPoiseDataで管理する。
  ただし実際にそれを使って何かをするのが許可されるのはデータアクセス層のみ。
- DBトランザクション: 抽象化したものをFacadeでのみ使用することが許可されます。
