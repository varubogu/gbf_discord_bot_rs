# PR04: Auto Recruitment機能のリファクタリング

## 対象機能
- 自動募集設定
- クエスト候補/マッチング
- 希望クエスト・参加者管理

## 目的
- `auto_recruitment` 一式の実装をinfrastructureへ集約し、feature単位の配置整合性を取る。

## 設計書修正
- `docs/en/developer/06_feature_specifications/auto_recruitment.md`
- `docs/en/developer/06_feature_specifications/scheduling_feature/task_type_6_auto_recruitment_rotation.md`
- `docs/en/developer/06_feature_specifications/scheduling_feature/task_type_7_auto_matching.md`
  - 実装参照パスを更新
  - scheduler連携時の依存方向を明記

## コード修正
- 移設対象:
  - `src/repository/database/auto_recruitment/**` -> `src/infrastructure/database/repositories/auto_recruitment/**`
- `src/services/auto_recruitment/**`
- `src/facades/auto_recruitment/**`
- `src/events/interactions/components/auto_recruit_time_handler.rs`
  - import先の整理

## テスト修正
- auto_recruitment servicesのユニットテスト
- scheduler task_type_6/7 関連テスト
- 主要フローの統合テスト

## 実行手順
1. auto_recruitment実装移動
2. 参照側importを一括更新
3. task_type_6/7 の回帰テスト

## 完了条件
- auto_recruitment領域が新配置に統一
- scheduler連携コードで旧実装パス依存なし
- 対象テストが通る
