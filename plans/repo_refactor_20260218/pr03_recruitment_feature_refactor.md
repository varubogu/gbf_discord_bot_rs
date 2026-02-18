# PR03: Recruitment機能のリファクタリング

## 対象機能
- マルチ募集作成/変更/キャンセル
- 参加者管理
- 通知ロール関連

## 目的
- `battle_recruitments`, `recruitment_participants`, 通知ロール系の実装依存をinfrastructureへ寄せる。

## 設計書修正
- `docs/en/developer/06_feature_specifications/multi_recruitment.md`
- `docs/en/developer/06_feature_specifications/recruitment_notifications.md`
  - repository実装配置パスと依存規約を更新

## コード修正
- 移設対象:
  - `battle_recruitments_repository`
  - `recruitment_participants_repository`
  - `all_recruitment_notification_roles_repository`
  - `quest_recruitment_notification_roles_repository`
- `src/services/recruitment/**`, `src/facades/recruitment/**`:
  - trait経由DIへ統一
- `src/di/repositories.rs`:
  - 上記実装の生成元を新パスに切替

## テスト修正
- 募集作成/更新/キャンセル系ユニットテスト
- 参加者関連統合テスト
- 通知ロール関連回帰テスト

## 実行手順
1. recruitment系実装を新パスへ移動
2. facade/serviceのimport置換
3. recruitment関連テスト実行

## 完了条件
- recruitment機能で旧 `repository/database` 具象型依存が解消
- 既存コマンド挙動に差分なし
- 対象テストが通る
