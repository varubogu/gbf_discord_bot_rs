# Poise/Serenity依存調査報告書

## 概要

facade、service、repository、infrastructureレイヤーにおけるpoise/serenityへの依存を調査した結果、**Clean Architectureの依存ルールに重大な違反**が発見されました。

### 調査結果サマリー

| レイヤー | 違反ファイル数 | 状態 |
|---------|--------------|------|
| Facade | 13ファイル | 重大な違反 |
| Service | 35ファイル | 広範な違反 |
| Repository | 1ファイル | 型レベルでの違反 |
| Infrastructure | 0ファイル | クリーン（違反なし） |

---

## Facadeレイヤーの違反一覧

### 1. category_setup_facade.rs

**ファイル:** `src/facades/auto_recruitment/category_setup_facade.rs`

**インポート:**
```rust
use poise::serenity_prelude::{
    ButtonStyle, ChannelId, ChannelType, Context, CreateActionRow,
    CreateButton, CreateChannel, CreateMessage, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, EditChannel, GuildId, Http
};
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `Context` | Discordコンテキスト（パラメータ） |
| `ctx.http()` | HTTPクライアント取得 |
| `channel_id.send_message(http, message)` | メッセージ送信 |
| `channel_id.delete(http)` | チャンネル削除 |
| `channel_id.delete_message(http, message_id)` | メッセージ削除 |
| `channel_id.to_channel(http)` | チャンネルオブジェクト取得 |
| `guild_id.create_channel(http, builder)` | チャンネル作成 |
| `CreateMessage`, `CreateSelectMenu` | メッセージ・UIビルダー |
| `CreateButton`, `CreateActionRow` | ボタンUIビルダー |
| `ButtonStyle` | ボタンスタイル列挙型 |
| `EditChannel` | チャンネル編集ビルダー |

---

### 2. change.rs（recruitment）

**ファイル:** `src/facades/recruitment/change.rs`

**インポート:**
```rust
use poise::serenity_prelude::Message;
use poise::serenity_prelude::{ChannelId, MessageId};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter, EditMessage};
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `Message` | Discordメッセージオブジェクト |
| `message.channel_id.get()` | チャンネルID抽出 |
| `message.id.get()` | メッセージID抽出 |
| `channel_id_obj.edit_message(http, message_id_obj, edit_message)` | メッセージ更新 |
| `channel_id_obj.reaction_users(http, ...)` | リアクションユーザー取得 |
| `message.components` | メッセージコンポーネントアクセス |
| `CreateEmbed`, `CreateEmbedFooter` | Embedビルダー |
| `EditMessage` | メッセージ編集ビルダー |

---

### 3. cancel.rs（recruitment）

**ファイル:** `src/facades/recruitment/cancel.rs`

**インポート:**
```rust
use poise::ReplyHandle;
use poise::serenity_prelude::{
    ButtonStyle, ChannelId, ComponentInteraction, ComponentInteractionCollector,
    Context, CreateActionRow, CreateButton, CreateMessage,
    EditInteractionResponse, Message, MessageId
};
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `ComponentInteractionCollector::new(ctx.serenity_context())` | インタラクションコレクター作成 |
| `interaction.defer(ctx.http())` | インタラクション遅延 |
| `channel.edit_message(ctx.http(), message_id, edit_message)` | メッセージ編集 |
| `channel.send_message(ctx.http(), message)` | メッセージ送信 |
| `interaction.edit_response(ctx.http(), ...)` | インタラクション応答 |
| `ctx.guild_id()` | ギルドID取得 |
| `ctx.http()` | HTTPクライアント取得 |
| `ctx.locale()` | ロケール取得 |
| `ReplyHandle` | Poiseリプライハンドル |

---

### 4. button_handler.rs

**ファイル:** `src/facades/recruitment/button_handler.rs`

**インポート:**
```rust
use poise::serenity_prelude::{ComponentInteraction, Context};
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `Context` | Discordコンテキスト |
| `ComponentInteraction` | コンポーネントインタラクション |
| Discordメッセージ編集・応答API | ボタンハンドリング |

---

### 5. participants.rs

**ファイル:** `src/facades/recruitment/participants.rs`

**インポート:**
```rust
use poise::serenity_prelude::Context;
```

---

### 6. guild_settings_facade.rs

**ファイル:** `src/facades/guild_settings/guild_settings_facade.rs`

**インポート:**
```rust
use poise::serenity_prelude::AutocompleteChoice;
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `AutocompleteChoice` | オートコンプリート選択肢 |
| `Vec<AutocompleteChoice>` を返却 | スラッシュコマンドの補完用 |

---

### 7. その他のFacade

| ファイル | 使用している型 |
|---------|--------------|
| `auto_recruitment/quest_selection_facade.rs` | `Context`（パラメータ、未使用） |
| `auto_recruitment/time_selection_facade.rs` | `Context`（パラメータ、未使用） |
| `recruitment/quest_list.rs` | `Vec<AutocompleteChoice>` |
| `recruitment/battle_style_list.rs` | `Vec<AutocompleteChoice>` |
| `recruitment/recruitment_schedule_list.rs` | `Vec<AutocompleteChoice>` |
| `channel/channel_management_facade.rs` | `Vec<AutocompleteChoice>` |

---

## Serviceレイヤーの違反一覧

### 1. notification_service.rs（schedule）

**ファイル:** `src/services/schedule/notification_service.rs`

**インポート:**
```rust
use poise::serenity_prelude::{ChannelId, CreateMessage, Http, MessageId};
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `Http` | HTTPクライアント（構造体フィールド） |
| `channel_id.send_message(&self.http, message)` | 通知送信 |
| `ChannelId`, `MessageId` | Discord識別子 |
| `CreateMessage` | メッセージビルダー |

**問題点:** ServiceがHTTPクライアントを保持し、直接Discord APIを呼び出している

---

### 2. new.rs（recruitment）

**ファイル:** `src/services/recruitment/new.rs`

**インポート:**
```rust
use poise::serenity_prelude::ButtonStyle;
use poise::serenity_prelude::ReactionType;
use poise::serenity_prelude::all::{
    CreateActionRow, CreateButton, CreateEmbed, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption
};
use poise::CreateReply;
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `CreateEmbed` | Embedビルダー |
| `ReactionType` | リアクション絵文字タイプ |
| `ButtonStyle` | ボタンスタイル |
| `CreateButton`, `CreateActionRow` | ボタンビルダー |
| `CreateSelectMenu`, `CreateSelectMenuOption` | セレクトメニュービルダー |
| `CreateReply` | Poiseリプライビルダー |

**問題点:** サービスにDiscord UIビルダーが含まれている

---

### 3. participants.rs（recruitment）

**ファイル:** `src/services/recruitment/participants.rs`

**インポート:**
```rust
use poise::serenity_prelude::all::{
    ChannelId, Context, CreateEmbed, EditMessage, MessageId, ReactionType
};
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `channel.message(ctx.http, message_id)` | メッセージ取得 |
| `message.reaction_users(ctx.http, ...)` | リアクションユーザー取得 |
| `message.edit(ctx.http, edit_message)` | メッセージ編集 |
| `channel.messages(ctx.http, GetMessages::new().limit(100))` | チャンネルメッセージ取得 |

**問題点:** サービスメソッドがContextを受け取り、直接Discord APIを呼び出している

---

### 4. cancel.rs（recruitment）

**ファイル:** `src/services/recruitment/cancel.rs`

**インポート:**
```rust
use poise::serenity_prelude::all::{
    ChannelId, Context, CreateEmbed, EditMessage, Message, Reaction, User
};
```

---

### 5. start.rs（recruitment）

**ファイル:** `src/services/recruitment/start.rs`

**インポート:**
```rust
use poise::serenity_prelude::all::{ChannelId, Context, MessageId};
```

---

### 6. update.rs（recruitment）

**ファイル:** `src/services/recruitment/update.rs`

**インポート:**
```rust
use poise::serenity_prelude::all::{
    ChannelId, Context, CreateEmbed, EditMessage, Message
};
```

---

### 7. auto_matching_task_executor.rs

**ファイル:** `src/services/schedule/auto_matching_task_executor.rs`

**インポート:**
```rust
use poise::serenity_prelude::{ChannelId, CreateEmbed, CreateMessage, Http};
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `Http` | HTTPクライアント（パラメータ） |
| メッセージ作成・送信 | 自動マッチング通知 |

---

### 8. auto_recruitment_rotation_task_executor.rs

**ファイル:** `src/services/schedule/auto_recruitment_rotation_task_executor.rs`

**インポート:**
```rust
use poise::serenity_prelude::{ChannelId, EditChannel, Http};
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `channel_id.edit(http, EditChannel::new()...)` | チャンネル編集 |

---

### 9. dismissal_task_executor.rs

**ファイル:** `src/services/schedule/dismissal_task_executor.rs`

**インポート:**
```rust
use poise::serenity_prelude::{ChannelId, EditMessage, Http, MessageId};
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `channel_id.message(http, message_id)` | メッセージ取得 |
| `message.edit(http, edit_message)` | メッセージ編集 |

---

### 10. dissolution_task_executor.rs

**ファイル:** `src/services/schedule/dissolution_task_executor.rs`

**インポート:**
```rust
use poise::serenity_prelude::{ChannelId, EditMessage, Http, MessageId};
```

---

### 11. scheduler_manager.rs

**ファイル:** `src/services/schedule/scheduler_manager.rs`

**インポート:**
```rust
use poise::serenity_prelude::Http;
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `Arc<Http>` | HTTPクライアント（構造体フィールド） |

---

### 12. notification_service.rs（auto_recruitment）

**ファイル:** `src/services/auto_recruitment/notification_service.rs`

**インポート:**
```rust
use poise::serenity_prelude::{
    ChannelId, CreateMessage, CreateEmbed, Http, MessageId, EditMessage
};
```

---

### 13. UIビルダーサービス群

| ファイル | 使用している型 |
|---------|--------------|
| `auto_recruitment/ui/quest_message_builder.rs` | `CreateActionRow`, `CreateButton`, `CreateSelectMenu`, `CreateMessage` など |
| `auto_recruitment/ui/quest_select_menu.rs` | `CreateSelectMenu`, `CreateSelectMenuKind`, `CreateSelectMenuOption` |
| `auto_recruitment/ui/time_select_menu.rs` | `CreateSelectMenu`, `CreateSelectMenuKind`, `CreateSelectMenuOption` |

---

### 14. guild_environment_service.rs

**ファイル:** `src/services/guild_environment_service.rs`

**インポート:**
```rust
use poise::serenity_prelude::{Emoji, GuildId, Http};
```

---

### 15. permission/mod.rs

**ファイル:** `src/services/permission/mod.rs`

**インポート:**
```rust
use poise::serenity_prelude::all::Member;
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `Member` | Discordメンバーオブジェクト |
| `ctx.guild_id()` | ギルドID取得 |
| `ctx.http()` | HTTPクライアント取得 |
| `ctx.get_guild_roles()` | ギルドロール取得 |

---

### 16. recruitment_creation_service.rs

**ファイル:** `src/services/recruitment/recruitment_creation_service.rs`

**インポート:**
```rust
use poise::serenity_prelude::{CreateEmbed, CreateMessage, Http};
```

---

### 17. timezone_service.rs

**ファイル:** `src/services/timezone_service.rs`

**インポート:**
```rust
use poise::serenity_prelude::AutocompleteChoice;
```

**問題点:** ビジネスロジックがDiscord固有の型を返している

---

### 18. channel_type_query_service.rs

**ファイル:** `src/services/channel/channel_type_query_service.rs`

**インポート:**
```rust
use poise::serenity_prelude::AutocompleteChoice;
```

---

### 19. schedule_display_service.rs

**ファイル:** `src/services/recruitment/schedule/schedule_display_service.rs`

**インポート:**
```rust
use poise::serenity_prelude::{AutocompleteChoice, CreateEmbed, CreateEmbedFooter};
```

---

## Repositoryレイヤーの違反一覧

### battle_recruitments_repository.rs

**ファイル:** `src/repository/battle_recruitments_repository.rs`

**インポート:**
```rust
use poise::serenity_prelude::MessageId;
```

**使用しているAPI:**
| 型/メソッド | 用途 |
|------------|------|
| `MessageId` | メソッドパラメータ型 |
| `set_end_message(..., message_id: MessageId)` | 終了メッセージ設定 |
| `set_canceled_with_txn(..., message_id: MessageId)` | キャンセル設定 |

**問題点:** Repositoryトレイト定義にDiscord固有の型が含まれている。`u64`などのプリミティブ型を使うべき

---

## Infrastructureレイヤー

**状態: クリーン**

Infrastructureレイヤーにはpoise/serenityへの依存は発見されませんでした。

---

## 発見された依存パターン

### パターン1: Httpクライアントの受け渡し

複数のタスクエグゼキューターサービスが`Arc<Http>`をパラメータとして受け取り、直接Discord APIを呼び出している。

```rust
// 現在の問題のあるパターン
pub async fn execute(&self, http: Arc<Http>) {
    channel_id.send_message(&http, message).await
}
```

**該当ファイル:**
- `auto_matching_task_executor.rs`
- `auto_recruitment_rotation_task_executor.rs`
- `dismissal_task_executor.rs`
- `dissolution_task_executor.rs`
- `scheduler_manager.rs`

---

### パターン2: Contextパラメータの伝播

FacadeやServiceが`Context`をパラメータとして受け取り、内部で以下を抽出している：

```rust
// 現在の問題のあるパターン
pub async fn some_method(ctx: Context<'_, Data, Error>) {
    let guild_id = ctx.guild_id();
    let http = ctx.http();
    let locale = ctx.locale();
}
```

---

### パターン3: データ構造内のUIビルダー型

`RecruitmentData`構造体に`CreateEmbed`や`Vec<ReactionType>`が含まれている。

```rust
// 現在の問題のあるパターン
pub struct RecruitmentData {
    pub embed: CreateEmbed,  // Discord固有の型
    pub reactions: Vec<ReactionType>,  // Discord固有の型
}
```

---

### パターン4: Discordオブジェクトを直接パラメータに

メソッドが`Message`、`ComponentInteraction`、`Reaction`を直接受け取っている。

```rust
// 現在の問題のあるパターン
pub async fn handle_message(message: Message) { ... }
pub async fn handle_interaction(interaction: ComponentInteraction) { ... }
```

---

### パターン5: AutocompleteChoiceの漏洩

複数のサービスが`Vec<AutocompleteChoice>`を直接返している。

```rust
// 現在の問題のあるパターン
pub fn get_timezones() -> Vec<AutocompleteChoice> { ... }
// あるべき姿
pub fn get_timezones() -> Vec<(String, String)> { ... }
```

---

## 依存方向の違反まとめ

| レイヤー | 依存すべき対象 | 実際の依存先 | 違反 |
|---------|---------------|------------|------|
| Facade | Services, Repository | **Poise, Serenity**, Services, Repository | **あり** |
| Service | Repository, Models | **Poise, Serenity**, Repository, Models | **あり** |
| Repository | Models, Database | **Poise/Serenity**（型レベル）, Models | **あり** |
| Infrastructure | Database | Database | **なし** |

---

## 主要な問題点まとめ

1. **HttpクライアントがServiceに渡されている** - タスクエグゼキューター群で顕著
2. **データ構造にUIビルダーが含まれている** - `RecruitmentData`に`CreateEmbed`
3. **Contextがビジネスロジック層まで伝播** - Facade/Serviceで広範に使用
4. **Repositoryトレイトにpoise型** - `MessageId`がトレイト定義に含まれる
5. **AutocompleteChoiceがServiceから返される** - プレゼンテーション層の型がService層に存在
6. **ServiceやFacadeから直接Discord APIを呼び出し** - メッセージ送信・編集等

---

## 参考: eventsレイヤー（違反なし・期待される使用箇所）

以下のeventsレイヤーファイルでのpoise/serenity使用は**適切**です：

- `src/events/` 配下のコマンドハンドラー
- `src/main.rs`
- `src/types/` のpoise関連型定義

これらはClean Architectureにおける「外部層」であり、フレームワーク依存が許容される場所です。

---

## 次のステップ

この報告書をもとに、以下の計画を検討してください：

1. **抽象化層（Gateway/Adapter）の導入** - poise/serenityとビジネスロジック間のインターフェース
2. **Value Objectの作成** - Discord型の代替となるドメイン固有の型
3. **UIビルダーのプレゼンテーション層への移動** - Service層からの分離
4. **依存性注入の改善** - Httpクライアントの適切な管理
5. **Repositoryトレイトの修正** - プリミティブ型への置き換え
