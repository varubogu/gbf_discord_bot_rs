# PR05: Message/Guild/Timezone機能のリファクタリング

## 対象機能
- message解決
- guild設定/環境値
- timezone設定

## 目的
- `AppState` と `services/message` の直接実装依存を削減し、DI経由に揃える。

## 設計書修正
- `docs/en/developer/06_feature_specifications/message_resolution.md`
- `docs/en/developer/06_feature_specifications/time_zone_settings.md`
  - 実装参照先を新配置へ更新
  - AppStateでの依存保持方針を更新

## コード修正
- 移設対象:
  - `guild_message_text_repository`
  - `message_text_repository`
  - `guild_repository`
  - `guild_settings_repository`
  - `guild_environment_repository`
  - `guild_channel_repository`
  - `guild_quest_disable_repository`
  - `last_process_time_repository`
- `src/types/app_state.rs`:
  - `repository::database` 直接参照を除去
- `src/events/helpers.rs`, `src/di/container.rs`:
  - importを新配置へ更新

## テスト修正
- message serviceテスト
- guild settings/timezone関連テスト
- 主要コマンドの回帰確認

## 実行手順
1. message/guild系実装を移動
2. AppState/DIの型参照更新
3. timezone/message関連テスト実行

## 完了条件
- AppStateが旧 `repository::database::*` を参照しない
- message/timezone機能で挙動差分なし
- 対象テストが通る
