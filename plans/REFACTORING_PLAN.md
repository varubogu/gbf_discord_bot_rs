# コマンド層リファクタリング計画

## 概要

コマンド層に処理が集中している問題を解決するため、ビジネスロジックをService層・Facade層に移動する大規模リファクタリングを実施します。

## 現状の問題点

### アーキテクチャ違反の統計

- **52%のコマンド（14ファイル）**がトランザクション管理をコマンド層で実施
- **18%のコマンド（5ファイル）**が複雑なビジネスロジックをコマンド層に実装
- 最大の問題ファイル: `recruitment_schedule_create.rs`（521行、100行超は禁止）

### 主な違反パターン

1. **コマンド層でのトランザクション管理**（Facade層の責務）
2. **Repository層の直接呼び出し**（Service層・Facade層をバイパス）
3. **複雑なビジネスロジックの実装**（曜日パース、データ整形など）

## リファクタリング戦略

### Phase 1: 最優先（複雑度・影響度が高い）

#### 1.1 定期募集作成コマンド（recruitment_schedule_create.rs）

**現状の問題:**
- 521行の巨大ファイル（ルール違反: 100行超）
- コマンド層でトランザクション管理
- 複雑な曜日パース処理（parse_days、parse_continuous_days等）
- 時刻パース、タイムゾーン変換、バリデーションがコマンド層に混在

**リファクタリング内容:**

新規作成するコンポーネント:
```
facades/recruitment/recruitment_schedule_facade.rs
  - create_recruitment_schedule() - 定期募集作成
  - update_recruitment_schedule() - 定期募集更新
  - delete_recruitment_schedule() - 定期募集削除
  - get_recruitment_schedule() - 定期募集取得

services/recruitment/schedule/
  - schedule_create_service.rs - 定期募集作成ビジネスロジック
  - days_parser_service.rs - 曜日パース処理
    - parse_days_input() - 各種フォーマット対応
    - parse_comma_separated()
    - parse_space_separated()
    - parse_continuous_pattern()
  - time_parser_service.rs - 時刻パース処理
    - parse_time_string()
    - convert_to_utc()
```

**責務の分離:**
- Command層: コマンド引数受け取り、Facade呼び出し、結果表示
- Facade層: トランザクション管理、複数Serviceの協調
- Service層: 曜日パース、時刻パース、バリデーション、スケジュール生成ロジック
- Repository層: データ永続化

**期待される改善:**
- コマンドファイル: 521行 → 100行以下
- テスタビリティ向上（Service層を独立してテスト可能）
- 再利用性向上（パース処理を他コマンドでも利用可能）

#### 1.2 チャンネル登録コマンド（channel_register.rs）

**現状の問題:**
- コマンド層でトランザクション管理
- Repository層を直接呼び出し
- 複雑なビジネスロジック（ギルド自動登録、チャンネル種別確認）

**リファクタリング内容:**

新規作成するコンポーネント:
```
facades/channel/channel_management_facade.rs
  - register_channel() - チャンネル登録
  - unregister_channel() - チャンネル削除
  - show_channel_settings() - チャンネル設定表示

services/channel/
  - channel_registration_service.rs - チャンネル登録ビジネスロジック
    - validate_channel_type() - チャンネル種別検証
    - ensure_guild_registered() - ギルド登録確認・作成
  - channel_display_service.rs - チャンネル設定表示ロジック
    - format_channel_settings() - 設定データ整形
```

**責務の分離:**
- Command層: チャンネル情報取得、Facade呼び出し、結果表示
- Facade層: トランザクション管理、Guild・Channel処理の協調
- Service層: バリデーション、ギルド自動登録ロジック、データ整形
- Repository層: データ永続化

#### 1.3 チャンネル設定表示コマンド（channel_show.rs）

**現状の問題:**
- コマンド層でトランザクション管理
- Repository層を直接呼び出し
- データ整形ロジックをコマンド層に実装

**リファクタリング内容:**
- Phase 1.2で作成する`ChannelManagementFacade`を使用
- `channel_display_service.rs`でデータ整形ロジックを実装

### Phase 2: 高優先（トランザクション管理の違反）

#### 2.1 チャンネル削除コマンド（channel_unregister.rs）

**リファクタリング内容:**
- Phase 1.2で作成する`ChannelManagementFacade::unregister_channel()`を使用

#### 2.2 タイムゾーン設定コマンド（timezone_set.rs）

**現状の問題:**
- コマンド層でトランザクション管理
- Repository層を直接呼び出し

**リファクタリング内容:**

新規作成するコンポーネント:
```
facades/timezone/timezone_facade.rs
  - set_guild_timezone() - タイムゾーン設定
  - get_guild_timezone() - タイムゾーン取得
```

**責務の分離:**
- Command層: タイムゾーン引数受け取り、Facade呼び出し
- Facade層: トランザクション管理
- Service層: 既存のTimezoneServiceを活用
- Repository層: データ永続化

#### 2.3 スケジュール一覧コマンド（schedule_list.rs）

**現状の問題:**
- コマンド層でトランザクション管理
- Repository層を直接呼び出し
- データフィルタリング・ソート・整形をコマンド層で実施

**リファクタリング内容:**

新規作成するコンポーネント:
```
facades/schedule/schedule_query_facade.rs
  - list_future_schedules() - 未来のスケジュール一覧
  - get_schedule_statistics() - スケジュール統計

services/schedule/
  - schedule_query_service.rs - スケジュール取得・整形
    - filter_future_notifications() - 未来の通知フィルタリング
    - sort_by_datetime() - 日時順ソート
    - format_schedule_list() - 一覧表示用整形
```

#### 2.4 スケジュール統計コマンド（schedule_stats.rs）

**リファクタリング内容:**
- Phase 2.3で作成する`ScheduleQueryFacade::get_schedule_statistics()`を使用

### Phase 3: 低優先（既存のFacadeを使用しているが改善の余地あり）

#### 3.1 新規募集作成コマンド（recruit_new.rs）

**現状の問題:**
- タイムゾーン取得処理がコマンド層に散在
- Service層の関数（add_recruitment_reactions）を直接呼び出し

**リファクタリング内容:**
- `new_recruit` Facadeにタイムゾーン取得処理を含める
- `add_recruitment_reactions`をFacade内に移動

#### 3.2 スプレッドシート読み込みコマンド（gspread_load.rs）

**現状の問題:**
- スプレッドシートID取得の際に直接トランザクションを開始

**リファクタリング内容:**
- スプレッドシートID取得処理をFacadeに移動

## 具体的な実装例

### 例1: recruitment_schedule_create.rs のリファクタリング

#### Before（現状: 521行、アーキテクチャ違反多数）

```rust
// ❌ コマンド層でトランザクション管理
pub async fn recruitment_schedule_create(
    ctx: PoiseContext<'_>,
    name: String,
    quest: String,
    quest_start_time: String,
    days: String,
    recruit_start_time: String,
    // ... その他引数
) -> Result<()> {
    // ❌ Repository層を直接呼び出し
    let quest_repo = SeaOrmQuestRepository::new();
    let search_results = quest_repo
        .search_by_name_or_alias(app_state.guild_db(), &quest)
        .await?;

    // ❌ ビジネスロジックをコマンド層で実装
    let timezone_service = TimezoneService::new(timezone_repo);
    let timezone = timezone_service
        .get_guild_timezone(app_state.guild_db(), guild_id.get() as i64)
        .await?;

    // ❌ 複雑なパース処理（138-146行）
    let quest_start_time_local = parse_time(&quest_start_time)?;
    let recruit_start_time_local = parse_time(&recruit_start_time)?;
    let local_day_of_weeks = parse_days(&days)?;

    // ❌ バリデーション処理
    service.validate_schedule_input(...)?;

    // ❌ UTC変換処理
    let (utc_quest_days, quest_start_time_tt) =
        convert_local_days_and_time_to_utc(...)?;

    // ❌ トランザクション開始（Facade層の責務）
    let txn = app_state.guild_db().begin().await?;

    // ❌ Repository層を直接呼び出し
    let schedule_repo = BattleRecruitmentScheduleRepository::new();
    schedule_repo.create_with_txn(&txn, ...).await?;

    // ❌ コミット（Facade層の責務）
    txn.commit().await?;

    // ... メッセージ表示（約100行）
}

// ❌ 複雑なパース関数（コマンド層に400行以上）
fn parse_days(days: &str) -> Result<Vec<DayOfWeek>> { /* 100行以上 */ }
fn parse_time(time: &str) -> Result<NaiveTime> { /* ... */ }
fn format_days(days: &[DayOfWeek]) -> String { /* ... */ }
```

#### After（理想: 100行以下、クリーンアーキテクチャ遵守）

**Command層（recruitment_schedule_create.rs: 約80行）**

```rust
// ✅ コマンド層はFacade呼び出しと結果表示のみ
pub async fn recruitment_schedule_create(
    ctx: PoiseContext<'_>,
    name: String,
    quest: String,
    quest_start_time: String,
    days: String,
    recruit_start_time: String,
    battle_style: Option<i32>,
    recruit_start_day_offset: Option<i64>,
    note: Option<String>,
) -> Result<()> {
    let guild_id = ctx.guild_id().ok_or(...)?;
    let user_id = ctx.author().id;

    info!(guild_id = guild_id.get(), "定期募集作成コマンドが実行されました");
    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;

    // ✅ Facade層を呼び出し
    let facade = RecruitmentScheduleFacade::new(app_state.clone());
    let result = facade.create_recruitment_schedule(
        guild_id.get(),
        user_id.get(),
        name.clone(),
        &quest,
        &quest_start_time,
        &days,
        &recruit_start_time,
        battle_style,
        recruit_start_day_offset.unwrap_or(1) as i32,
        note.clone(),
    ).await?;

    // ✅ 結果表示のみ（UI層の責務）
    let embed = CreateEmbed::default()
        .title("✅ 定期募集スケジュールを作成しました")
        .description(format!(
            "**スケジュール名**: {}\n\
             **スケジュールID**: {}\n\
             **クエスト**: {}\n\
             **対象曜日**: {}\n\
             ...",
            result.schedule_name,
            result.schedule_id,
            result.quest_name,
            result.days_display,
        ));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
```

**Facade層（facades/recruitment/recruitment_schedule_facade.rs: 約150行）**

```rust
pub struct RecruitmentScheduleFacade {
    app_state: Arc<AppState>,
}

impl RecruitmentScheduleFacade {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// 定期募集スケジュールを作成
    pub async fn create_recruitment_schedule(
        &self,
        guild_id: u64,
        user_id: u64,
        name: String,
        quest_alias: &str,
        quest_start_time: &str,
        days: &str,
        recruit_start_time: &str,
        battle_style_id: Option<i32>,
        recruit_day_offset: i32,
        note: Option<String>,
    ) -> Result<ScheduleCreationResult> {
        // ✅ トランザクション開始（Facade層の責務）
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        set_current_guild_id(&txn, guild_id as i64).await?;

        let result = async {
            // ✅ 複数Serviceを協調（Facade層の責務）

            // 1. タイムゾーン取得Service
            let timezone_service = TimezoneService::new(
                Arc::new(GuildTimezoneRepository::new())
            );
            let timezone = timezone_service
                .get_guild_timezone(conn, guild_id as i64)
                .await?;

            // 2. スケジュール作成Service（新規）
            let schedule_service = ScheduleCreateService::new(
                Arc::new(SeaOrmQuestRepository::new()),
                Arc::new(SeaOrmBattleStyleRepository::new()),
                Arc::new(BattleRecruitmentScheduleRepository::new()),
                Arc::new(GuildChannelRepository::new()),
            );

            let schedule_data = schedule_service.create_schedule(
                &txn,
                guild_id as i64,
                user_id as i64,
                name,
                quest_alias,
                quest_start_time,
                days,
                recruit_start_time,
                battle_style_id,
                recruit_day_offset,
                note,
                timezone,
            ).await?;

            Ok::<_, AppError>(schedule_data)
        }.await;

        // ✅ 結果に応じてcommit/rollback（Facade層の責務）
        match result {
            Ok(schedule_data) => {
                txn.commit().await?;
                info!(schedule_id = schedule_data.schedule_id, "定期募集スケジュールを作成しました");
                Ok(schedule_data)
            }
            Err(e) => {
                txn.rollback().await?;
                error!(error = %e, "定期募集スケジュール作成エラー");
                Err(e)
            }
        }
    }
}
```

**Service層（services/recruitment/schedule/schedule_create_service.rs: 約200行）**

```rust
pub struct ScheduleCreateService {
    quest_repo: Arc<dyn QuestRepository>,
    battle_style_repo: Arc<SeaOrmBattleStyleRepository>,
    schedule_repo: Arc<BattleRecruitmentScheduleRepository>,
    channel_repo: Arc<GuildChannelRepository>,
}

impl ScheduleCreateService {
    /// スケジュール作成（ビジネスロジック）
    pub async fn create_schedule(
        &self,
        txn: &DatabaseTransaction,  // ✅ トランザクションを受け取るのみ
        guild_id: i64,
        user_id: i64,
        name: String,
        quest_alias: &str,
        quest_start_time: &str,
        days_str: &str,
        recruit_start_time: &str,
        battle_style_id: Option<i32>,
        recruit_day_offset: i32,
        note: Option<String>,
        timezone: Tz,
    ) -> Result<ScheduleCreationResult> {
        // 1. クエスト検索・取得
        let quest = self.find_quest(txn, quest_alias).await?;

        // 2. バトルスタイル決定
        let battle_style_id = battle_style_id
            .unwrap_or(quest.default_battle_style_id);
        let battle_style = self.get_battle_style(txn, battle_style_id).await?;

        // 3. 時刻・曜日パース（DaysParserService, TimeParserServiceを使用）
        let parser = DaysParserService::new();
        let local_days = parser.parse_days_input(days_str)?;

        let time_parser = TimeParserService::new();
        let quest_start_time_local = time_parser.parse_time_string(quest_start_time)?;
        let recruit_start_time_local = time_parser.parse_time_string(recruit_start_time)?;

        // 4. バリデーション
        let schedule_service = RecruitmentScheduleService::new();
        schedule_service.validate_schedule_input(
            &local_days,
            quest_start_time_local,
            recruit_day_offset,
            Some(recruit_start_time_local),
        )?;

        // 5. UTC変換
        let (utc_days, quest_start_utc) =
            convert_local_days_and_time_to_utc(&local_days, quest_start_time_local, timezone)?;
        let (_, recruit_start_utc) =
            convert_local_days_and_time_to_utc(&local_days, recruit_start_time_local, timezone)?;

        // 6. チャンネル取得
        let channel = self.channel_repo
            .get_by_guild_and_type_with_txn(txn, guild_id, 2)
            .await?
            .ok_or_else(|| AppError::Business {
                message: "マルチ募集チャンネルが登録されていません".to_string(),
            })?;

        // 7. スケジュール保存
        let (schedule, _) = self.schedule_repo.create_with_txn(
            txn,  // ✅ トランザクションを渡す
            name.clone(),
            guild_id,
            channel.channel_id,
            quest.id,
            battle_style_id,
            quest_start_utc,
            recruit_day_offset,
            Some(recruit_start_utc),
            None,
            note.clone(),
            user_id,
            utc_days.clone(),
        ).await?;

        // 8. 結果データ作成
        Ok(ScheduleCreationResult {
            schedule_id: schedule.id,
            schedule_name: name,
            quest_name: quest.name,
            days_display: DaysParserService::format_days(&local_days),
            // ... その他フィールド
        })
        // ✅ commit/rollbackはしない（Facade層の責務）
    }
}
```

**Service層（services/recruitment/schedule/days_parser_service.rs: 約150行）**

```rust
pub struct DaysParserService;

impl DaysParserService {
    pub fn new() -> Self {
        Self
    }

    /// 曜日文字列を解析（各種フォーマット対応）
    pub fn parse_days_input(&self, days: &str) -> Result<Vec<DayOfWeek>> {
        // "毎日" "すべて" などの特殊ケース
        if days.contains("毎日") || days.contains("すべて") || days == "all" {
            return Ok(vec![
                DayOfWeek::Monday, DayOfWeek::Tuesday, DayOfWeek::Wednesday,
                DayOfWeek::Thursday, DayOfWeek::Friday, DayOfWeek::Saturday,
                DayOfWeek::Sunday,
            ]);
        }

        // カンマ区切り優先
        if days.contains(',') {
            return self.parse_comma_separated(days);
        }

        // スペース区切り
        if days.contains(' ') {
            return self.parse_space_separated(days);
        }

        // 連続パターン（「月火水」など）
        self.parse_continuous_pattern(days)
    }

    /// 曜日リストを表示用文字列に整形
    pub fn format_days(days: &[DayOfWeek]) -> String {
        days.iter()
            .map(|d| d.to_japanese())
            .collect::<Vec<_>>()
            .join(", ")
    }

    // ... その他のパース関数
}
```

**Service層（services/recruitment/schedule/time_parser_service.rs: 約50行）**

```rust
pub struct TimeParserService;

impl TimeParserService {
    pub fn new() -> Self {
        Self
    }

    /// 時刻文字列をパース（"22:00" → NaiveTime）
    pub fn parse_time_string(&self, time: &str) -> Result<NaiveTime> {
        let parts: Vec<&str> = time.split(':').collect();
        if parts.len() != 2 {
            return Err(AppError::Validation {
                message: format!("時刻の形式が不正です: {}", time),
            });
        }

        let hour = parts[0].parse::<u32>()
            .map_err(|_| AppError::Validation {
                message: format!("時が不正です: {}", parts[0]),
            })?;

        let minute = parts[1].parse::<u32>()
            .map_err(|_| AppError::Validation {
                message: format!("分が不正です: {}", parts[1]),
            })?;

        NaiveTime::from_hms_opt(hour, minute, 0)
            .ok_or_else(|| AppError::Validation {
                message: format!("時刻が範囲外です: {}:{}", hour, minute),
            })
    }
}
```

### 例2: 理想的なCommand実装（recruit_cancel.rs: 79行）

これは既に理想的な実装です：

```rust
pub async fn recruit_cancel(
    ctx: PoiseContext<'_>,
    message: Message,
) -> Result<()> {
    ctx.defer().await?;

    // ✅ Facade層のみを呼び出し
    match CancelFacade::can_cancel(ctx, &message).await {
        Ok(CanCancelResult::Success) => {
            CancelFacade::confirm_cancel(ctx, &message).await
        }
        // ✅ 結果パターンごとにメッセージ表示（UI層の責務）
        Ok(CanCancelResult::AlreadyCancelled) => {
            ctx.send(
                poise::CreateReply::default()
                    .content("募集は既にキャンセルされています。")
                    .ephemeral(true),
            ).await?;
            Ok(())
        }
        // ... その他結果パターン
    }
}
```

**優れている点:**
- コマンド層は79行のみ
- Facade層の関数のみを呼び出し
- UI処理（メッセージ表示）に専念
- ビジネスロジックへの直接アクセスなし

## 実装順序（推奨）

### Step 1: Facade・Service新規作成（Phase 1）
1. `DaysParserService`, `TimeParserService` （約200行、再利用性高）
2. `ScheduleCreateService` （約200行）
3. `RecruitmentScheduleFacade` （約150行）
4. `ChannelRegistrationService`, `ChannelDisplayService` （約150行）
5. `ChannelManagementFacade` （約100行）

### Step 2: Facade・Service新規作成（Phase 2）
6. `TimezoneFacade` （約50行）
7. `ScheduleQueryService` （約100行）
8. `ScheduleQueryFacade` （約100行）

### Step 3: コマンド層修正（全Phase）
9. `recruitment_schedule_create.rs` - 最優先（521行 → 80行）
10. `channel_register.rs`, `channel_show.rs`, `channel_unregister.rs` （各100行以下に）
11. `timezone_set.rs` （80行以下に）
12. `schedule_list.rs`, `schedule_stats.rs` （各80行以下に）
13. `recruit_new.rs`, `gspread_load.rs` - 微修正

### Step 4: テスト作成
14. `DaysParserService`, `TimeParserService` の単体テスト（重要）
15. 各Facade・Serviceの単体テスト
16. 統合テスト（必要に応じて）

## アーキテクチャ原則の再確認

リファクタリング時に遵守すべき原則:

### トランザクション管理
```rust
// ✅ 正しい: Facade層でトランザクション管理
pub async fn facade_method(...) -> Result<T> {
    let txn = conn.begin().await?;
    let result = async {
        service_method(&txn, ...).await?;
        Ok(value)
    }.await;
    match result {
        Ok(value) => { txn.commit().await?; Ok(value) }
        Err(e) => { txn.rollback().await?; Err(e) }
    }
}

// ❌ 間違い: Command層でトランザクション管理
pub async fn command_handler(...) -> Result<()> {
    let txn = db.begin().await?;  // NG
    repository.create_with_txn(&txn, ...).await?;
    txn.commit().await?;  // NG
}
```

### 層間の依存関係
```
Command → Facade → Service → Repository
  ✅        ✅        ✅         ✅

Command → Service → Repository
  ❌        ❌         ✅

Command → Repository
  ❌         ❌
```

### Service層の実装
```rust
// ✅ 正しい: トランザクションを受け取るのみ
pub async fn service_method(
    txn: &DatabaseTransaction,
    ...
) -> Result<T> {
    repository.create_with_txn(txn, ...).await?;
    Ok(result)  // commit/rollbackはしない
}

// ❌ 間違い: Service層でトランザクション管理
pub async fn service_method(conn: &DatabaseConnection, ...) -> Result<T> {
    let txn = conn.begin().await?;  // NG
    repository.create_with_txn(&txn, ...).await?;
    txn.commit().await?;  // NG
}
```

## 期待される効果

### コード品質
- アーキテクチャルール違反: 52% → 0%
- 平均コマンド行数: 大幅削減（特にrecruitment_schedule_create.rs: 521行 → 100行以下）
- テスタビリティ向上

### 保守性
- ビジネスロジックの再利用性向上
- 責務の明確化
- 変更容易性の向上

### 一貫性
- 全コマンドが同一のアーキテクチャパターンに従う
- トランザクション管理の一元化

## リスク管理

### 潜在的リスク
1. **大規模変更による既存機能への影響**
   - 対策: 各コマンドごとに段階的に修正、テスト実施
2. **トランザクション境界の誤り**
   - 対策: Facade層でのみtxn管理、コードレビュー徹底
3. **パフォーマンス低下**
   - 対策: 不要なDB呼び出しを避ける、必要に応じてキャッシュ

### テスト戦略
- 既存の動作を保証する回帰テスト
- 新規Service層の単体テスト
- Facade層の統合テスト

## 完了条件

- [ ] 全コマンドがFacade層経由でビジネスロジックを呼び出す
- [ ] コマンド層でのトランザクション管理が0件
- [ ] コマンド層からのRepository直接呼び出しが0件
- [ ] 100行を超えるコマンドファイルが0件
- [ ] 新規作成したFacade・Serviceに適切なテストが存在
- [ ] 全テストがパス
- [ ] ビルドエラー・警告が0件
