# PR07: DI・起動配線切替 + 旧パス完全削除

## 対象機能
- DIコンテナ
- main/cleanup起動経路
- 全機能横断

## 目的
- DIと起動配線を新構成へ収束させ、旧 import パスを完全に除去する。

## 設計書修正
- `docs/en/developer/02_architecture/project_structure.md`
  - 最終構成に確定
- `docs/en/developer/02_architecture/dependency_injection.md`
  - 最終配線図へ更新

## コード修正
- `src/di/repositories.rs`
- `src/main.rs`
- `src/bin/cleanup.rs`
- `src/types/app_state.rs`
  - 旧 `repository::database` 参照を完全削除
- `src/repository/database/**`
  - ディレクトリおよび残存ファイルを削除

## テスト修正
- 全体コンパイル確認
- 主要統合テスト再実行
- 最終回帰としてフルテスト

## 実行手順
1. DI配線を新実装パスへ完全移行
2. 起動エントリのimport置換
3. 旧 `repository/database` の残存ファイルを削除
4. フルチェック実行

## 検証コマンド
```bash
cargo fmt
cargo clippy -j 1
cargo test -j 1
```

## 完了条件
- `rg -n "repository::database" src` が0件
- 設計書と実装構成が一致
- フルテストが通る
