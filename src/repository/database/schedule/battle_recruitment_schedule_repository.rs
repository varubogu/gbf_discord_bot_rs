use crate::models::entities::{battle_recruitment_schedule_days, battle_recruitment_schedules};
use crate::types::Result;
use sea_orm::prelude::TimeTime;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, Set,
};
use tracing::{debug, error};

/// マルチ募集スケジュールリポジトリ
pub struct BattleRecruitmentScheduleRepository;

impl BattleRecruitmentScheduleRepository {
    pub fn new() -> Self {
        Self
    }

    /// 有効な全スケジュールと曜日情報を取得
    pub async fn find_all_enabled_schedules_with_days(
        &self,
        db: &DatabaseConnection,
    ) -> Result<
        Vec<(
            battle_recruitment_schedules::Model,
            Vec<battle_recruitment_schedule_days::Model>,
        )>,
    > {
        debug!("有効な全スケジュールと曜日情報を取得します");

        // 有効なスケジュールを取得
        let schedules = battle_recruitment_schedules::Entity::find()
            .filter(battle_recruitment_schedules::Column::IsEnabled.eq(true))
            .all(db)
            .await
            .map_err(|e| {
                error!(error = %e, "スケジュールの取得に失敗しました");
                e
            })?;

        debug!(count = schedules.len(), "有効なスケジュールを取得しました");

        // 各スケジュールの曜日情報を取得
        let mut result = Vec::new();
        for schedule in schedules {
            let days = battle_recruitment_schedule_days::Entity::find()
                .filter(battle_recruitment_schedule_days::Column::ScheduleId.eq(schedule.id))
                .all(db)
                .await
                .map_err(|e| {
                    error!(error = %e, schedule_id = schedule.id, "曜日情報の取得に失敗しました");
                    e
                })?;

            result.push((schedule, days));
        }

        debug!(
            total_schedules = result.len(),
            "スケジュールと曜日情報の取得が完了しました"
        );
        Ok(result)
    }

    /// ギルドIDでスケジュール取得
    pub async fn find_by_guild_id<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
    ) -> Result<
        Vec<(
            battle_recruitment_schedules::Model,
            Vec<battle_recruitment_schedule_days::Model>,
        )>,
    >
    where
        C: sea_orm::ConnectionTrait,
    {
        debug!(guild_id = %guild_id, "ギルドIDでスケジュールを取得します");

        let schedules = battle_recruitment_schedules::Entity::find()
            .filter(battle_recruitment_schedules::Column::GuildId.eq(guild_id))
            .all(db)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id = %guild_id, "スケジュールの取得に失敗しました");
                e
            })?;

        // 各スケジュールの曜日情報を取得
        let mut result = Vec::new();
        for schedule in schedules {
            let days = battle_recruitment_schedule_days::Entity::find()
                .filter(battle_recruitment_schedule_days::Column::ScheduleId.eq(schedule.id))
                .all(db)
                .await
                .map_err(|e| {
                    error!(error = %e, schedule_id = schedule.id, "曜日情報の取得に失敗しました");
                    e
                })?;

            result.push((schedule, days));
        }

        debug!(count = result.len(), "ギルドのスケジュールを取得しました");
        Ok(result)
    }

    /// 作成者IDでスケジュール取得
    pub async fn find_by_created_by<'c, C>(
        &self,
        db: &'c C,
        created_by: i64,
    ) -> Result<
        Vec<(
            battle_recruitment_schedules::Model,
            Vec<battle_recruitment_schedule_days::Model>,
        )>,
    >
    where
        C: sea_orm::ConnectionTrait,
    {
        debug!(created_by = %created_by, "作成者IDでスケジュールを取得します");

        let schedules = battle_recruitment_schedules::Entity::find()
            .filter(battle_recruitment_schedules::Column::CreatedBy.eq(created_by))
            .all(db)
            .await
            .map_err(|e| {
                error!(error = %e, created_by = %created_by, "スケジュールの取得に失敗しました");
                e
            })?;

        // 各スケジュールの曜日情報を取得
        let mut result = Vec::new();
        for schedule in schedules {
            let days = battle_recruitment_schedule_days::Entity::find()
                .filter(battle_recruitment_schedule_days::Column::ScheduleId.eq(schedule.id))
                .all(db)
                .await
                .map_err(|e| {
                    error!(error = %e, schedule_id = schedule.id, "曜日情報の取得に失敗しました");
                    e
                })?;

            result.push((schedule, days));
        }

        debug!(count = result.len(), "作成者のスケジュールを取得しました");
        Ok(result)
    }

    /// IDでスケジュール取得
    pub async fn find_by_id<'c, C>(
        &self,
        db: &'c C,
        id: i32,
    ) -> Result<
        Option<(
            battle_recruitment_schedules::Model,
            Vec<battle_recruitment_schedule_days::Model>,
        )>,
    >
    where
        C: sea_orm::ConnectionTrait,
    {
        debug!(id = %id, "IDでスケジュールを取得します");

        let schedule = battle_recruitment_schedules::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| {
                error!(error = %e, id = %id, "スケジュールの取得に失敗しました");
                e
            })?;

        if let Some(schedule) = schedule {
            let days = battle_recruitment_schedule_days::Entity::find()
                .filter(battle_recruitment_schedule_days::Column::ScheduleId.eq(schedule.id))
                .all(db)
                .await
                .map_err(|e| {
                    error!(error = %e, schedule_id = schedule.id, "曜日情報の取得に失敗しました");
                    e
                })?;

            debug!("スケジュールを取得しました");
            Ok(Some((schedule, days)))
        } else {
            debug!("スケジュールが見つかりませんでした");
            Ok(None)
        }
    }

    /// スケジュールと曜日を作成（トランザクション使用）
    pub async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        name: String,
        guild_id: i64,
        channel_id: i64,
        quest_id: i32,
        battle_style_id: i32,
        quest_start_time: TimeTime,
        recruit_start_day_offset: i32,
        recruit_start_time: Option<TimeTime>,
        max_participants: Option<i32>,
        note: Option<String>,
        created_by: i64,
        day_of_weeks: Vec<i32>,
    ) -> Result<(
        battle_recruitment_schedules::Model,
        Vec<battle_recruitment_schedule_days::Model>,
    )> {
        debug!(
            name = %name,
            guild_id = %guild_id,
            channel_id = %channel_id,
            quest_id = %quest_id,
            "スケジュールを作成します"
        );

        let now = chrono::Utc::now();

        // スケジュールを作成
        let schedule_active_model = battle_recruitment_schedules::ActiveModel {
            id: sea_orm::NotSet,
            name: Set(name),
            guild_id: Set(guild_id),
            channel_id: Set(channel_id),
            quest_id: Set(quest_id),
            battle_style_id: Set(battle_style_id),
            quest_start_time: Set(quest_start_time),
            recruit_start_day_offset: Set(recruit_start_day_offset),
            recruit_start_time: Set(recruit_start_time),
            max_participants: Set(max_participants),
            note: Set(note),
            is_enabled: Set(true),
            created_by: Set(created_by),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let schedule = schedule_active_model.insert(txn).await.map_err(|e| {
            error!(error = %e, "スケジュールの作成に失敗しました");
            e
        })?;

        debug!(schedule_id = schedule.id, "スケジュールを作成しました");

        // 曜日情報を作成
        let mut days = Vec::new();
        for day_of_week in day_of_weeks {
            let day_active_model = battle_recruitment_schedule_days::ActiveModel {
                id: sea_orm::NotSet,
                schedule_id: Set(schedule.id),
                day_of_week: Set(day_of_week),
                created_at: Set(now),
                updated_at: Set(now),
            };

            let day = day_active_model.insert(txn).await.map_err(|e| {
                error!(error = %e, day_of_week = %day_of_week, "曜日情報の作成に失敗しました");
                e
            })?;

            days.push(day);
        }

        debug!(day_count = days.len(), "曜日情報を作成しました");

        Ok((schedule, days))
    }

    /// スケジュール更新（トランザクション使用）
    pub async fn update_with_txn(
        &self,
        txn: &DatabaseTransaction,
        id: i32,
        quest_id: Option<i32>,
        battle_style_id: Option<i32>,
        quest_start_time: Option<TimeTime>,
        recruit_start_day_offset: Option<i32>,
        recruit_start_time: Option<Option<TimeTime>>,
        max_participants: Option<Option<i32>>,
        note: Option<Option<String>>,
        day_of_weeks: Option<Vec<i32>>,
    ) -> Result<(
        battle_recruitment_schedules::Model,
        Vec<battle_recruitment_schedule_days::Model>,
    )> {
        debug!(id = %id, "スケジュールを更新します");

        // 既存のスケジュールを取得
        let schedule = battle_recruitment_schedules::Entity::find_by_id(id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, id = %id, "スケジュールの取得に失敗しました");
                e
            })?
            .ok_or_else(|| {
                error!(id = %id, "スケジュールが見つかりません");
                crate::types::AppError::NotFound(format!("スケジュールID {id} が見つかりません"))
            })?;

        let now = chrono::Utc::now();
        let mut active_model: battle_recruitment_schedules::ActiveModel = schedule.into();

        // 更新フィールドを設定
        if let Some(quest_id) = quest_id {
            active_model.quest_id = Set(quest_id);
        }
        if let Some(battle_style_id) = battle_style_id {
            active_model.battle_style_id = Set(battle_style_id);
        }
        if let Some(quest_start_time) = quest_start_time {
            active_model.quest_start_time = Set(quest_start_time);
        }
        if let Some(recruit_start_day_offset) = recruit_start_day_offset {
            active_model.recruit_start_day_offset = Set(recruit_start_day_offset);
        }
        if let Some(recruit_start_time) = recruit_start_time {
            active_model.recruit_start_time = Set(recruit_start_time);
        }
        if let Some(max_participants) = max_participants {
            active_model.max_participants = Set(max_participants);
        }
        if let Some(note) = note {
            active_model.note = Set(note);
        }
        active_model.updated_at = Set(now);

        let updated_schedule = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, id = %id, "スケジュールの更新に失敗しました");
            e
        })?;

        debug!(id = %id, "スケジュールを更新しました");

        // 曜日情報を更新（指定された場合）
        let days = if let Some(day_of_weeks) = day_of_weeks {
            // 既存の曜日情報を削除
            battle_recruitment_schedule_days::Entity::delete_many()
                .filter(battle_recruitment_schedule_days::Column::ScheduleId.eq(id))
                .exec(txn)
                .await
                .map_err(|e| {
                    error!(error = %e, id = %id, "既存の曜日情報の削除に失敗しました");
                    e
                })?;

            // 新しい曜日情報を作成
            let mut new_days = Vec::new();
            for day_of_week in day_of_weeks {
                let day_active_model = battle_recruitment_schedule_days::ActiveModel {
                    id: sea_orm::NotSet,
                    schedule_id: Set(id),
                    day_of_week: Set(day_of_week),
                    created_at: Set(now),
                    updated_at: Set(now),
                };

                let day = day_active_model.insert(txn).await.map_err(|e| {
                    error!(error = %e, day_of_week = %day_of_week, "曜日情報の作成に失敗しました");
                    e
                })?;

                new_days.push(day);
            }

            debug!(day_count = new_days.len(), "曜日情報を更新しました");
            new_days
        } else {
            // 曜日情報の更新なし、既存のものを取得
            battle_recruitment_schedule_days::Entity::find()
                .filter(battle_recruitment_schedule_days::Column::ScheduleId.eq(id))
                .all(txn)
                .await
                .map_err(|e| {
                    error!(error = %e, id = %id, "曜日情報の取得に失敗しました");
                    e
                })?
        };

        Ok((updated_schedule, days))
    }

    /// スケジュール削除（トランザクション使用、CASCADE により曜日も削除）
    pub async fn delete_with_txn(&self, txn: &DatabaseTransaction, id: i32) -> Result<()> {
        debug!(id = %id, "スケジュールを削除します");

        let result = battle_recruitment_schedules::Entity::delete_by_id(id)
            .exec(txn)
            .await
            .map_err(|e| {
                error!(error = %e, id = %id, "スケジュールの削除に失敗しました");
                e
            })?;

        if result.rows_affected == 0 {
            error!(id = %id, "スケジュールが見つかりません");
            return Err(crate::types::AppError::NotFound(format!(
                "スケジュールID {id} が見つかりません"
            )));
        }

        debug!(id = %id, "スケジュールを削除しました（CASCADE により曜日も削除）");
        Ok(())
    }

    /// 有効/無効の切り替え（トランザクション使用）
    pub async fn toggle_enabled_with_txn(
        &self,
        txn: &DatabaseTransaction,
        id: i32,
        is_enabled: bool,
    ) -> Result<battle_recruitment_schedules::Model> {
        debug!(id = %id, is_enabled = %is_enabled, "スケジュールの有効/無効を切り替えます");

        // 既存のスケジュールを取得
        let schedule = battle_recruitment_schedules::Entity::find_by_id(id)
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, id = %id, "スケジュールの取得に失敗しました");
                e
            })?
            .ok_or_else(|| {
                error!(id = %id, "スケジュールが見つかりません");
                crate::types::AppError::NotFound(format!("スケジュールID {id} が見つかりません"))
            })?;

        let now = chrono::Utc::now();
        let mut active_model: battle_recruitment_schedules::ActiveModel = schedule.into();
        active_model.is_enabled = Set(is_enabled);
        active_model.updated_at = Set(now);

        let updated_schedule = active_model.update(txn).await.map_err(|e| {
            error!(error = %e, id = %id, "スケジュールの更新に失敗しました");
            e
        })?;

        debug!(id = %id, is_enabled = %is_enabled, "スケジュールの有効/無効を切り替えました");
        Ok(updated_schedule)
    }
}
