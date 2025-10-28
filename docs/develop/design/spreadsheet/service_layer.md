# スプレッドシート機能 Service層設計書

## 概要

本設計書では、Googleスプレッドシート連携機能におけるService層の責務、Service一覧、各Serviceの詳細設計を定義します。Service層は単一の業務処理を担当し、ドメインルールを実装します。

## Service層の責務と役割

### 基本責務

- **単一業務処理**: 一つのServiceは一つの業務処理を担当
- **ドメインルール実装**: ビジネスロジックとバリデーションの実装
- **Repository層呼び出し**: データ永続化・取得のためのRepository層利用
- **トランザクション受け渡し**: Facade層から受け取ったトランザクションをRepository層に渡す
- **他Serviceへの依存禁止**: Service間の直接依存は避け、Facade層で協調

### 設計原則

```
Facade層 → Service層 → Repository層
    ↓          ↓           ↓
トランザクション開始 → 受け渡し → DB操作実行
    ↓
コミット/ロールバック
```

**重要な制約**:
- Service層はトランザクションを開始・コミット・ロールバックしない
- Service層はトランザクションを引数で受け取り、Repository層に渡すのみ
- トランザクション管理の責務はFacade層のみ

## Service一覧

### 1. GoogleAuthService - Google認証サービス

**責務**: Googleサービスアカウント認証とGoogle Sheets APIクライアント提供

**主要機能**:
- サービスアカウント認証
- 認証トークンのキャッシュ管理
- Google Sheets APIクライアントの提供
- 認証エラーのハンドリング

---

### 2. SpreadsheetReaderService - スプレッドシート読み込みサービス

**責務**: Googleスプレッドシートからのデータ読み込み

**主要機能**:
- スプレッドシートURLからの読み込み
- シート名指定での読み込み
- 読み込みデータの構造化
- Google Sheets APIエラーハンドリング

---

### 3. SpreadsheetWriterService - スプレッドシート書き込みサービス

**責務**: Googleスプレッドシートへのデータ書き込み

**主要機能**:
- スプレッドシートへのデータ書き込み
- シートの作成・更新
- バルク書き込み
- 書き込みエラーハンドリング

---

### 4. TableDefinitionService - テーブル定義解析サービス

**責務**: 「テーブル名」シートの解析とテーブル定義情報の提供

**主要機能**:
- 「テーブル名」シートの読み込み
- table_ioの解釈（in/out/in,out）
- table_typeの解釈（reference/transaction/history）
- テーブル定義の検証

---

### 5. DataConverterService - データ型変換サービス

**責務**: スプレッドシート（文字列）とPostgreSQL（型付きデータ）間のデータ変換

**主要機能**:
- PostgreSQL型 → スプレッドシート文字列変換
- スプレッドシート文字列 → PostgreSQL型変換
- NULL値の扱い
- カスタム変換ルール適用（カンマ区切り文字列等）

---

### 6. DataValidatorService - データ検証サービス

**責務**: データの妥当性検証とバリデーションエラーの報告

**主要機能**:
- 型バリデーション（Integer, DateTime, UUID等）
- 外部キー制約の検証
- NOT NULL制約の検証
- ビジネスルールの検証（guild_id一致等）

---

## 各Serviceの詳細設計

### GoogleAuthService

#### トレイト定義

```rust
#[async_trait]
pub trait GoogleAuthService: Send + Sync {
    /// サービスアカウント認証を実行し、Google Sheetsクライアントを取得
    async fn authenticate(&self) -> Result<GoogleSheetsClient, ExternalServiceError>;

    /// 認証トークンをリフレッシュ
    async fn refresh_token(&self) -> Result<(), ExternalServiceError>;

    /// 認証状態を確認
    fn is_authenticated(&self) -> bool;
}
```

#### 依存関係

- **外部ライブラリ**: `google-sheets4`, `yup-oauth2`
- **Repository層**: なし（外部API専用）

#### エラーハンドリング

- `ExternalServiceError::GoogleAuthError` - 認証失敗
- `ExternalServiceError::NetworkError` - ネットワークエラー
- `ExternalServiceError::ServiceTimeout` - タイムアウト

---

### SpreadsheetReaderService

#### トレイト定義

```rust
#[async_trait]
pub trait SpreadsheetReaderService: Send + Sync {
    /// スプレッドシートからシートデータを読み込み
    async fn read_sheet(
        &self,
        client: &GoogleSheetsClient,
        spreadsheet_id: &str,
        sheet_name: &str,
    ) -> Result<Vec<Vec<String>>, ExternalServiceError>;

    /// 複数シートを並行読み込み
    async fn read_multiple_sheets(
        &self,
        client: &GoogleSheetsClient,
        spreadsheet_id: &str,
        sheet_names: Vec<String>,
    ) -> Result<HashMap<String, Vec<Vec<String>>>, ExternalServiceError>;
}
```

#### 依存関係

- **他Service**: `GoogleAuthService` - 認証クライアント取得
- **Repository層**: なし（外部API専用）

#### エラーハンドリング

- `ExternalServiceError::GoogleSheetsApiError` - API呼び出し失敗
- `ExternalServiceError::SpreadsheetNotFound` - スプレッドシート未検出
- `ExternalServiceError::SheetNotFound` - シート未検出

---

### SpreadsheetWriterService

#### トレイト定義

```rust
#[async_trait]
pub trait SpreadsheetWriterService: Send + Sync {
    /// シートにデータを書き込み
    async fn write_sheet(
        &self,
        client: &GoogleSheetsClient,
        spreadsheet_id: &str,
        sheet_name: &str,
        data: Vec<Vec<String>>,
    ) -> Result<(), ExternalServiceError>;

    /// 複数シートに並行書き込み
    async fn write_multiple_sheets(
        &self,
        client: &GoogleSheetsClient,
        spreadsheet_id: &str,
        sheet_data: HashMap<String, Vec<Vec<String>>>,
    ) -> Result<(), ExternalServiceError>;

    /// シートをクリアしてから書き込み
    async fn clear_and_write(
        &self,
        client: &GoogleSheetsClient,
        spreadsheet_id: &str,
        sheet_name: &str,
        data: Vec<Vec<String>>,
    ) -> Result<(), ExternalServiceError>;
}
```

#### 依存関係

- **他Service**: `GoogleAuthService` - 認証クライアント取得
- **Repository層**: なし（外部API専用）

#### エラーハンドリング

- `ExternalServiceError::GoogleSheetsApiError` - API呼び出し失敗
- `ExternalServiceError::SpreadsheetNotFound` - スプレッドシート未検出
- `ExternalServiceError::ServiceTimeout` - タイムアウト

---

### TableDefinitionService

#### トレイト定義

```rust
pub struct TableDefinition {
    pub table_name_jp: String,
    pub table_name_en: String,
    pub table_io: TableIo,
    pub table_type: TableType,
}

pub enum TableIo {
    In,      // スプレッドシート → PostgreSQL
    Out,     // PostgreSQL → スプレッドシート
    InOut,   // 双方向
}

pub enum TableType {
    Reference,   // 参照系テーブル
    Transaction, // トランザクション系テーブル
    History,     // 履歴系テーブル
}

#[async_trait]
pub trait TableDefinitionService: Send + Sync {
    /// 「テーブル名」シートからテーブル定義を取得
    async fn get_table_definitions(
        &self,
        client: &GoogleSheetsClient,
        spreadsheet_id: &str,
    ) -> Result<Vec<TableDefinition>, BusinessRuleError>;

    /// 特定テーブルの定義を取得
    async fn get_table_definition(
        &self,
        client: &GoogleSheetsClient,
        spreadsheet_id: &str,
        table_name_en: &str,
    ) -> Result<TableDefinition, BusinessRuleError>;

    /// table_ioをパース
    fn parse_table_io(&self, value: &str) -> Result<TableIo, ValidationError>;

    /// table_typeをパース
    fn parse_table_type(&self, value: &str) -> Result<TableType, ValidationError>;
}
```

#### 依存関係

- **他Service**: `SpreadsheetReaderService` - シートデータ読み込み
- **Repository層**: なし

#### テーブル定義解析ロジック

**「テーブル名」シートの構造**:

- **1行目**: マッピングキー（スネークケース）
- **2行目**: 日本語の説明（プログラムでは使用しない）
- **3行目以降**: テーブル定義データ

| 1行目（キー名） | 説明（2行目の例） | 値の例（3行目以降） |
|----------------|-------------------|----------------------|
| `sheet_name`   | シート名          | `クエスト情報`       |
| `table_name`   | テーブル名        | `quests`             |
| `table_scope`  | テーブルの対象    | `global`             |
| `table_io`     | 入力・出力        | `in,out`             |
| `table_type`   | テーブル種類      | `reference`          |

- 未対応のキーは無視される
- `table_scope` は将来拡張用（未使用の場合は空でも可）
- キー名でマッピングするため列順は不問、追加列も許容する

**table_ioの解釈**:
- `"in"` → `TableIo::In` - 読み込み専用（load操作のみ）
- `"out"` → `TableIo::Out` - 書き込み専用（push操作のみ）
- `"in,out"` / `"out,in"` / `"both"` → `TableIo::Both` - 双方向

**table_typeの解釈**:
- `"reference"` → `TableType::Reference` - マスターデータ
- `"transaction"` → `TableType::Transaction` - トランザクションデータ
- `"history"` → `TableType::History` - 履歴データ

#### エラーハンドリング

- `ValidationError::InvalidFormat` - table_io/table_typeの値が不正
- `BusinessRuleError::TableDefinitionError` - テーブル定義の整合性エラー
- `ExternalServiceError::SheetNotFound` - 「テーブル名」シートが存在しない

---

### DataConverterService

#### トレイト定義

```rust
#[async_trait]
pub trait DataConverterService: Send + Sync {
    /// PostgreSQL型 → スプレッドシート文字列変換
    fn to_spreadsheet_string(&self, value: &sea_orm::Value) -> String;

    /// スプレッドシート文字列 → Integer変換
    fn parse_integer(&self, value: &str, field: &str) -> Result<Option<i32>, ValidationError>;

    /// スプレッドシート文字列 → BigInteger変換
    fn parse_bigint(&self, value: &str, field: &str) -> Result<Option<i64>, ValidationError>;

    /// スプレッドシート文字列 → String変換
    fn parse_string(
        &self,
        value: &str,
        field: &str,
        nullable: bool,
    ) -> Result<Option<String>, ValidationError>;

    /// スプレッドシート文字列 → Boolean変換
    fn parse_boolean(&self, value: &str, field: &str) -> Result<bool, ValidationError>;

    /// スプレッドシート文字列 → DateTime変換
    fn parse_datetime(&self, value: &str, field: &str) -> Result<DateTime<Utc>, ValidationError>;

    /// スプレッドシート文字列 → UUID変換
    fn parse_uuid(&self, value: &str, field: &str) -> Result<Uuid, ValidationError>;

    /// カンマ区切り文字列 → 整数配列変換
    fn parse_comma_separated_integers(&self, value: &str) -> Vec<i32>;

    /// guild_idの自動付与
    fn apply_guild_id(
        &self,
        row_data: &mut HashMap<String, String>,
        guild_id: i64,
        has_guild_id_column: bool,
    ) -> Result<(), ValidationError>;
}
```

#### 依存関係

- **Repository層**: なし（純粋な変換処理）

#### 主要変換ルール

**NULL値の扱い**:
- 空文字列 → `NULL`（NULLABLEカラムの場合）
- 空文字列 → デフォルト値（NOT NULLカラムの場合）

**日時フォーマット対応**:
1. RFC3339: `"2025-01-15T12:00:00+09:00"`
2. ISO8601: `"2025-01-15T12:00:00Z"`
3. スペース区切り: `"2025-01-15 12:00:00"`
4. 日付のみ: `"2025-01-15"` → `00:00:00`として解釈

**Boolean値の柔軟な対応**:
- `true`: `"true"`, `"1"`, `"yes"`, `"t"`
- `false`: `"false"`, `"0"`, `"no"`, `"f"`, `""`（空文字列）

#### エラーハンドリング

- `ValidationError::TypeConversionError` - 型変換失敗
- `ValidationError::DateTimeFormatError` - 日時形式エラー
- `ValidationError::UuidFormatError` - UUID形式エラー
- `ValidationError::RequiredFieldMissing` - 必須フィールド未入力

---

### DataValidatorService

#### トレイト定義

```rust
#[async_trait]
pub trait DataValidatorService: Send + Sync {
    /// 外部キー制約の検証
    async fn validate_foreign_key(
        &self,
        txn: &DatabaseTransaction,
        field: &str,
        value: i64,
        reference_table: &str,
        reference_column: &str,
    ) -> Result<(), ValidationError>;

    /// NOT NULL制約の検証
    fn validate_not_null(
        &self,
        field: &str,
        value: Option<&str>,
    ) -> Result<(), ValidationError>;

    /// ギルドID一致検証
    fn validate_guild_id(
        &self,
        expected_guild_id: i64,
        actual_guild_id: i64,
    ) -> Result<(), BusinessRuleError>;

    /// 値の範囲検証
    fn validate_range<T: PartialOrd + Display>(
        &self,
        field: &str,
        value: T,
        min: T,
        max: T,
    ) -> Result<(), ValidationError>;

    /// バルク検証（複数行の検証）
    fn validate_rows(
        &self,
        rows: Vec<HashMap<String, String>>,
        table_def: &TableDefinition,
    ) -> Vec<ValidationError>;
}
```

#### 依存関係

- **Repository層**: 外部キー検証時にRepositoryを利用（参照先テーブルの存在確認）

#### 外部キー制約検証フロー

```
1. フィールド値を取得
2. 参照先テーブル・カラムを特定
3. Repository層で存在確認クエリ実行
4. 存在しない場合はValidationError::ForeignKeyViolation
```

**例**: `quests.default_battle_type` → `battle_types.type_id`

```rust
// 概念的な実装
async fn validate_foreign_key(...) -> Result<(), ValidationError> {
    let exists = repository
        .exists_by_column(txn, reference_table, reference_column, value)
        .await?;

    if !exists {
        return Err(ValidationError::ForeignKeyViolation {
            field: field.to_string(),
            reference_table: reference_table.to_string(),
            value: value.to_string(),
        });
    }

    Ok(())
}
```

#### エラーハンドリング

- `ValidationError::RequiredFieldMissing` - 必須フィールド未入力
- `ValidationError::ValueOutOfRange` - 値が範囲外
- `ValidationError::ForeignKeyViolation` - 外部キー制約違反
- `BusinessRuleError::GuildIdMismatch` - ギルドID不一致

---

## Google認証フロー詳細

### サービスアカウント認証の流れ

```
1. 環境変数からサービスアカウントキーファイルパスを取得
   ↓
2. サービスアカウントキーファイルを読み込み
   ↓
3. yup-oauth2を使用してサービスアカウント認証実行
   ↓
4. アクセストークン取得
   ↓
5. google-sheets4のHubクライアント作成
   ↓
6. 認証済みクライアントを返却
```

### google-sheets4 + yup-oauth2の使用方法

**認証クライアントの作成**:

```rust
use google_sheets4::{hyper, hyper_rustls, oauth2, Sheets};
use yup_oauth2::ServiceAccountAuthenticator;

pub async fn create_google_sheets_client(
    service_account_key_path: &str,
) -> Result<Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>, ExternalServiceError> {
    // サービスアカウントキーの読み込み
    let service_account_key = yup_oauth2::read_service_account_key(service_account_key_path)
        .await
        .map_err(|e| ExternalServiceError::GoogleAuthError {
            message: format!("サービスアカウントキーの読み込みに失敗: {}", e),
        })?;

    // Authenticatorの作成
    let auth = ServiceAccountAuthenticator::builder(service_account_key)
        .build()
        .await
        .map_err(|e| ExternalServiceError::GoogleAuthError {
            message: format!("認証情報の作成に失敗: {}", e),
        })?;

    // HTTPSクライアントの作成
    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .build();

    let client = hyper::Client::builder().build(https_connector);

    // Google Sheets Hubの作成
    let hub = Sheets::new(client, auth);

    Ok(hub)
}
```

### 認証トークンのキャッシュ

**キャッシュ戦略**:
- yup-oauth2が内部的にトークンをキャッシュ
- トークンの有効期限切れ時は自動リフレッシュ
- Service層ではキャッシュを意識せず、透過的に利用可能

**実装イメージ**:

```rust
pub struct GoogleAuthServiceImpl {
    client_cache: Arc<RwLock<Option<Sheets>>>,
    service_account_key_path: String,
}

impl GoogleAuthServiceImpl {
    async fn get_or_create_client(&self) -> Result<Sheets, ExternalServiceError> {
        // キャッシュ確認
        {
            let cache = self.client_cache.read().await;
            if let Some(client) = cache.as_ref() {
                return Ok(client.clone());
            }
        }

        // キャッシュがない場合は新規作成
        let client = self.create_new_client().await?;

        // キャッシュに保存
        {
            let mut cache = self.client_cache.write().await;
            *cache = Some(client.clone());
        }

        Ok(client)
    }
}
```

### 必要なGoogleスコープ

```rust
const GOOGLE_SHEETS_SCOPE: &str = "https://www.googleapis.com/auth/spreadsheets";
```

**スコープの説明**:
- `spreadsheets` - スプレッドシートの読み書き権限
- GCPコンソールでサービスアカウントに付与が必要

---

## 依存性注入

### AppStateへの登録方法

**AppState定義**:

```rust
pub struct AppState {
    pub db_connection: Arc<DatabaseConnection>,
    pub google_auth_service: Arc<dyn GoogleAuthService>,
    pub spreadsheet_reader_service: Arc<dyn SpreadsheetReaderService>,
    pub spreadsheet_writer_service: Arc<dyn SpreadsheetWriterService>,
    pub table_definition_service: Arc<dyn TableDefinitionService>,
    pub data_converter_service: Arc<dyn DataConverterService>,
    pub data_validator_service: Arc<dyn DataValidatorService>,
}
```

**main.rsでの初期化**:

```rust
#[tokio::main]
async fn main() {
    // DB接続...

    // Service層の初期化
    let google_auth_service = Arc::new(GoogleAuthServiceImpl::new(
        env::var("GOOGLE_SERVICE_ACCOUNT_KEY_FILE").expect("GOOGLE_SERVICE_ACCOUNT_KEY_FILE not set"),
    ));

    let spreadsheet_reader_service = Arc::new(SpreadsheetReaderServiceImpl::new(
        google_auth_service.clone(),
    ));

    let spreadsheet_writer_service = Arc::new(SpreadsheetWriterServiceImpl::new(
        google_auth_service.clone(),
    ));

    let table_definition_service = Arc::new(TableDefinitionServiceImpl::new(
        spreadsheet_reader_service.clone(),
    ));

    let data_converter_service = Arc::new(DataConverterServiceImpl::new());

    let data_validator_service = Arc::new(DataValidatorServiceImpl::new());

    // AppState作成
    let app_state = Arc::new(AppState {
        db_connection,
        google_auth_service,
        spreadsheet_reader_service,
        spreadsheet_writer_service,
        table_definition_service,
        data_converter_service,
        data_validator_service,
    });

    // PoiseData作成...
}
```

### Facade層での利用

```rust
pub struct SpreadsheetLoadFacade<'a> {
    app_state: &'a AppState,
}

impl<'a> SpreadsheetLoadFacade<'a> {
    pub async fn load_from_spreadsheet(
        &self,
        spreadsheet_url: &str,
        guild_id: Option<i64>,
    ) -> Result<(), FacadeError> {
        // Service層へのアクセス
        let google_auth_service = &self.app_state.google_auth_service;
        let table_definition_service = &self.app_state.table_definition_service;
        let data_converter_service = &self.app_state.data_converter_service;

        // 認証
        let client = google_auth_service.authenticate().await?;

        // テーブル定義取得
        let table_defs = table_definition_service
            .get_table_definitions(&client, spreadsheet_id)
            .await?;

        // データ変換...
    }
}
```

---

## エラーハンドリングパターン

### Service層でのエラー伝播

```rust
// 良い例: エラーを適切に分類して返す
pub async fn read_sheet(...) -> Result<Vec<Vec<String>>, ExternalServiceError> {
    let response = api_call().await.map_err(|e| {
        ExternalServiceError::GoogleSheetsApiError {
            message: format!("シートの読み込みに失敗: {}", e),
        }
    })?;

    if response.is_empty() {
        return Err(ExternalServiceError::SheetNotFound {
            sheet_name: sheet_name.to_string(),
            spreadsheet_id: spreadsheet_id.to_string(),
        });
    }

    Ok(response)
}
```

### ログ出力

```rust
use tracing::{error, warn, info};

// Service層でのログ出力例
impl SpreadsheetReaderServiceImpl {
    pub async fn read_sheet(...) -> Result<Vec<Vec<String>>, ExternalServiceError> {
        info!(
            spreadsheet_id = %spreadsheet_id,
            sheet_name = %sheet_name,
            "スプレッドシートからシートを読み込み開始"
        );

        match self.do_read_sheet(spreadsheet_id, sheet_name).await {
            Ok(data) => {
                info!(
                    spreadsheet_id = %spreadsheet_id,
                    sheet_name = %sheet_name,
                    row_count = data.len(),
                    "スプレッドシートの読み込み成功"
                );
                Ok(data)
            }
            Err(e) => {
                error!(
                    error = %e,
                    spreadsheet_id = %spreadsheet_id,
                    sheet_name = %sheet_name,
                    "スプレッドシートの読み込み失敗"
                );
                Err(e)
            }
        }
    }
}
```

---

## テスト戦略

### Serviceのモック実装

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        pub GoogleAuthServiceMock {}

        #[async_trait]
        impl GoogleAuthService for GoogleAuthServiceMock {
            async fn authenticate(&self) -> Result<GoogleSheetsClient, ExternalServiceError>;
            async fn refresh_token(&self) -> Result<(), ExternalServiceError>;
            fn is_authenticated(&self) -> bool;
        }
    }

    #[tokio::test]
    async fn test_spreadsheet_reader_with_mock() {
        let mut mock_auth = MockGoogleAuthServiceMock::new();
        mock_auth
            .expect_authenticate()
            .times(1)
            .returning(|| Ok(create_mock_client()));

        let reader = SpreadsheetReaderServiceImpl::new(Arc::new(mock_auth));
        let result = reader.read_sheet(...).await;

        assert!(result.is_ok());
    }
}
```

---

## パフォーマンス考慮事項

### 並行処理の活用

**複数シートの並行読み込み**:

```rust
use futures::future::try_join_all;

pub async fn read_multiple_sheets(...) -> Result<HashMap<String, Vec<Vec<String>>>, ExternalServiceError> {
    let futures = sheet_names
        .into_iter()
        .map(|name| self.read_sheet(client, spreadsheet_id, &name));

    let results = try_join_all(futures).await?;

    let sheet_data = sheet_names
        .into_iter()
        .zip(results)
        .collect();

    Ok(sheet_data)
}
```

### キャッシュの活用

- 認証クライアントのキャッシュ
- テーブル定義のキャッシュ（同一スプレッドシート内で複数回読み込む場合）

---

## 関連ドキュメント

### 機能概要
- [Googleスプレッドシート連携機能](../../features/google_spreadsheet.md)

### アーキテクチャ
- [データフロー設計](../../architecture/spreadsheet/data_flow.md)
- [依存性注入設計](../../architecture/dependency_injection.md)

### 詳細設計
- [データ変換仕様](data_conversion.md)
- [エラー型定義](../error_types.md)

### データベース
- [データベース接続・トランザクション管理](../database/db_connection_transaction.md)

### ルール
- [エラーハンドリングルール](../../rules/error_handling.md)
- [ロギングルール](../../rules/logging.md)
