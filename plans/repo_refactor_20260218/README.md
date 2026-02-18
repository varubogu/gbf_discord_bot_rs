# Repository再編リファクタリング計画（2026-02-18）

## 目的
- `repository` をインターフェース（trait）中心に統一し、実DB実装を `infrastructure` に集約する。
- `repository/schedule` と `repository/database/schedule` の二重構造を解消し、機能ごとの粒度を揃える。
- 段階的にPRを分割しつつ、各PRで対象範囲のimportを新パスへ切り替える。

## 対象
- `src/repository/**`
- `src/infrastructure/database/**`
- `src/di/**`, `src/types/app_state.rs`, `src/main.rs`, `src/bin/cleanup.rs`
- `docs/en/developer/**`（アーキテクチャ/機能設計）

## 非対象
- 新機能追加
- DBスキーマ変更（マイグレーション）
- Discord I/F仕様変更

## To-Be構成（最終形）
```text
src/
  repository/                         # traitのみ（port）
    mod.rs
    recruitment/
    schedule/
    auto_recruitment/
    guild/
    master_data/
  infrastructure/
    database/
      connection/
      session/                        # RLS文脈設定等
      repositories/                   # SeaORM実装（adapter）
        recruitment/
        schedule/
        auto_recruitment/
        guild/
        master_data/
  di/
    repositories.rs                   # trait <- impl の配線
```

## 実施方針
- フェーズ1で設計書を先に修正し、命名規約・配置規約・依存方向を固定する。
- フェーズ2で機能別にPRを分割し、各PRで `設計書/コード/テスト` を同時更新する。
- 互換用re-exportは導入しない。対象PR内で旧importを新importへ置換し、旧パスを削除する。

## 品質ゲート（各PR共通）
- `cargo fmt`
- `cargo clippy -j 1`
- 影響範囲の単体/統合テスト
- 最終PRで `cargo test -j 1`

## PRロードマップ
- PR01: 基盤再配置（新レイアウト作成 + import切替ルール適用）
- PR02: Scheduling機能の移行
- PR03: Recruitment機能の移行
- PR04: Auto Recruitment機能の移行
- PR05: Message/Guild/Timezone機能の移行
- PR06: Spreadsheet/Startup/Environment機能の移行
- PR07: DI・起動配線切替と旧パス完全削除

各PRの詳細は同フォルダ内の `pr*.md` を参照。
