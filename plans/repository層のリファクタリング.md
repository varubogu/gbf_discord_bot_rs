# Repository層のリファクタリング計画

## 現状の問題点

### 1. ファイル構成の不整合

Repository層には以下の3つの配置場所がある:
- `src/repository/*.rs` - トップレベル（10ファイル）
- `src/repository/database/*.rs` - database実装（18ファイル + mod.rs等）
- `src/repository/database/schedule/*.rs` - スケジュール関連（13ファイル）

**合計: 約44ファイル**

### 2. trait定義と実装のファイル名不一致

トップレベルのtrait定義ファイルとdatabase/の実装ファイルで名前が異なる（単数形・複数形の混在）:

| トップレベル（trait） | database/（実装） | 状態 |
|---------------------|------------------|------|
| `guild_message_texts_repository.rs` | `guild_message_text_repository.rs` | ❌ 不一致 |
| `message_texts_repository.rs` | `message_text_repository.rs` | ❌ 不一致 |
| `quests_repository.rs` | `quest_repository.rs` | ❌ 不一致 |
| `guild_environments_repository.rs` | `guild_environment_repository.rs` | ❌ 不一致 |
| `battle_recruitments_repository.rs` | `battle_recruitments_repository.rs` | ✅ 一致 |
| `guild_quest_disable_repository.rs` | `guild_quest_disable_repository.rs` | ✅ 一致 |
| `recruitment_participants_repository.rs` | `recruitment_participants_repository.rs` | ✅ 一致 |

### 3. trait定義の欠如

以下のリポジトリはdatabase/に実装があるが、トップレベルに対応するtrait定義がない:

- `all_recruitment_notification_roles_repository.rs`
- `battle_style_repository.rs`
- `channel_type_repository.rs`
- `environment_repository.rs`
- `guild_channel_repository.rs`
- `guild_repository.rs`
- `guild_settings_repository.rs`
- `last_process_time_repository.rs`
- `quest_recruitment_notification_roles_repository.rs`

**問題**: Clean Architectureの依存性ルールに違反している可能性（上位層が実装に直接依存）

### 4. 実装の欠如

以下はトップレベルにtraitがあるが、database/に実装がない:

- `quest_aliases_repository.rs`
- `guild_spreadsheet_config_repository.rs`

**推測**: これらは別の実装方法を取っているか、未使用の可能性

### 5. schedule配下の整理不足

`database/schedule/` に13ファイルあるが、すべてトップレベルにtraitがない。
これらは直接実装を参照している可能性が高い。

スケジュール関連ファイル:
```
battle_recruitment_dismissal_repository.rs
battle_recruitment_schedule_dismissal_repository.rs
battle_recruitment_schedule_repository.rs
notification_rel_battle_recruitment_repository.rs
notification_rel_event_schedule_repository.rs
notification_repository.rs
schedule_repository.rs
scheduled_task_cleanup_repository.rs
scheduled_task_dismissal_repository.rs
scheduled_task_dissolution_repository.rs
scheduled_task_notification_repository.rs
scheduled_task_recurring_recruitment_repository.rs
scheduled_task_repository.rs
```

## 提案するリファクタリング

### 方針

1. **trait定義と実装の命名統一** - 単数形に統一
2. **trait定義の補完** - 実装があるものにはすべてtraitを作成
3. **フォルダ構成の整理** - 機能別に整理
4. **未使用コードの削除** - 実装がないtraitは削除または実装を追加

### 提案する新しいフォルダ構成

```
repository/
├── mod.rs                    # 全traitのre-export
├── traits/                   # すべてのtrait定義を集約
│   ├── mod.rs
│   ├── battle_recruitment.rs
│   ├── quest.rs
│   ├── guild.rs
│   ├── message.rs
│   ├── schedule.rs
│   └── ...
├── database/                 # SeaORM実装
│   ├── mod.rs
│   ├── battle_recruitment.rs
│   ├── quest.rs
│   ├── guild/
│   │   ├── mod.rs
│   │   ├── guild.rs
│   │   ├── guild_channel.rs
│   │   ├── guild_environment.rs
│   │   └── guild_settings.rs
│   ├── message/
│   │   ├── mod.rs
│   │   ├── message_text.rs
│   │   └── guild_message_text.rs
│   ├── recruitment/
│   │   ├── mod.rs
│   │   ├── battle_recruitment.rs
│   │   ├── recruitment_participant.rs
│   │   ├── notification_role.rs
│   │   └── ...
│   ├── schedule/
│   │   ├── mod.rs
│   │   ├── notification.rs
│   │   ├── scheduled_task.rs
│   │   ├── battle_recruitment_schedule.rs
│   │   └── ...
│   └── util/
│       ├── db_compat.rs
│       └── models_database.rs
```

### 代替案: 最小限のリファクタリング

全面的な再構成が大規模すぎる場合、以下の最小限の修正を行う:

#### Phase 1: 命名の統一（破壊的変更なし）

1. トップレベルのファイル名を実装に合わせて単数形に変更
   - `message_texts_repository.rs` → `message_text_repository.rs`
   - `quests_repository.rs` → `quest_repository.rs`
   - `guild_message_texts_repository.rs` → `guild_message_text_repository.rs`
   - `guild_environments_repository.rs` → `guild_environment_repository.rs`

2. trait名も対応して変更
   - `MessageTextsRepository` → `MessageTextRepository`
   - `QuestsRepository` → `QuestRepository`
   - etc.

#### Phase 2: 欠落しているtraitの追加

database/にあるがtraitがないリポジトリにtraitを追加:

優先度高:
- `guild_settings_repository.rs` - ギルド設定（重要）
- `guild_channel_repository.rs` - チャンネル管理（重要）
- `battle_style_repository.rs` - バトルスタイル（頻繁に使用）

優先度中:
- `all_recruitment_notification_roles_repository.rs`
- `quest_recruitment_notification_roles_repository.rs`
- `guild_repository.rs`
- `last_process_time_repository.rs`

優先度低（masterテーブル、読み取り専用が多い）:
- `channel_type_repository.rs`
- `environment_repository.rs`

#### Phase 3: schedule配下の整理

スケジュール関連リポジトリのtrait定義を作成し、依存性を逆転:

```
repository/
├── traits/
│   └── schedule/
│       ├── notification.rs
│       ├── scheduled_task.rs
│       └── ...
└── database/
    └── schedule/
        └── (既存のまま、traitを実装)
```

## 実施優先順位

### 🔴 優先度: 高（Clean Architecture違反の修正）

1. **trait定義の追加**
   - `guild_settings_repository`
   - `guild_channel_repository`
   - `battle_style_repository`

   **理由**: これらは現在Service層から直接実装を参照している可能性が高い

### 🟡 優先度: 中（命名の統一）

2. **ファイル名とtrait名の単数形統一**
   - 影響範囲が大きいため、一括で実施
   - import文の一括置換が必要

### 🟢 優先度: 低（将来的な改善）

3. **フォルダ構成の再編成**
   - 大規模な変更のため、v2.0などメジャーバージョンアップ時に検討
   - 現時点では44ファイルで管理可能な範囲内

## 実施時の注意事項

### 破壊的変更の影響範囲

- **Service層**: traitのimportパスが変更される
- **Facade層**: Repository生成箇所が影響を受ける
- **テストコード**: mockの定義が変更される

### テスト戦略

1. trait追加時は既存の実装を変更しない
2. 命名変更時は全ファイル一括で行う（部分的な変更は避ける）
3. 各フェーズ後に全テストを実行
4. リファクタリング前後で動作が同一であることを確認

## 参考: 現在のtrait定義状況

### トップレベルに存在するtrait（10個）

```rust
// src/repository/mod.rs より
pub use battle_recruitments_repository::BattleRecruitmentsRepository;
pub use guild_environments_repository::GuildEnvironmentRepository;
pub use guild_message_texts_repository::GuildMessageTextRepository;
pub use guild_quest_disable_repository::GuildQuestDisableRepository;
pub use message_texts_repository::MessageTextRepository;
pub use recruitment_participants_repository::RecruitmentParticipantsRepository;
pub use guild_spreadsheet_config_repository::GuildSpreadsheetConfigRepositoryTrait;
pub use quests_repository::QuestRepository;
```

### database/の実装（18個 + schedule配下13個 = 31個）

- trait定義あり: 7個
- trait定義なし: 24個 ← **要対応**

## 実施チェックリスト

### Phase 1: 命名統一
- [ ] `message_texts_repository` → `message_text_repository` にリネーム
- [ ] `quests_repository` → `quest_repository` にリネーム
- [ ] `guild_message_texts_repository` → `guild_message_text_repository` にリネーム
- [ ] `guild_environments_repository` → `guild_environment_repository` にリネーム
- [ ] 対応するtrait名を変更
- [ ] Service層のimport文を修正
- [ ] ビルド確認
- [ ] テスト実行

### Phase 2: trait定義追加（優先度高）
- [ ] `GuildSettingsRepository` trait作成
- [ ] `GuildChannelRepository` trait作成
- [ ] `BattleStyleRepository` trait作成
- [ ] 既存実装にtraitを適用
- [ ] Service層で実装ではなくtraitを参照するように修正
- [ ] ビルド確認
- [ ] テスト実行

### Phase 3: trait定義追加（優先度中）
- [ ] `AllRecruitmentNotificationRolesRepository` trait作成
- [ ] `QuestRecruitmentNotificationRolesRepository` trait作成
- [ ] `GuildRepository` trait作成
- [ ] `LastProcessTimeRepository` trait作成
- [ ] ビルド確認
- [ ] テスト実行

### Phase 4: schedule配下の整理
- [ ] schedule配下の各リポジトリにtrait定義を作成
- [ ] Service層の依存を修正
- [ ] ビルド確認
- [ ] テスト実行

## まとめ

現在のRepository層は以下の問題を抱えている:

1. **命名の不統一** - 単数形・複数形が混在
2. **trait定義の欠如** - 24個のリポジトリにtraitがない
3. **Clean Architecture違反の可能性** - 上位層が実装に直接依存

最小限のリファクタリングとして、以下を推奨:

1. Phase 1: 命名統一（工数: 小、影響: 中）
2. Phase 2: 重要なリポジトリのtrait追加（工数: 中、影響: 大）
3. Phase 3以降: 段階的に改善（工数: 大、影響: 小）

全面的なフォルダ再構成は将来的な課題とし、まずは**アーキテクチャ違反の修正**を優先すべき。
