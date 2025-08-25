# データベースコーディングルール

このドキュメントでは、プロジェクトにおけるデータベース操作のコーディングルールとベストプラクティスを定義します。

## 基本原則

### 1. クリーンアーキテクチャにおけるトランザクション管理の層別責務

**重要**: トランザクションの生成（begin）とコミット・ロールバックは**Facade層でのみ実行可能**です。
生成されたトランザクションはService層を経由してRepository層に渡され、最終的なコミット判断はFacade層まで戻ってきて行います。

#### 各層の責務

##### Facade層の責務

- `app_state.db().begin().await?` によるトランザクション生成
- `txn.commit().await?` または `txn.rollback().await?` による最終判断
- Service層への協調とトランザクション受け渡し
- 全体の処理結果に基づく成否判断

##### Service層の責務

- トランザクションを引数として受け取る（`txn: &DatabaseTransaction`）
- ビジネスロジックの実行
- Repository層へのトランザクション受け渡し
- **トランザクションの生成・コミット・ロールバックは禁止**

##### Repository層の責務

- トランザクションを使用したデータアクセス操作
- `create_with_txn(txn, ...)` などのトランザクション付きメソッド使用
- **トランザクションの生成・コミット・ロールバックは禁止**

#### 層間フロー図

```
Facade層
│
├─ app_state.db().begin().await? （トランザクション生成）
│
├─ Service層呼び出し（txnを引数で渡す）
│   │
│   ├─ ビジネスロジック実行
│   │
│   └─ Repository層呼び出し（txnを引数で渡す）
│       │
│       └─ create_with_txn(txn, ...) などでDB操作
│
└─ 結果判定 → txn.commit() / txn.rollback()
```

### 2. トランザクション管理

データベース操作は必ずトランザクション内で実行し、適切なエラーハンドリングを行う。

#### 基本パターン

```rust
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

#[instrument]
pub async fn example_function(
    ctx: &PoiseContext<'_>,
    // その他のパラメータ
) -> types::Result<()> {
    info!("処理開始メッセージ");
    
    let app_state = &ctx.data().app_state;
    let txn = app_state.db().begin().await?;

    let result = async {
        // ここでデータベース操作を実行
        // 複数の処理がある場合は順番に実行
        
        Ok::<(), crate::types::AppError>(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!("処理完了メッセージ");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "エラー詳細メッセージ");
            Err(e)
        }
    }
}
```

### 2. エラーハンドリング

#### エラー処理の原則

- トランザクション内で発生したエラーは必ずrollbackを実行
- エラー情報はtracing::errorを使用してログ出力
- 元のエラーを適切に伝播させる

#### エラーログのベストプラクティス

```rust
match result {
Ok(_) => {
txn.commit().await ?;
info! (key = % value, "成功時の構造化ログ");
Ok(())
}
Err(e) => {
txn.rollback().await ?;
error! (error = % e, key = %value, "エラー時の構造化ログ");
Err(e)
}
}
```

### 3. トランザクションスコープ

#### スコープ設計

- トランザクションは論理的な業務単位で開始
- asyncブロック内で関連する処理をすべて実行
- 処理結果の成否でcommit/rollbackを判断

#### 注意点

- トランザクション外でのDB操作は禁止
- 長時間のトランザクションは避ける
- 外部API呼び出しとDB操作を適切に分離

### 4. パフォーマンス考慮

#### 最適化のガイドライン

- 必要最小限のデータのみ取得
- バッチ処理での一括操作を活用
- インデックスを考慮したクエリ設計

### 5. 使用例

#### 実際のコード例（層別責務の実装パターン）

##### Facade層の実装例（new_recruit.rsから抜粋）

```rust
// Facade層：トランザクション管理の責務を持つ
pub async fn new_recruitment(
    ctx: &PoiseContext<'_>,
    quest_alias: &str,
    battle_type: BattleType,
) -> types::Result<()> {
    info!("BattleRecruitmentFacade::new_recruitment - 新しい募集を開始します");
    let app_state = &ctx.data().app_state;
    
    // ✓ Facade層でトランザクション生成
    let txn = app_state.db().begin().await?;

    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);
    let channel_id = ctx.channel_id().get();

    let result = async {
        // 1. Service層で募集データ作成（トランザクション不要な処理）
        let recruitment_data = new::create_recruitment_data(
            quest_alias, battle_type, channel_id, guild_id, app_state, None
        ).await?;
        
        // 2. Service層でメッセージ送信（Discord API操作）
        let message_id = new::send_recruitment_message(ctx, &recruitment_data).await?;
        
        // 3. Service層でリアクション追加（Discord API操作）
        new::add_recruitment_reactions(ctx, message_id, &recruitment_data.reactions).await?;
        
        // 4. ✓ Service層でデータ保存（トランザクションをService層に渡す）
        new::save_recruitment(&recruitment_data, message_id, &txn, app_state).await?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            // ✓ Facade層でコミット判断・実行
            txn.commit().await?;
            Ok(())
        }
        Err(e) => {
            // ✓ Facade層でロールバック判断・実行
            txn.rollback().await?;
            Err(e)
        }
    }
}
```

##### Service層の実装例（new.rsから抜粋）

```rust
// Service層：トランザクションを受け取り、Repository層に渡す
pub async fn save_recruitment(
    recruitment_data: &RecruitmentData,
    message_id: u64,
    txn: &DatabaseTransaction,  // ✓ Facade層からトランザクションを受け取る
    app_state: &AppState,
) -> types::Result<()> {
    let repos = RepositoryContainer::new(&app_state.db_connection);
    let battle_recruitment_repo = repos.battle_recruitment();

    // ✓ Repository層にトランザクションを渡してDB操作を委託
    battle_recruitment_repo
        .create_with_txn(
            txn,  // ✓ トランザクションをRepository層に渡す
            recruitment_data.guild_id as i64,
            recruitment_data.channel_id as i64,
            message_id as i64,
            recruitment_data.quest.target_id,
            recruitment_data.battle_type as i32,
            recruitment_data.expiry_date,
        )
        .await?;

    info!("Successfully registered recruitment in database");
    Ok(())
    // ❌ Service層ではcommit/rollbackしない
}
```

##### Repository層の実装例（概念例）

```rust
// Repository層：受け取ったトランザクションでDB操作を実行
impl BattleRecruitmentRepository {
    pub async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,  // ✓ Service層からトランザクションを受け取る
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        quest_id: i32,
        battle_type: i32,
        expiry_date: DateTime<chrono::Utc>,
    ) -> types::Result<()> {
        // ✓ 受け取ったトランザクションを使用してDB操作
        let battle_recruitment = battle_recruitment::ActiveModel {
            guild_id: Set(guild_id),
            channel_id: Set(channel_id),
            message_id: Set(message_id),
            quest_id: Set(quest_id),
            battle_type: Set(battle_type),
            expiry_date: Set(expiry_date),
            ..Default::default()
        };
        
        battle_recruitment.insert(txn).await?;
        Ok(())
        // ❌ Repository層ではcommit/rollbackしない
    }
}
```

```rust
pub async fn new_recruitment(
    ctx: &PoiseContext<'_>,
    quest_alias: &str,
    battle_type: BattleType,
) -> types::Result<()> {
    info!("BattleRecruitmentFacade::new_recruitment - 新しい募集を開始します");
    let app_state = &ctx.data().app_state;
    let txn = app_state.db().begin().await?;

    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);
    let channel_id = ctx.channel_id().get();

    let result = async {
        // 1. Service層で募集データ作成
        let recruitment_data = new::create_recruitment_data(
            quest_alias, battle_type, channel_id, guild_id, app_state, None
        ).await?;
        
        // 2. Service層でメッセージ送信
        let message_id = new::send_recruitment_message(ctx, &recruitment_data).await?;
        
        // 3. Service層でリアクション追加
        new::add_recruitment_reactions(ctx, message_id, &recruitment_data.reactions).await?;
        
        // 4. Service層でデータ保存
        new::save_recruitment(&recruitment_data, message_id, &txn, app_state).await?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            Err(e)
        }
    }
}
```

## 禁止事項

### やってはいけないこと

#### 共通禁止事項

- トランザクション外でのDB操作
- エラー時のrollback忘れ
- トランザクションの入れ子
- 長時間保持するトランザクション

#### 層別責務違反の禁止事項

##### Service層・Repository層での禁止事項

- **Service層やRepository層でのトランザクション生成は禁止**
- **Service層やRepository層でのcommit/rollback実行は禁止**
- **トランザクションの最終判断はFacade層でのみ実行**

##### 層跳ばしの禁止事項

- **Facade層からRepository層への直接呼び出しは禁止**
- **Service層を経由せずにRepository層にアクセスすることは禁止**

### 非推奨パターン

```rust
// ❌ 非推奨：トランザクション外でのDB操作
pub async fn bad_example() -> types::Result<()> {
    let app_state = get_app_state();
    // 直接DB操作（トランザクションなし）
    some_db_operation(app_state.db()).await?;
    Ok(())
}

// ❌ 非推奨：rollback忘れ
match result {
    Ok(_) => txn.commit().await?,
    Err(e) => return Err(e), // rollbackが実行されない
}
```