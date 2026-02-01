# poise/serenity依存リファクタリング 残タスク一覧

作成日: 2026-02-01
最終更新: 2026-02-01
状況: Phase 2完了 - Services層の整理、Gateway経由化の継続

## 正しいClean Architecture方針

| レイヤー | poise依存 | 役割 |
|----------|-----------|------|
| events層 | OK | Discord APIとの境界、poise型の受け取り |
| facades層 | **NG** | Gateway経由でDiscord操作、ビジネスロジック調整 |
| services層 | NG | 純粋なビジネスロジック |
| repository層 | NG | データアクセス |

**重要**: facades層でもpoise依存は許容されない。Gateway抽象化が必須。

## 完了済みファイル（Gateway抽象化済み / 移動・削除）

- [x] src/facades/auto_recruitment/category_setup_facade.rs - Gateway抽象化完了
- [x] src/facades/auto_recruitment/quest_selection_facade.rs - 未使用引数_ctx削除
- [x] src/facades/auto_recruitment/time_selection_facade.rs - 未使用引数_ctx削除
- [x] src/services/schedule/scheduler_manager.rs - Gateway抽象化完了（Http import削除）
- [x] src/services/auto_recruitment/notification_service.rs - Gateway抽象化完了
- [x] src/facades/environment.rs - コメントアウトのみ、スキップ
- [x] src/services/recruitment/reaction_handler.rs - **削除**（未使用）
- [x] src/services/permission/mod.rs - **events層に移動** (src/events/permission.rs)

## 残り修正対象ファイル

### Facades層（要Gateway抽象化）

| ファイル | poise依存 | 対応方針 |
|----------|-----------|----------|
| src/facades/recruitment/new_recruit.rs | あり | Gateway抽象化（複雑: PoiseContext, CreateReply） |
| src/facades/recruitment/button_handler.rs | あり | Gateway抽象化（複雑: ComponentInteraction） |
| src/facades/recruitment/cancel.rs | あり | Gateway抽象化（複雑: ComponentInteractionCollector） |
| src/facades/recruitment/change.rs | あり | Gateway抽象化（中程度: Message, Http） |
| src/facades/recruitment/participants.rs | あり | Gateway抽象化（中程度: Context） |

### Services層（要対応）

| ファイル | 状況 | 対応方針 |
|----------|------|----------|
| src/services/recruitment/participants.rs | poise依存あり（TODO記載済） | Discord操作関数をfacade層に移動 |
| src/services/recruitment/cancel.rs | poise依存あり（TODO記載済） | Discord操作関数をfacade層に移動 |

## Gateway抽象化パターン

### 既存のGateway実装

```rust
// src/gateway/mod.rs に統合Gateway trait定義
pub trait DiscordGateway:
    DiscordMessageGateway
    + DiscordChannelGateway
    + DiscordInteractionGateway
    + DiscordReactionGateway
    + DiscordGuildGateway
{
}

// src/gateway/impl/poise_discord_gateway.rs
pub struct PoiseDiscordGateway {
    http: Arc<Http>,
}
```

### 利用可能なGatewayメソッド

- **DiscordMessageGateway**: send_message, edit_message, delete_message, get_message, get_messages, send_reply
- **DiscordChannelGateway**: create_channel, edit_channel, delete_channel, get_channel
- **DiscordReactionGateway**: get_reaction_users, add_reaction, remove_own_reaction
- **DiscordGuildGateway**: get_member, get_roles, get_emojis
- **DiscordInteractionGateway**: defer_interaction, respond_to_interaction, edit_interaction_response（※現在未完全実装）

### 適用手順

1. facades関数の引数から`PoiseContext`を除去
2. 代わりに`&dyn DiscordGateway`を受け取る
3. events層でPoiseDiscordGatewayを作成してfacadeに渡す

## 次のステップ

1. **優先度高**: services/recruitment/cancel.rs と participants.rs の Discord操作関数をfacade層に移動
2. **優先度中**: facades/recruitment/*.rs のGateway抽象化
3. **優先度低**: ComponentInteraction対応のGateway拡張

## 技術的課題

### ComponentInteraction対応
現在のGateway設計では、ComponentInteractionCollectorやReplyHandleなどの
複雑なpoise固有機能に対応できていない。これらは以下の選択肢がある：

1. **Gatewayを拡張**: ComponentInteractionGateway traitを追加
2. **events層で処理**: インタラクション収集・応答はevents層で完結し、結果のみfacadeに渡す
3. **現状維持**: facades層にpoise依存を許容（Clean Architecture違反だが動作する）

## 進捗記録

- 2026-02-01: メモファイル作成
- 2026-02-01: Phase 1完了（シンプルなファイル6件）
- 2026-02-01: services/recruitment層の一部対応
- 2026-02-01: **方針修正** - facades層もpoise依存除去必須と確認
- 2026-02-01: **Phase 2完了**:
  - scheduler_manager.rsのpoise依存完全除去（Gateway受け取りに変更）
  - reaction_handler.rs削除（未使用）
  - permission/mod.rsをevents層に移動（26ファイルのimport更新）
  - 全体のビルド検証完了
