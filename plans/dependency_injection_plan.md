# 依存性注入（DI）移行計画

## 1. 概要

Discord Bot（Rust + poise）における依存性注入パターン導入の具体的な実装ステップを整理します。設計方針そのものは `docs/develop/architecture/dependency_injection.md` を参照してください。

## 2. フェーズ構成

### Phase 1: 基盤整備

**目標**: DIコンテナと `main.rs` での初期化

- `DIContainer` 構造体の作成
- `main.rs` での DB 接続初期化処理の実装
- `PoiseData` への `DIContainer` 追加
- 既存コードとの互換性確保

**成果物**

- `src/infrastructure/di_container.rs`
- 更新された `src/main.rs`
- 更新された `src/types/mod.rs`

### Phase 2: Repository 層の変更

**目標**: 依存性注入対応のコンストラクタ実装

- `RepositoryContainer::new_with_connection()` の実装
- `TransactionManager` の依存性注入対応
- 既存の `new()` メソッドを deprecated 化
- 単体テストの更新

**成果物**

- 更新された `src/infrastructure/database/container.rs`
- 更新された `src/infrastructure/database/transaction_manager.rs`

### Phase 3: プレゼンテーション・Facade 層の更新

**目標**: DI コンテナの利用開始

- コマンドハンドラでの DI コンテナ利用
- Facade 層のコンストラクタ変更
- 既存処理の動作確認
- 統合テストの実行

**成果物**

- 更新されたコマンドハンドラ
- 更新された Facade 層
- 動作確認結果

### Phase 4: 最適化・クリーンアップ

**目標**: 旧実装の削除と最適化

- deprecated メソッドの削除
- 不要な `DatabaseConnectionManager::new()` 呼び出しの削除
- パフォーマンステストの実施
- ドキュメントの更新

**成果物**

- クリーンアップされたコードベース
- パフォーマンステスト結果
- 更新されたドキュメント

## 3. テスト計画

### 3.1 ユニットテスト

- `mockall` 等によるモック活用を想定し、Facade 層はトランザクションマネージャのモックで検証する。
- Arrange-Act-Assert パターンで記述し、依存注入経路を明示する。

### 3.2 統合テスト

- テスト用 DI コンテナを構築し、SeaORM のテストデータベースで動作検証する。
- 主要コマンドの実行フローを通した疎結合化の確認を行う。

## 4. リスクと対策

- **既存機能への影響**: フェーズ分割による段階的導入と deprecated 化で互換性を確保する。各フェーズ後に回帰テストを実施。
- **パフォーマンス劣化**: `Arc<T>` の利用箇所を監視し、必要であればプロファイリングを行う。
- **デッドロック**: SeaORM のコネクションプール設定とタイムアウト値を調整し、競合を回避する。

## 5. 追跡と更新

- フェーズ完了ごとに本計画を更新し、完了状況と追加課題を記録する。
- 設計方針に変更がある場合は、必ず `docs/develop/architecture/dependency_injection.md` を先に更新し、本計画で参照箇所を最新化する。
