pub use sea_orm_migration::prelude::*;

mod m20251222_000000_init_complete_schema;
mod m20251224_000000_rename_to_guild_settings_and_add_locale;
mod m20251225_000000_move_battle_recruitment_schedules_to_guild_master;
mod m20251225_010000_create_scheduled_tasks;
mod m20251226_000000_add_recruitment_dismissal;
mod m20251228_000000_add_full_notification_sent_flag;
mod m20251229_000000_add_quest_sort_order_and_guild_quests;
mod m20251231_000000_rename_schedule_id_to_recruitment_schedule_id;
mod m20260117_000000_refactor_notifications_as_child_of_scheduled_tasks;
mod m20250117_000000_remove_notifications_schedule_datetime;
mod m20260122_000000_create_auto_recruitment;
mod m20260123_000000_add_auto_recruitment_flags;
mod m20260123_100000_refactor_auto_matching;
mod m20260216_000000_add_execution_status_to_scheduled_tasks;
mod m20260217_000000_add_notification_channel_id_to_event_schedule_details;
mod m20260222_000000_add_host_discord_user_id_to_battle_recruitments;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251222_000000_init_complete_schema::Migration),
            Box::new(m20251224_000000_rename_to_guild_settings_and_add_locale::Migration),
            Box::new(m20251225_000000_move_battle_recruitment_schedules_to_guild_master::Migration),
            Box::new(m20251225_010000_create_scheduled_tasks::Migration),
            Box::new(m20251226_000000_add_recruitment_dismissal::Migration),
            Box::new(m20251228_000000_add_full_notification_sent_flag::Migration),
            Box::new(m20251229_000000_add_quest_sort_order_and_guild_quests::Migration),
            Box::new(m20251231_000000_rename_schedule_id_to_recruitment_schedule_id::Migration),
            Box::new(m20260117_000000_refactor_notifications_as_child_of_scheduled_tasks::Migration),
            Box::new(m20250117_000000_remove_notifications_schedule_datetime::Migration),
            Box::new(m20260122_000000_create_auto_recruitment::Migration),
            Box::new(m20260123_000000_add_auto_recruitment_flags::Migration),
            Box::new(m20260123_100000_refactor_auto_matching::Migration),
            Box::new(m20260216_000000_add_execution_status_to_scheduled_tasks::Migration),
            Box::new(m20260217_000000_add_notification_channel_id_to_event_schedule_details::Migration),
            Box::new(m20260222_000000_add_host_discord_user_id_to_battle_recruitments::Migration),
        ]
    }
}
