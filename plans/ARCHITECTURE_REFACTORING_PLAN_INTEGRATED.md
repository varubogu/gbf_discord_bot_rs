# アーキテクチャ違反リファクタリング計画（統合版）

## 📅 調査日時
2025-12-15

## 📋 概要

本計画は、以下2つのリファクタリング計画を統合したものです：
1. **REFACTORING_PLAN.md** - Events層の詳細な実装ガイド
2. **architecture_refactoring_plan_2025-12-15.md** - 全層の包括的な調査結果

architecture.md（`docs/develop/rules/architecture.md`）で定義されたクリーンアーキテクチャルールと実際の実装に多数の乖離が発見されました。本計画は、これらの違反を体系的に修正し、アーキテクチャの整合性を回復するための統合リファクタリング計画です。

---

## 🏗️ アーキテクチャルールの要約

```
プレゼンテーション層（events/commands）
    ↓ Facadeを呼び出し（1対1）
Facade層
    ↓ Service層の協調、トランザクション境界管理
Service層
    ↓ Repository層を呼び出し、単一業務処理
Repository層
    ↓ データ永続化・取得
```

### 重要な制約
- **トランザクション管理**: Facade層でのみbegin/commit/rollback可能
- **層間アクセス**: 隣接層のみ呼び出し可能（層をスキップ禁止）
- **責務分離**: 各層は定義された責務のみ実行
- **ファイルサイズ**: 100行超の関数禁止

---

## 📊 違反の全体像

### 統計サマリー（全43ファイル）

| 層 | 違反カテゴリ | 違反ファイル数 | 重大度 |
|---|---|---|---|
| **Events層** | Repository直接アクセス | 14 | ⚠️ 極めて高 |
| **Events層** | トランザクション管理 | 7 | ⚠️ 極めて高 |
| **Events層** | Service直接アクセス | 5 | 🔶 高 |
| **Events層** | ビジネスロジック実装 | 6 | 🔶 中〜高 |
| **Facades層** | Repository直接アクセス | 13 | ⚠️ 極めて高 |
| **Services層** | トランザクション管理実装 | 1 | ⚠️ 極めて高 |
| **Services層** | DatabaseConnection保持 | 3 | 🔶 高 |
| **Services層** | 他Service直接依存 | 2 | 🔶 中 |

### 主な違反パターン
1. **Events層でのトランザクション管理**（Facade層の責務）
2. **Repository層の直接呼び出し**（Service層・Facade層をバイパス）
3. **複雑なビジネスロジックの実装**（曜日パース、データ整形など）
4. **Service層でのトランザクション管理**（Facade層の責務）
5. **Facade層でのRepository直接呼び出し**（Service層をバイパス）

### 最も深刻な違反TOP 3

1. 🔥 **scheduler.rs（Facades層）** - 20箇所以上のRepository直接アクセス
2. 🔥 **recruitment_schedule_create.rs（Events層）** - 521行の巨大ファイル
3. 🔥 **notification_service.rs（Services層）** - Service層でトランザクション管理

---

## 🎯 Phase 1: 最優先修正（重大度：極めて高）

### 1.1 Facades層の最重要違反修正

#### 1.1.1 scheduler.rs - 20箇所以上のRepository直接アクセス ⚠️

**ファイル**: `src/facades/scheduler.rs`

**現状の問題:**
- 複数のRepository（ScheduleRepository, NotificationRepository, BattleRecruitmentScheduleRepository等）を直接操作
- ビジネスロジックがFacade層に実装されている
- トランザクション管理は適切だが、Service層をバイパス

**修正方針:**
- **新規Service作成**: `src/services/schedule/scheduler_service.rs`
- Repository操作をすべてSchedulerServiceに移譲
- Facadeはトランザクション管理とServiceの協調のみ実行

**Before（Facade層）:**
```rust
pub async fn initialize_schedules(&self) -> Result<()> {
    let txn = self.app_state.db().begin().await?;

    // ❌ Repository層を直接操作
    let schedule_repo = ScheduleRepository::new();
    let notification_repo = NotificationRepository::new();
    let rel_repo = NotificationRelEventScheduleRepository::new();

    rel_repo.delete_all_with_txn(&txn).await?;
    notification_repo.delete_all_with_txn(&txn).await?;
    let event_schedules = schedule_repo.find_all_event_schedules(self.app_state.system_db()).await?;

    // ❌ 複雑なビジネスロジック（100行以上）
    for schedule in event_schedules {
        // ... 複雑な処理
    }

    txn.commit().await?;
    Ok(())
}
```

**After（Facade層）:**
```rust
pub async fn initialize_schedules(&self) -> Result<()> {
    let txn = self.app_state.db().begin().await?;

    // ✅ Service層を呼び出し
    let scheduler_service = SchedulerService::new();
    let result = scheduler_service.initialize_schedules(&txn, self.app_state).await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!("スケジュール初期化完了");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "スケジュール初期化失敗");
            Err(e)
        }
    }
}
```

**新規作成（Service層 - src/services/schedule/scheduler_service.rs）:**
```rust
pub struct SchedulerService;

impl SchedulerService {
    pub fn new() -> Self {
        Self
    }

    /// スケジュール初期化（ビジネスロジック）
    pub async fn initialize_schedules(
        &self,
        txn: &DatabaseTransaction,  // ✅ トランザクションを引数で受け取る
        app_state: &AppState,
    ) -> Result<()> {
        // Repository層を呼び出し
        let schedule_repo = ScheduleRepository::new();
        let notification_repo = NotificationRepository::new();
        let rel_repo = NotificationRelEventScheduleRepository::new();

        // 既存通知削除
        rel_repo.delete_all_with_txn(txn).await?;
        notification_repo.delete_all_with_txn(txn).await?;

        // イベントスケジュール取得
        let event_schedules = schedule_repo.find_all_event_schedules(app_state.system_db()).await?;

        // ビジネスロジック実行
        for schedule in event_schedules {
            self.process_event_schedule(txn, &schedule).await?;
        }

        Ok(())  // ✅ commit/rollbackはしない
    }

    async fn process_event_schedule(
        &self,
        txn: &DatabaseTransaction,
        schedule: &EventSchedule,
    ) -> Result<()> {
        // ビジネスロジック実装
        // ...
        Ok(())
    }
}
```

---

### 1.2 Events層のトランザクション管理違反（7ファイル）

**影響範囲**: トランザクション管理の責務がEvents層に漏れている

#### 対象ファイル:
1. `src/events/interactions/command_interactions/slash/recruitment_schedule_list.rs`
2. `src/events/interactions/command_interactions/slash/schedule_list.rs`
3. `src/events/interactions/command_interactions/slash/schedule_history.rs`
4. `src/events/interactions/command_interactions/slash/recruitment_schedule_delete.rs`
5. `src/events/interactions/command_interactions/slash/recruitment_schedule_toggle.rs`
6. `src/events/interactions/command_interactions/slash/gspread_push.rs`
7. `src/events/handlers/guild_create.rs`

#### 修正方針:
各ファイルについて、以下のパターンで修正：

**Before（Events層）:**
```rust
pub async fn command_handler(ctx: Context) -> Result<()> {
    let txn = db.begin().await?;  // ❌ Events層でトランザクション開始
    let result = repository.do_something(&txn).await?;
    txn.commit().await?;  // ❌ Events層でコミット
    Ok(())
}
```

**After（Events層）:**
```rust
pub async fn command_handler(ctx: Context) -> Result<()> {
    // ✅ Facade層を呼び出し
    let facade = SomeFacade::new(&ctx.data().app_state);
    let result = facade.execute_usecase(params).await?;
    ctx.say(format!("結果: {}", result)).await?;
    Ok(())
}
```

**新規作成（Facade層）:**
```rust
// facades/appropriate_name_facade.rs
pub async fn execute_usecase(&self, params: Params) -> Result<Output> {
    // ✅ トランザクション管理はFacade層の責務
    let txn = self.app_state.db().begin().await?;

    let result = async {
        let service = ServiceImpl::new();
        service.do_business_logic(&txn, params).await?;
        Ok(output)
    }.await;

    match result {
        Ok(output) => {
            txn.commit().await?;
            Ok(output)
        }
        Err(e) => {
            txn.rollback().await?;
            Err(e)
        }
    }
}
```

#### 具体的な修正タスク:

**1.2.1 RecruitmentScheduleFacade作成・拡張**
- ファイル: `src/facades/recruitment/recruitment_schedule_facade.rs`（既存の場合は拡張）
- 担当ユースケース:
  - `list_recruitment_schedules(user_id, guild_id, show_all)` - スケジュール一覧取得
  - `delete_recruitment_schedule(schedule_id, user_id)` - スケジュール削除
  - `toggle_recruitment_schedule(schedule_id, user_id)` - スケジュール有効/無効切替
- 修正対象Events:
  - `recruitment_schedule_list.rs`
  - `recruitment_schedule_delete.rs`
  - `recruitment_schedule_toggle.rs`

**1.2.2 NotificationScheduleFacade作成**
- 新規ファイル: `src/facades/schedule/notification_schedule_facade.rs`
- 担当ユースケース:
  - `list_future_notifications(guild_id)` - 未来の通知一覧
  - `list_notification_history(guild_id, from_date)` - 通知履歴一覧
- 修正対象Events:
  - `schedule_list.rs`
  - `schedule_history.rs`

**1.2.3 SpreadsheetExportFacade拡張**
- 既存ファイル: `src/facades/spreadsheet/spreadsheet_export_facade.rs`
- トランザクション管理をFacade層に移動
- 修正対象Events: `gspread_push.rs`

**1.2.4 GuildManagementFacade作成**
- 新規ファイル: `src/facades/guild/guild_management_facade.rs`
- 担当ユースケース:
  - `register_new_guild(guild_id, guild_name)` - 新規ギルド登録
- 修正対象Events: `guild_create.rs`

---

### 1.3 Events層の最重要違反: recruitment_schedule_create.rs（521行）

**ファイル**: `src/events/interactions/command_interactions/slash/recruitment_schedule_create.rs`

**現状の問題:**
- **521行の巨大ファイル**（ルール違反: 100行超禁止）
- Events層でトランザクション管理
- 複雑な曜日パース処理（parse_days、parse_continuous_days等）
- 時刻パース、タイムゾーン変換、バリデーションがEvents層に混在
- Repository層を直接呼び出し

**リファクタリング内容:**

新規作成するコンポーネント:
```
facades/recruitment/recruitment_schedule_facade.rs
  - create_recruitment_schedule() - 定期募集作成（既存なら拡張）
  - update_recruitment_schedule() - 定期募集更新

services/recruitment/schedule/
  - schedule_create_service.rs - 定期募集作成ビジネスロジック（既存）
  - days_parser_service.rs - 曜日パース処理（既存）
  - time_parser_service.rs - 時刻パース処理（既存）
```

**責務の分離:**
- Events層: コマンド引数受け取り、Facade呼び出し、結果表示
- Facade層: トランザクション管理、複数Serviceの協調
- Service層: 曜日パース、時刻パース、バリデーション、スケジュール生成ロジック
- Repository層: データ永続化

**期待される改善:**
- Eventsファイル: 521行 → 100行以下
- テスタビリティ向上（Service層を独立してテスト可能）
- 再利用性向上（パース処理を他コマンドでも利用可能）

#### Before（現状: 521行、アーキテクチャ違反多数）

```rust
// ❌ Events層でトランザクション管理、Repository直接呼び出し、ビジネスロジック実装
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

    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;

    // ❌ Repository層を直接呼び出し
    let quest_repo = SeaOrmQuestRepository::new();
    let search_results = quest_repo
        .search_by_name_or_alias(app_state.guild_db(), &quest)
        .await?;

    // ❌ ビジネスロジックをEvents層で実装
    let timezone_repo = Arc::new(GuildTimezoneRepository::new());
    let timezone_service = TimezoneService::new(timezone_repo);
    let timezone = timezone_service
        .get_guild_timezone(app_state.guild_db(), guild_id.get() as i64)
        .await?;

    // ❌ 複雑なパース処理（138-146行）
    let quest_start_time_local = parse_time(&quest_start_time)?;
    let recruit_start_time_local = parse_time(&recruit_start_time)?;
    let local_day_of_weeks = parse_days(&days)?;

    // ❌ バリデーション処理
    let schedule_service = RecruitmentScheduleService::new();
    schedule_service.validate_schedule_input(...)?;

    // ❌ UTC変換処理
    let (utc_quest_days, quest_start_time_tt) =
        convert_local_days_and_time_to_utc(...)?;

    // ❌ トランザクション開始（Facade層の責務）
    let txn = app_state.guild_db().begin().await?;

    // ❌ Repository層を直接呼び出し
    let schedule_repo = BattleRecruitmentScheduleRepository::new();
    let (schedule, _) = schedule_repo.create_with_txn(&txn, ...).await?;

    // ❌ コミット（Facade層の責務）
    txn.commit().await?;

    // ... メッセージ表示（約100行）
    Ok(())
}

// ❌ 複雑なパース関数（Events層に400行以上）
fn parse_days(days: &str) -> Result<Vec<DayOfWeek>> {
    // 100行以上のパース処理
}

fn parse_time(time: &str) -> Result<NaiveTime> {
    // 時刻パース処理
}

fn format_days(days: &[DayOfWeek]) -> String {
    // フォーマット処理
}
```

#### After（理想: 100行以下、クリーンアーキテクチャ遵守）

**Events層（recruitment_schedule_create.rs: 約80行）**

```rust
// ✅ Events層はFacade呼び出しと結果表示のみ
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
             **クエスト開始時刻**: {}\n\
             **募集開始時刻**: {}",
            result.schedule_name,
            result.schedule_id,
            result.quest_name,
            result.days_display,
            result.quest_start_time_display,
            result.recruit_start_time_display,
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

            // 2. スケジュール作成Service
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

        // 3. パーサーサービスを初期化
        let time_parser = TimeParserService::new();
        let days_parser = DaysParserService::new();

        // 4. 時刻・曜日パース
        let quest_start_time_local = time_parser.parse_time_string(quest_start_time)?;
        let recruit_start_time_local = time_parser.parse_time_string(recruit_start_time)?;
        let local_day_of_weeks = days_parser.parse_days_input(days_str)?;

        // 5. バリデーション
        let schedule_service = RecruitmentScheduleService::new();
        schedule_service.validate_schedule_input(
            &local_day_of_weeks,
            quest_start_time_local,
            recruit_day_offset,
            Some(recruit_start_time_local),
        )?;

        // 6. UTC変換
        let (utc_quest_days, quest_start_time_utc) =
            convert_local_days_and_time_to_utc(&local_day_of_weeks, quest_start_time_local, timezone)?;
        let (_, recruit_start_time_utc) =
            convert_local_days_and_time_to_utc(&local_day_of_weeks, recruit_start_time_local, timezone)?;

        // 7. チャンネル取得
        let channel = self.channel_repo
            .get_by_guild_and_type_with_txn(txn, guild_id, 2)
            .await?
            .ok_or_else(|| AppError::Business {
                message: "マルチ募集チャンネルが登録されていません".to_string(),
            })?;

        // 8. スケジュール保存
        let (schedule, _) = self.schedule_repo.create_with_txn(
            txn,  // ✅ トランザクションを渡す
            name.clone(),
            guild_id,
            channel.channel_id,
            quest.id,
            battle_style_id,
            quest_start_time_utc,
            recruit_day_offset,
            Some(recruit_start_time_utc),
            None,
            note.clone(),
            user_id,
            utc_quest_days.clone(),
        ).await?;

        // 9. 結果データ作成
        Ok(ScheduleCreationResult {
            schedule_id: schedule.id,
            schedule_name: name,
            quest_name: quest.name,
            days_display: DaysParserService::format_days(&local_day_of_weeks),
            quest_start_time_display: format!("{}", quest_start_time_local.format("%H:%M")),
            recruit_start_time_display: format!("{}", recruit_start_time_local.format("%H:%M")),
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

    fn parse_comma_separated(&self, days: &str) -> Result<Vec<DayOfWeek>> {
        // カンマ区切りパース実装
        // ...
    }

    fn parse_space_separated(&self, days: &str) -> Result<Vec<DayOfWeek>> {
        // スペース区切りパース実装
        // ...
    }

    fn parse_continuous_pattern(&self, days: &str) -> Result<Vec<DayOfWeek>> {
        // 連続パターンパース実装
        // ...
    }
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

---

### 1.4 Events層のRepository直接アクセス違反（14ファイル）

**影響範囲**: アーキテクチャの層分離原則に違反

#### 対象ファイル（優先度順）:
1. `recruitment_schedule_list.rs` - BattleRecruitmentScheduleRepository, SeaOrmQuestRepository, GuildTimezoneRepository
2. `schedule_list.rs` - NotificationRepository
3. `schedule_history.rs` - NotificationRepository
4. `recruitment_schedule_delete.rs` - BattleRecruitmentScheduleRepository
5. `recruitment_schedule_toggle.rs` - BattleRecruitmentScheduleRepository
6. `gspread_push.rs` - GuildSpreadsheetConfigRepository
7. `recruit_change_handler.rs` - SeaOrmQuestRepository, SeaOrmBattleStyleRepository
8. `guild_create.rs` - GuildRepository
9. `channel_register.rs` (autocomplete) - ChannelTypeRepository
10. `channel_unregister.rs` (autocomplete) - ChannelTypeRepository

#### 修正方針:
Repository呼び出しをすべてFacade層に移動。Events層はFacadeのみを呼び出す。

**修正タスク:**
- 1.2で作成したFacadeに統合
- Autocomplete関数は、Facade層に`get_autocomplete_data()`メソッドを追加

---

### 1.5 Facades層のRepository直接アクセス違反（13ファイル）

#### その他のFacades層違反（優先度：高）

**1.5.1 recruitment/new_recruit.rs**
- Repository: NotificationRepository, NotificationRelBattleRecruitmentRepository
- 新規Service: `NotificationManagementService`
- 責務: 通知の作成・リレーション作成

**1.5.2 recruitment/button_handler.rs**
- Repository: BattleRecruitmentsRepository, BattleStyleRepository, RecruitmentParticipantEntity直接操作
- 既存Serviceに統合: `RecruitmentQueryService`, `ParticipantsService`

**1.5.3 recruitment/cancel.rs**
- Repository: NotificationRelBattleRecruitmentRepository, NotificationRepository
- 既存Serviceに統合: `NotificationManagementService`（1.5.1で作成）

**1.5.4 recruitment/change.rs**
- Repository: 複数（Quest, BattleStyle, Notification関連）
- 既存Serviceに統合: `RecruitmentUpdateService`, `NotificationManagementService`

**1.5.5 recruitment/participants.rs**
- Repository: BattleRecruitmentsRepository, QuestRepository, ParticipantsRepository
- 既存Serviceに統合: `ParticipantsService`

**1.5.6 recruitment/role_management.rs**
- Repository: QuestRepository
- 新規Service: `QuestSearchService` または既存の`RoleNotificationService`に統合

**1.5.7 channel/channel_management_facade.rs**
- Repository: GuildRepository, ChannelTypeRepository, GuildChannelRepository
- 新規Service: `ChannelManagementService`

**1.5.8 timezone/timezone_facade.rs**
- Repository: GuildTimezoneRepository
- 既存Service拡張: `TimezoneService`にupsert操作を追加

**1.5.9～1.5.13 その他のFacade**
- `recruitment/recruitment_schedule_list.rs` - ScheduleQueryServiceに統合
- `recruitment/battle_style_list.rs` - BattleStyleServiceに統合
- `recruitment/quest_list.rs` - 既に適切（QuestSearchService使用）✅

---

### 1.6 Services層のトランザクション管理違反（1ファイル）

**対象ファイル**: `src/services/schedule/notification_service.rs`

**問題箇所**: 行72-92
```rust
// ❌ Service層でトランザクション管理
if send_result.is_ok() {
    let txn = self.db.begin().await?;  // ❌
    match self.notification_repo.mark_as_sent_with_txn(&txn, notification.id).await {
        Ok(_) => txn.commit().await?,  // ❌
        Err(e) => txn.rollback().await?,  // ❌
    }
}
```

**修正方針:**
1. NotificationServiceから`DatabaseConnection`フィールドを削除
2. トランザクション管理をFacade層（`SchedulerFacade`）に移動
3. `execute_scheduled_notifications`メソッドにトランザクションを引数として渡す

**修正後:**
```rust
// Service層
pub async fn execute_scheduled_notifications(
    &self,
    txn: &DatabaseTransaction,  // ✅ 引数で受け取る
    notifications: Vec<Notification>
) -> Result<ExecutionResult> {
    // 通知送信処理
    // Repository呼び出しにはtxnを渡す
    self.notification_repo.mark_as_sent_with_txn(txn, notification.id).await?;
    Ok(result)
}

// Facade層
pub async fn execute_notifications(&self) -> Result<()> {
    let txn = self.app_state.db().begin().await?;
    let notification_service = NotificationService::new();

    let result = notification_service.execute_scheduled_notifications(&txn, notifications).await;

    match result {
        Ok(_) => txn.commit().await?,
        Err(e) => txn.rollback().await?,
    }
    Ok(())
}
```

---

## 🎯 Phase 2: 高優先度修正（重大度：高）

### 2.1 Services層のDatabaseConnection保持違反（3ファイル）

#### 対象ファイル:
1. `src/services/schedule/notification_service.rs`
2. `src/services/schedule/notification_history_service.rs`
3. `src/services/spreadsheet/guild_spreadsheet_config_service.rs`

**問題**: Service層が`DatabaseConnection`をフィールドとして保持

**修正方針**:
- DatabaseConnectionフィールドを削除
- 各メソッドでトランザクションまたは接続を引数で受け取る
- コンストラクタから`DatabaseConnection`引数を削除

**Before:**
```rust
pub struct NotificationService {
    db: DatabaseConnection,  // ❌
    notification_repo: Arc<dyn NotificationRepository>,
}

impl NotificationService {
    pub fn new(db: DatabaseConnection) -> Self {  // ❌
        Self { db, notification_repo }
    }

    pub async fn execute(&self) -> Result<()> {
        let data = self.notification_repo.find_all(self.db).await?;  // ❌
    }
}
```

**After:**
```rust
pub struct NotificationService {
    notification_repo: Arc<dyn NotificationRepository>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self { notification_repo }
    }

    pub async fn execute(&self, txn: &DatabaseTransaction) -> Result<()> {
        let data = self.notification_repo.find_all(txn).await?;
    }
}
```

---

### 2.2 Events層のService直接アクセス違反（5ファイル）

#### 対象ファイル:
1. `recruitment_schedule_list.rs` - TimezoneService直接呼び出し
2. `timezone_show.rs` - TimezoneService直接呼び出し
3. `recruit_new_v2.rs` - TimezoneService直接呼び出し
4. `recruit_change.rs` - TimezoneService直接呼び出し
5. `recruit_change_date_modal.rs` - TimezoneService直接呼び出し

**修正方針:**
- TimezoneServiceの呼び出しをFacade層に移動
- Events層は対応するFacadeを呼び出すのみ

**パターン:**
```rust
// Before (Events層)
let timezone_repo = Arc::new(GuildTimezoneRepository::new());
let timezone_service = TimezoneService::new(timezone_repo);  // ❌
let timezone = timezone_service.get_guild_timezone(db, guild_id).await?;

// After (Events層)
let facade = RecruitmentFacade::new(&ctx.data().app_state);
let result = facade.create_recruitment(params).await?;  // タイムゾーン取得はFacade内部で実施
```

---

### 2.3 Services層の他Service直接依存違反（2ファイル）

#### 対象ファイル:
1. `src/services/recruitment/schedule/schedule_create_service.rs`
   - TimeParserService, DaysParserService, RecruitmentScheduleServiceを直接生成

2. `src/services/spreadsheet/global_loader_service.rs`
   - TableDefinitionService, DataConverterService, SpreadsheetReaderService, SchemaExtractorServiceを直接生成

**修正方針:**

**ケース1: ユーティリティService（状態なし）の場合**
- コンストラクタインジェクションで依存Serviceを受け取る
- またはモジュール関数として提供

**ケース2: ビジネスロジックServiceの場合**
- 複数Serviceの協調はFacade層に移動
- Service層では単一業務処理のみ実行

**修正例（schedule_create_service.rs）:**
```rust
// Before
pub async fn create_schedule(&self, params: Params) -> Result<()> {
    let time_parser = TimeParserService::new();  // ❌
    let days_parser = DaysParserService::new();  // ❌
    let time = time_parser.parse(params.time)?;
    let days = days_parser.parse(params.days)?;
}

// After (Option 1: コンストラクタインジェクション)
pub struct ScheduleCreateService {
    time_parser: TimeParserService,
    days_parser: DaysParserService,
}

pub async fn create_schedule(&self, params: Params) -> Result<()> {
    let time = self.time_parser.parse(params.time)?;
    let days = self.days_parser.parse(params.days)?;
}

// After (Option 2: モジュール関数化)
use crate::services::utils::{parse_time, parse_days};

pub async fn create_schedule(&self, params: Params) -> Result<()> {
    let time = parse_time(params.time)?;
    let days = parse_days(params.days)?;
}
```

---

## 🎯 Phase 3: 改善推奨（重大度：中）

### 3.1 Events層のビジネスロジック実装（6ファイル）

#### 対象ファイル:
1. `recruitment_schedule_list.rs` - UTC→ローカル変換、曜日フォーマット
2. `schedule_list.rs` - JST変換、フィルタリング、ソート
3. `schedule_history.rs` - ギルドフィルタリング、ソート
4. `recruitment_schedule_delete.rs` - 権限チェック
5. `recruitment_schedule_toggle.rs` - 権限チェック、状態反転
6. `timezone_show.rs` - デフォルト判定

**修正方針:**
- フィルタリング、ソート、変換ロジックをService層に移動
- 権限チェックは専用のPermissionServiceまたはFacade層で実施
- Events層は結果の表示のみ

**例（schedule_list.rs）:**
```rust
// Before (Events層)
let mut future_notifications: Vec<_> = notifications
    .into_iter()
    .filter(|n| n.schedule_datetime > now)  // ❌ ビジネスロジック
    .collect();
future_notifications.sort_by_key(|n| n.schedule_datetime);  // ❌
let display_count = future_notifications.len().min(10);  // ❌

// After (Events層)
let facade = NotificationScheduleFacade::new(&ctx.data().app_state);
let formatted_list = facade.get_future_notifications_formatted(guild_id).await?;
ctx.say(formatted_list).await?;

// 新規 (Service層)
pub async fn get_future_notifications(
    &self,
    txn: &DatabaseTransaction,
    guild_id: i64
) -> Result<Vec<Notification>> {
    let notifications = self.notification_repo.find_by_guild_id(txn, guild_id).await?;
    let future = notifications.into_iter()
        .filter(|n| n.schedule_datetime > Utc::now())
        .collect();
    future.sort_by_key(|n| n.schedule_datetime);
    Ok(future.into_iter().take(10).collect())
}
```

---

## 📋 実装順序

### Step 1: トランザクション管理の修正（1週間）

**優先度1: 最重要**
1. **scheduler.rs（Facade層）** - SchedulerService作成（最も影響範囲が大きい） ✅ 完了
2. **notification_service.rs（Service層）** - トランザクション管理修正 ✅ 完了

**優先度2: 定期募集関連（使用頻度が高い）**
3. RecruitmentScheduleFacade作成・拡張（1.2.1） ✅ 完了
4. Events層のトランザクション管理削除: ✅ 完了
   - `recruitment_schedule_list.rs` ✅
   - `recruitment_schedule_delete.rs` ✅
   - `recruitment_schedule_toggle.rs` ✅

**優先度3: 通知スケジュール関連**
5. NotificationScheduleFacade作成（1.2.2） ✅ 完了
6. Events層のトランザクション管理削除: ✅ 完了
   - `schedule_list.rs` ✅
   - `schedule_history.rs` ✅

**優先度4: その他**
7. GuildManagementFacade作成（1.2.4） ✅ 完了
8. SpreadsheetExportFacade拡張（1.2.3） ✅ 完了
9. Events層のトランザクション管理削除: ✅ 完了
   - `guild_create.rs` ✅
   - `gspread_push.rs` ✅

### Step 2: Repository直接アクセスの修正（2週間）

**Phase 2.1: 巨大ファイルの分割**
1. **recruitment_schedule_create.rs** - 521行 → 100行以下（最優先） （着手予定）
   - DaysParserService, TimeParserService既存確認・活用
   - ScheduleCreateService既存確認・活用
   - RecruitmentScheduleFacade拡張

**Phase 2.2: Facade層のService作成（優先度順）**
2. NotificationManagementService作成（1.5.1）
3. その他のService層作成:
   - ChannelManagementService（1.5.7）
   - QuestSearchService（1.5.6）
   - ScheduleQueryService
   - BattleStyleService

**Phase 2.3: Facade層のリファクタリング**
4. 各Facadeの修正（1.5.2〜1.5.13）

**Phase 2.4: Events層のRepository直接アクセス削除**
5. 残りのEventsファイル修正（1.4）

### Step 3: Service層の整理（1週間）
1. DatabaseConnection保持の削除（2.1）
2. 他Service直接依存の修正（2.3）
3. Events層のService直接呼び出し修正（2.2）

### Step 4: ビジネスロジック移動（1週間）
1. フィルタリング・ソートロジックのService層移動
2. 権限チェックの統合
3. フォーマット処理の整理

### Step 5: テスト作成
1. DaysParserService, TimeParserService の単体テスト（重要）
2. SchedulerService の単体テスト
3. 各Facade・Serviceの単体テスト
4. 統合テスト（必要に応じて）

---

## 📐 アーキテクチャ原則の再確認

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

// ❌ 間違い: Events層でトランザクション管理
pub async fn command_handler(...) -> Result<()> {
    let txn = db.begin().await?;  // NG
    repository.create_with_txn(&txn, ...).await?;
    txn.commit().await?;  // NG
}

// ❌ 間違い: Service層でトランザクション管理
pub async fn service_method(conn: &DatabaseConnection, ...) -> Result<T> {
    let txn = conn.begin().await?;  // NG
    repository.create_with_txn(&txn, ...).await?;
    txn.commit().await?;  // NG
}
```

### 層間の依存関係

```
Events → Facade → Service → Repository
  ✅       ✅        ✅         ✅

Events → Service → Repository
  ❌       ❌         ✅

Events → Repository
  ❌         ❌

Facade → Repository
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

// ❌ 間違い: DatabaseConnectionを保持
pub struct Service {
    db: DatabaseConnection,  // NG
}

// ❌ 間違い: 他Serviceを直接生成
pub async fn service_method(...) -> Result<T> {
    let other_service = OtherService::new();  // NG
    other_service.do_something(...).await?;
}
```

---

## ✅ 理想的な実装例

### Events層: recruit_cancel.rs（79行）

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
        Ok(CanCancelResult::NotAuthorized) => {
            ctx.send(
                poise::CreateReply::default()
                    .content("この募集をキャンセルする権限がありません。")
                    .ephemeral(true),
            ).await?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}
```

**優れている点:**
- Events層は79行のみ
- Facade層の関数のみを呼び出し
- UI処理（メッセージ表示）に専念
- ビジネスロジックへの直接アクセスなし
- トランザクション管理なし

### Facades層: recruitment_schedule_facade.rs

既に良好な実装：
- Service層の協調が適切
- トランザクション管理を正しく実装
- Repository層への直接アクセスなし

### Services層: recruitment_participants_service.rs

既に良好な実装：
- トランザクションを引数で受け取る
- DatabaseConnectionを保持しない
- 単一責務、適切なトランザクション処理

---

## 🧪 テスト戦略

### 各Phaseでの確認事項

**Phase 1完了時:**
- [x] すべてのトランザクション管理がFacade層に存在（対象ユースケース群について）
- [x] Events層にbegin/commit/rollbackが存在しない（対象ファイル群について）
- [x] Service層にbegin/commit/rollbackが存在しない（scheduler/notification他、対応済み範囲）
- [x] scheduler.rsがService層を経由してRepository層にアクセス

**Phase 2完了時:**
- [ ] Facade層がRepository層を直接呼び出していない
- [ ] Service層がDatabaseConnectionを保持していない
- [ ] Events層がService層を直接呼び出していない
- [ ] 100行を超えるEventsファイルが0件

**Phase 3完了時:**
- [ ] Events層にビジネスロジックが存在しない
- [ ] 各層の責務が明確に分離されている
- [ ] architecture.mdのルールに100%準拠

### テスト方法
1. 既存の統合テストがすべてパス
2. 各コマンドを手動実行して動作確認
3. スケジューラーの動作確認（定期実行）
4. エラーハンドリングの確認（ロールバック動作）
5. 新規作成したService層の単体テスト実行

---

## ⚠️ リスク管理

### 高リスク領域

1. **scheduler.rs** - 複雑な処理、影響範囲大
   - 対策: 段階的リファクタリング、十分なテスト
   - スケジュール実行を停止してからデプロイ

2. **トランザクション境界** - ロールバック漏れのリスク
   - 対策: match式でのエラーハンドリング徹底
   - コードレビューでトランザクション管理を重点確認

3. **通知システム** - 実行中の通知への影響
   - 対策: デプロイ時の通知停止、実行後の確認
   - 通知履歴の検証

4. **大規模変更による既存機能への影響**
   - 対策: 各コマンドごとに段階的に修正、テスト実施
   - 機能ごとにブランチを分けて段階的にマージ

### 後方互換性
- ✅ 既存のDiscordコマンドのインターフェースは変更しない
- ✅ データベーススキーマの変更は不要
- ✅ 既存の設定ファイルへの影響なし
- ✅ 既存のリポジトリインターフェースは維持

---

## 🎯 成功基準

### Phase 1完了時（重大度：極めて高）
- [x] Events層のトランザクション管理違反: 対象5ファイル解消（残はなし）
- [x] Service層のトランザクション管理違反: 1→0ファイル（notification_service）
- [x] scheduler.rsのRepository直接アクセス違反: 20箇所→0箇所（SchedulerServiceへ移譲）
- [ ] すべてのテストがパス

### Phase 2完了時（重大度：高）
- [ ] Facade層のRepository直接アクセス違反: 13→0ファイル
- [ ] Events層のRepository直接アクセス違反: 14→0ファイル
  - [x] autocomplete.rs の Timezone 候補取得を Facade 経由に統一（Repo/Service直参照排除）
  - [x] channel_register.rs の チャンネル種別オートコンプリートを Facade/Service 経由に統一
  - [x] recruit_change_handler.rs の クエスト/攻略方法取得を Facade 経由に統一
  - [ ] その他（participants 系など）
- [ ] Events層のService直接アクセス違反: 5→0ファイル
  - [x] autocomplete.rs（TimezoneService 直参照の排除 → TimezoneFacade）
  - [x] recruit_new.rs（Service 直呼びでのリアクション追加 → utils::discord_helper に置換）
  - [x] recruit_new_v2.rs（Timezone 取得を TimezoneFacade 経由に）
  - [x] recruit_change.rs（Timezone 取得を TimezoneFacade 経由に）
  - [x] timezone_show.rs（Timezone 取得を TimezoneFacade 経由に）
- [~] Service層のDatabaseConnection保持: 3→0ファイル
  - [x] NotificationHistoryService（DatabaseConnectionフィールド削除・引数化）
  - [x] GuildSpreadsheetConfigService（既に解消済）
  - [ ] 残り1件（確認・対応）
- [ ] recruitment_schedule_create.rs: 521行→100行以下（段階的縮小中：Embed生成をサービス化・エラーハンドリング簡素化 済）

### Phase 3完了時（重大度：中）
- [ ] Events層のビジネスロジック実装: 6→0ファイル
- [ ] Service層の他Service直接依存: 適切に整理
- [ ] architecture.mdのルールに100%準拠

### 最終確認
- [ ] すべての統合テストがパス
- [ ] 手動テストで全機能が正常動作
- [ ] コードレビューで承認
- [ ] ドキュメントの更新完了
- [ ] アーキテクチャ違反: 43ファイル→0ファイル

---

## 📚 参考資料

- アーキテクチャルール: `docs/develop/rules/architecture.md`
- 元の計画1: `REFACTORING_PLAN.md`（Events層の詳細実装例）
- 元の計画2: `architecture_refactoring_plan_2025-12-15.md`（全層の調査結果）
- 調査レポート: 3つのExploreエージェント調査結果

---

## 🔄 期待される効果

### コード品質
- アーキテクチャルール違反: **43ファイル → 0ファイル**
- 平均Eventsファイル行数: 大幅削減（特にrecruitment_schedule_create.rs: 521行 → 100行以下）
- テスタビリティ向上: Service層を独立してテスト可能

### 保守性
- ビジネスロジックの再利用性向上
- 責務の明確化
- 変更容易性の向上
- 新機能追加時の影響範囲が明確

### 一貫性
- 全コマンドが同一のアーキテクチャパターンに従う
- トランザクション管理の一元化
- エラーハンドリングの統一

---

<!-- 進捗ログは本計画では保持しません（チェックリストのみ運用） -->
