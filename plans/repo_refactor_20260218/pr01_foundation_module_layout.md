# PR01: 基盤再配置（モジュール配置統一）

## 対象機能
- 全機能共通（機能実装変更なし）

## 目的
- `repository/database` の具象実装を `infrastructure/database/repositories` に移動する土台を作る。
- 旧importを残さず、新importのみで構成できる状態を作る。

## 設計書修正
- `docs/en/developer/02_architecture/project_structure.md`
  - 新しい配置を追記
- `docs/en/developer/02_architecture/dependency_injection.md`
  - DI配線でのみ具象型を扱う規約を追記

## コード修正
- 追加:
  - `src/infrastructure/database/repositories/mod.rs`
  - `src/infrastructure/database/repositories/{schedule,recruitment,auto_recruitment,guild,master_data}/...`
- 変更:
  - `src/infrastructure/database/mod.rs` に `repositories` を追加
  - 参照側importを新パスへ置換
- 削除:
  - 旧 `src/repository/database/**` の対象実装ファイル

## テスト修正
- 影響なし（原則）
- 追加確認:
  - 旧import参照が残っていないことの確認（`rg`）

## 実行手順
1. 新ディレクトリ作成・mod定義
2. 実装ファイルを新配置へ移動
3. 参照側importを新パスへ一括置換
4. 移設済み旧ファイルを削除
5. ビルド・lintで崩れを確認

## 完了条件
- 具象実装の実体が `infrastructure/database/repositories` に移っている
- 旧 `repository/database` import が残っていない
- `cargo clippy -j 1` が通る
