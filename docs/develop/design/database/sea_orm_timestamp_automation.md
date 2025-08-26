# Sea-ORM タイムスタンプ自動化実装仕様書

## 概要

Sea-ORMを使用したテーブルにおいて、`created_at`と`updated_at`フィールドの自動更新機能を実装しました。

## 実装方法

### 1. エンティティレベルでの実装

各エンティティファイルで`ActiveModelBehavior`トレイトの`new`メソッドをオーバーライドして、新規作成時のタイムスタンプ自動設定を実装します。

#### 実装例: `battle_recruitments.rs`

```rust
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "battle_recruitments")]
pub struct Model {
    // ... 他のフィールド
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
    }
}
```

### 2. リポジトリレベルでの更新時自動設定

更新処理では、リポジトリ内でActiveModelを変更する際に`updated_at`を明示的に設定します。

#### 実装例: `battle_recruitments_repository.rs`

```rust
// 更新処理内
active_model.recruit_end_message_id = Set(Some(message_id));
active_model.updated_at = Set(chrono::Utc::now()); // 更新時刻を自動設定
active_model
    .update(&self.connection)
    .await
    .map_err(|e| AppError::Database(e))?;
```

## 適用済みエンティティ

- `battle_recruitments` ✓
- `quests` ✓

## 使用方法

### 新規作成時

```rust
// ActiveModelBehavior::newを使用すると自動的にタイムスタンプが設定される
let active_model = ActiveModel::new();
// または
let active_model = ActiveModel {
    guild_id: Set(guild_id),
    channel_id: Set(channel_id),
    // created_at, updated_atは自動設定されるため省略可能
    ..ActiveModel::new()
};
```

### 更新時

```rust
let mut active_model: ActiveModel = entity.into();
// 必要なフィールドを更新
active_model.some_field = Set(new_value);
// updated_atを手動で設定
active_model.updated_at = Set(chrono::Utc::now());
active_model.update(&connection).await?;
```

## メリット

1. **自動化**: 新規作成時のタイムスタンプ設定が自動化される
2. **一貫性**: 全エンティティで統一されたタイムスタンプ処理
3. **エラー防止**: 手動設定忘れによるNULLエラーを防止
4. **保守性**: 共通の実装パターンにより保守が容易

## 注意事項

- `new()`メソッドを使わずに`Default::default()`を使用した場合、タイムスタンプは自動設定されません
- 更新時の`updated_at`設定は現在手動で行う必要があります（将来的にはbefore_saveトレイトで自動化予定）
- 他のエンティティ（environments、message_texts、quest_aliases）への適用は今後の課題です

## 今後の改善点

1. `before_save`トレイトの実装による完全自動化
2. 全エンティティへの適用
3. マクロによる実装の簡略化
4. トランザクション内でのタイムスタンプ一貫性保証

---
