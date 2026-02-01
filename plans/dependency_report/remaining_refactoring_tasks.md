# poise/serenity依存リファクタリング 残タスク一覧

作成日: 2026-02-01
最終更新: 2026-02-01
状況: 方針修正 - facades層もpoise依存除去必須

## 正しいClean Architecture方針

| レイヤー | poise依存 | 役割 |
|----------|-----------|------|
| events層 | OK | Discord APIとの境界、poise型の受け取り |
| facades層 | **NG** | Gateway経由でDiscord操作、ビジネスロジック調整 |
| services層 | NG | 純粋なビジネスロジック |
| repository層 | NG | データアクセス |

**重要**: facades層でもpoise依存は許容されない。Gateway抽象化が必須。

## 完了済みファイル（Gateway抽象化済み）

- [x] src/facades/auto_recruitment/category_setup_facade.rs - Gateway抽象化完了
- [x] src/facades/auto_recruitment/quest_selection_facade.rs - 未使用引数_ctx削除
- [x] src/facades/auto_recruitment/time_selection_facade.rs - 未使用引数_ctx削除
- [x] src/services/schedule/scheduler_manager.rs - Gateway抽象化完了
- [x] src/services/auto_recruitment/notification_service.rs - Gateway抽象化完了
- [x] src/facades/environment.rs - コメントアウトのみ、スキップ

## 残り修正対象ファイル

### Facades層（要Gateway抽象化）

| ファイル | poise依存 | 対応方針 |
|----------|-----------|----------|
| src/facades/recruitment/new_recruit.rs | あり | Gateway抽象化 |
| src/facades/recruitment/button_handler.rs | あり | Gateway抽象化 |
| src/facades/recruitment/cancel.rs | あり | Gateway抽象化 |
| src/facades/recruitment/change.rs | あり | Gateway抽象化 |
| src/facades/recruitment/participants.rs | あり | Gateway抽象化 |
| src/facades/recruitment/recruitment_schedule_list.rs | あり | Gateway抽象化 |
| src/facades/recruitment/quest_list.rs | あり | Gateway抽象化 |
| src/facades/recruitment/battle_style_list.rs | あり | Gateway抽象化 |
| src/facades/recruitment/role_management.rs | あり | Gateway抽象化 |
| src/facades/spreadsheet/global_load_facade.rs | あり | Gateway抽象化 |
| src/facades/spreadsheet/global_push_facade.rs | あり | Gateway抽象化 |
| src/facades/spreadsheet/guild_load_facade.rs | あり | Gateway抽象化 |
| src/facades/spreadsheet/guild_push_facade.rs | あり | Gateway抽象化 |

### Services層（要対応）

| ファイル | 状況 | 対応方針 |
|----------|------|----------|
| src/services/recruitment/participants.rs | poise依存あり | Gateway抽象化 or facade移動 |
| src/services/recruitment/cancel.rs | poise依存あり | Gateway抽象化 or facade移動 |
| src/services/recruitment/reaction_handler.rs | 未使用 | 削除検討 |
| src/services/permission/mod.rs | poise依存あり | events層に移動 |

## Gateway抽象化パターン

### 既存のGateway実装

```rust
// src/gateways/discord_gateway.rs
#[async_trait]
pub trait DiscordGateway: Send + Sync {
    async fn send_message(&self, channel_id: DiscordChannelId, content: &MessageContent) -> Result<DiscordMessageId>;
    async fn edit_message(&self, channel_id: DiscordChannelId, message_id: DiscordMessageId, content: &MessageContent) -> Result<()>;
    // ... その他のDiscord操作
}

// src/gateways/poise_discord_gateway.rs
pub struct PoiseDiscordGateway<'a> {
    ctx: PoiseContext<'a>,
}
```

### 適用手順

1. facades関数の引数から`PoiseContext`を除去
2. 代わりに`&dyn DiscordGateway`を受け取る
3. events層でPoiseDiscordGatewayを作成してfacadeに渡す

## 次のステップ

1. Gateway traitに必要なメソッドを追加
2. 優先度の高いfacadeから順次Gateway抽象化を適用
3. services層のpoise依存も完全除去

## 進捗記録

- 2026-02-01: メモファイル作成
- 2026-02-01: Phase 1完了（シンプルなファイル6件）
- 2026-02-01: services/recruitment層の一部対応
- 2026-02-01: **方針修正** - facades層もpoise依存除去必須と確認
