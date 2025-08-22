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

## コーディングスタイル

クリーンアーキテクチャを遵守。
そのため下記の1方向の流れは遵守してください。

プレゼンテーション層（event,commandなど）>アプリケーション層（facade,service）>データアクセス層（repository）

#### 依存性注入の原則:

- **コンストラクタインジェクション**: 依存関係はコンストラクタ（`new`メソッド）で受け取る
- **明示的な依存関係**: 各オブジェクトが何に依存しているかをコンストラクタシグネチャで明示
- **テスタビリティ**: モックオブジェクトの注入を容易にするため、traitベースのインターフェースを使用
- **ライフタイム管理**: `Arc<T>`を使用して複数の参照先での共有を安全に管理

#### 実装上の注意点

- main.rsでのDB接続初期化とその配布機構の実装が必要

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

### データアクセス層（repository）

- 保存・読み込み処理
  - トランザクションを引数として受け取る
  - DB接続は引数として受け取らない

### データアクセス層（infrastructure）

#### DB接続管理:

- **main関数でのDB接続初期化**: アプリケーション起動時（main関数実行～bot起動まで）に単一のDB接続（コネクションプール含む）を作成し、これを全体で共有する。
- **依存性注入による配布**: 作成したDB接続は依存性注入（Dependency Injection）パターンを使って各層に配布する。
- **シングルトン化**: DB接続は実質的にシングルトンとして機能するが、グローバル変数ではなく依存性注入によって管理する。
- **データアクセス層での使用制限**: DB接続の直接的な操作はデータアクセス層（infrastructure, repository）でのみ許可される。

#### 依存性注入の流れ:

1. **main.rs**: 単一のDB接続を作成
2. **プレゼンテーション層**: 共有DB接続から必要な依存関係オブジェクトを準備
3. **Facade層**: プレゼンテーション層から依存関係を受け取り、Service層に渡す
4. **Service層**: Repository層への依存関係を管理
5. **Repository層**: 注入されたDB接続を使用して実際の操作を実行

#### コンストラクタでの依存関係受け取り:

- **TransactionManager**: `new(db_service, repos)` - DB接続関連オブジェクトを外部から受け取る
- **RepositoryContainer**: `new_with_connection(db_connection)` - 共有DB接続を外部から受け取る
- **各Repository実装**: `new(db_connection)` - 共有DB接続を外部から受け取る

#### 禁止事項:

- 各層での個別DB接続作成は禁止（`DatabaseConnectionManager::new()`の直接呼び出し禁止）
- Repository層でのDB接続引数受け取りは禁止（トランザクションのみ受け取る）
- サービスロケータパターンによるグローバル状態への依存は禁止

#### トランザクション管理:

- **トランザクション抽象オブジェクトの取得**: Service層で取得されるが、実際の実行はFacade層で行う
- **DB接続の抽象化**: Service層でトランザクション取得時にDB接続を行っておき、データアクセス層以外からはDB接続タイミングを意識不要とする
- **Repository層での使用**: トランザクション抽象オブジェクトを引数として受け取り、DB接続は引数として受け取らない
- **トランザクション内接続**: トランザクションに紐づくコネクションを使って処理を実行する

## 依存性注入の具体的実装指針

### コンストラクタインジェクションの詳細ルール

- **必須**: すべての依存関係はコンストラクタ（`new`メソッド）で受け取る
- **禁止**: `new()`メソッド内での他のオブジェクトの`new()`呼び出し
- **推奨**: traitによる抽象化を用いた依存関係の定義

**実装例**:

```rust
// ✅ 正しい実装
impl TransactionManager {
  pub fn new(db_service: Arc<dyn DatabaseService>, repos: RepositoryContainer) -> Self {
    Self { db_service, repos }
  }
}

// ❌ 間違った実装
impl TransactionManager {
  pub async fn new() -> Result<Self, PoiseError> {
    let db_manager = DatabaseConnectionManager::new().await?; // 禁止
    let repos = RepositoryContainer::new().await?; // 禁止
    // ...
  }
}
```

### 各層での依存関係受け取りパターン

**プレゼンテーション層（Events/Commands）**:

- `Context`または`Data`から必要な依存関係を取得
- Facade層への依存関係注入を行う

```rust
pub async fn handle_command(ctx: PoiseContext) -> Result<(), PoiseError> {
  let tx_manager = ctx.data().transaction_manager.clone();
  let facade = BattleRecruitmentFacade::new(tx_manager);
  facade.create_new_recruitment(/* params */).await
}
```

**Facade層**:

- Service層とTransactionManagerを依存関係として受け取る
- 1つのオペレーションを担当するクラスとして実装

```rust
pub struct BattleRecruitmentFacade {
  tx_manager: Arc<TransactionManager>,
  new_service: Arc<dyn NewRecruitmentService>,
  update_service: Arc<dyn UpdateRecruitmentService>,
}

impl BattleRecruitmentFacade {
  pub fn new(
    tx_manager: Arc<TransactionManager>,
    new_service: Arc<dyn NewRecruitmentService>,
    update_service: Arc<dyn UpdateRecruitmentService>,
  ) -> Self {
    Self { tx_manager, new_service, update_service }
  }
}
```

**Service層**:

- Repository層の依存関係を受け取る
- 単一責任の原則に従う

```rust
pub struct NewRecruitmentService {
  repo: Arc<dyn BattleRecruitmentRepository>,
}

impl NewRecruitmentService {
  pub fn new(repo: Arc<dyn BattleRecruitmentRepository>) -> Self {
    Self { repo }
  }
}
```

## アーキテクチャ層間の責務詳細化

### プレゼンテーション層の詳細責務

**主要責務**:

- ユーザー入力の検証とサニタイゼーション
- facade層への適切なパラメータ変換
- エラーハンドリングと適切なレスポンス生成

**禁止事項**:

- Service層やRepository層への直接アクセス
- ビジネスロジックの実装
- データベース操作の実行

**実装パターン**:

```rust
pub async fn slash_command_handler(ctx: PoiseContext) -> Result<(), PoiseError> {
  // 1. 入力検証
  let quest_alias = validate_quest_alias(quest_alias)?;

  // 2. Facade層への依存関係注入
  let facade = create_battle_recruitment_facade(&ctx.data())?;

  // 3. Facade層の呼び出し
  let result = facade.create_new_recruitment(quest_alias, battle_type).await?;

  // 4. レスポンス生成
  ctx.say(format!("募集を作成しました: {}", result.recruitment_id)).await?;
  Ok(())
}
```

### Facade層の詳細責務

**主要責務**:

- Service層の協調（オーケストレーション）
- トランザクション境界の管理
- 複数のServiceを組み合わせた1つのユースケース実行

**禁止事項**:

- Repository層への直接アクセス
- データベース固有の操作
- 複数のユースケースを1つのクラスで処理

**実装パターン**:

```rust
impl BattleRecruitmentFacade {
  pub async fn create_new_recruitment(&self, quest_alias: &str, battle_type: BattleType) -> Result<RecruitmentResult, PoiseError> {
    self.tx_manager.execute_in_transaction(|tx_ctx| {
      Box::pin(async move {
        // 1. クエスト情報の取得
        let quest_info = self.quest_service.get_quest_info(quest_alias).await?;

        // 2. 募集の作成
        let recruitment = self.new_service.create_recruitment(&quest_info, battle_type, tx_ctx).await?;

        // 3. メッセージの送信
        let message = self.message_service.send_recruitment_message(&recruitment).await?;

        // 4. 結果の返却
        Ok(RecruitmentResult { recruitment_id: recruitment.id, message_id: message.id })
      })
    }).await
  }
}
```

### Service層の詳細責務

**主要責務**:

- 単一の業務処理実行
- ドメインルールの実装
- Repository層の呼び出し

**禁止事項**:

- 他のService層への直接依存（Facade層経由で協調）
- プレゼンテーション層特有の処理
- トランザクション管理（Facade層の責務）

### Repository層の詳細責務

**主要責務**:

- データの永続化・取得
- データベース固有の操作
- エンティティとドメインモデルの変換

**禁止事項**:

- ビジネスロジックの実装
- 他のRepository層への依存
- DB接続の直接管理（Transactionのみ受け取る）

## エラーハンドリング戦略

### 層別エラーハンドリング

- **プレゼンテーション層**: ユーザー向けエラーメッセージの生成
- **Facade層**: ビジネス例外の捕捉とログ出力
- **Service層**: ドメイン固有例外の生成
- **Repository層**: データアクセス例外の適切な変換

### エラー種別と対応方針

```rust
// エラーの階層化
pub enum ApplicationError {
  ValidationError(String),
  BusinessRuleViolation(String),
  DataAccessError(String),
  ExternalServiceError(String),
}
```

## テスト戦略

### 単体テストの指針

- **各層での単体テスト必須**
- **モックオブジェクトによる依存関係の分離**
- **テストダブルの適切な使用**

### 結合テストの指針

- **Facade層での結合テスト**
- **Repository層での実データベーステスト**

## パフォーマンス考慮事項

### DB接続管理

- **コネクションプーリングの適切な設定**
- **長時間トランザクションの回避**
- **N+1問題の防止**

### メモリ管理

- **Arc<T>を用いた適切な参照共有**
- **不要な clone() の回避**

## セキュリティ考慮事項

### 入力検証

- **プレゼンテーション層での必須入力検証**
- **SQLインジェクション対策**
- **適切なサニタイゼーション**

### 認証・認可

- **Discord権限の適切な確認**
- **サーバー固有権限の実装**

## ログ・監視

### ログレベルの使い分け

- **ERROR**: システムエラー、予期しない例外
- **WARN**: 業務例外、リトライ可能なエラー
- **INFO**: 重要な業務処理の開始・終了
- **DEBUG**: 詳細なトレース情報

### 構造化ログの実装

- **トランザクションIDによる処理追跡**
- **メトリクス収集のための適切な情報出力**

### その他ユースケース別

