# テスト全体設計

## 概要

本ドキュメントは、テスト基盤の共通設計（使用クレート、ディレクトリ構成、実行責務）を定義する。
個々のテストケース仕様は `docs/develop/design/testing/` 配下の機能別設計書で管理する。

## 使用クレート

### テスト実行

- 標準: `cargo test`
- 高速実行・運用改善（任意導入）: `cargo-nextest`

### 非同期テスト

- `tokio` (`#[tokio::test]`)
- `tokio-test`（非同期ユーティリティ）

### テストダブル

- `mockall`（単体テストでの依存分離）

### ログ・デバッグ

- `test-log`
- `tracing`
- `tracing-subscriber`
- `env_logger`

### 並列制御

- `serial_test`（競合するテストの直列化）

### DB関連

- `sea-orm`
- `sea-orm-migration`
- `migration` クレート（本リポジトリ内）

## フォルダ構成

```text
docs/develop/rules/testing/
├── overview.md                # テスト全体設計（本書）
├── unit_test.md               # 単体テスト方針
└── integration_test.md        # 結合テスト方針
```

```text
docs/develop/design/testing/
└── integration/
    ├── README.md
    └── {feature}.md
```

```text
src/
└── **/*.rs
    └── #[cfg(test)] mod tests { ... }   # 単体テスト（対象実装と同一ファイル）
```

```text
tests/
└── integration/
    ├── facades/                          # Facade起点の結合テスト
    ├── services/                         # 必要時のみ
    └── repository/                       # 必要時のみ
```

## 責務分離

- `overview.md`: テスト基盤の共通設計のみを記載する。
- `unit_test.md` / `integration_test.md`: テスト種別ごとの設計原則を記載する。
- `docs/develop/design/testing/integration/{feature}.md`: 機能ごとの前提データ、ケース一覧、期待結果、クリーンアップ方式を記載する。

## DB運用の共通方針

- テスト用DBは1ホスト内の複数データベースで分離する。
- `migrate up` はテスト実行単位で開始時に1回実施する。
- データクリーンアップはテストケース単位で実施する。
- ワーカー数は `DB数 / CPU / 接続上限` の最小値を上限とする。

## 個別設計書作成ルール

- 個別設計書は1機能1ファイルとする。
- 記載項目は「対象ユースケース」「前提データ」「正常系」「異常系」「クリーンアップ」「実行コマンド」を最低限含める。
- 実装コード断片は最小限にし、振る舞い仕様を中心に記述する。
