# プロジェクト構成

## ディレクトリ構造

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

## 主要ディレクトリの役割

### src/events/

Discord Botのイベントハンドリングを担当するプレゼンテーション層です。

- **handlers/**: Discord イベント（メッセージ受信、リアクション追加等）のハンドラー
- **interactions/**: ユーザーとのインタラクション処理
    - **command_interactions/slash/**: スラッシュコマンドの定義
    - **components/**: ボタンやセレクトメニューなどのコンポーネント処理
    - **modal/**: モーダルダイアログの処理

### src/facades/

アプリケーション層のファサードパターンを実装。複数のサービスを組み合わせて1つのユースケースを実現します。

### src/models/

ドメインモデルを定義。

- **entities/**: データベーステーブルに対応するエンティティ定義（SeaORM使用）

### src/services/

ビジネスロジックを担当するサービス層です。

- **battle_recruitment/**: マルチバトル募集機能の業務処理
- **environment/**: 環境設定の管理
- **message/**: 多言語対応メッセージ処理
- **permission/**: 権限管理処理

### src/repository/

データアクセス層。データの永続化と取得を担当します。

### docs/

プロジェクトのドキュメント類を格納。

- **develop/**: 開発者向けドキュメント
    - **design/**: 設計書
    - **rules/**: 開発ルール
- **user/**: エンドユーザー向けドキュメント

### tests/

結合テストとシステムテストを格納します。