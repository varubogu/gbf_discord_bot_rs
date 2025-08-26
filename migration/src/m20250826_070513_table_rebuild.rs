use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // テーブル削除（外部キー制約があるため先に削除）
        manager
            .drop_table(
                Table::drop()
                    .table(RecruitmentParticipants::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Bosses::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        // schedules テーブルを notifications に名前変更
        manager
            .rename_table(
                Table::rename()
                    .table(Schedules::Table, Notifications::Table)
                    .to_owned(),
            )
            .await?;

        // notifications テーブル（旧 schedules）の列変更
        // message_id を message_text_id に変更
        manager
            .alter_table(
                Table::alter()
                    .table(Notifications::Table)
                    .rename_column(Notifications::MessageId, Notifications::MessageTextId)
                    .to_owned(),
            )
            .await?;

        // parent_schedule_id と parent_schedule_detail_id を削除
        manager
            .alter_table(
                Table::alter()
                    .table(Notifications::Table)
                    .drop_column(Notifications::ParentScheduleId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Notifications::Table)
                    .drop_column(Notifications::ParentScheduleDetailId)
                    .to_owned(),
            )
            .await?;

        // message_texts テーブルの列名変更
        manager
            .alter_table(
                Table::alter()
                    .table(MessageTexts::Table)
                    .rename_column(MessageTexts::MessageId, MessageTexts::MessageTextId)
                    .to_owned(),
            )
            .await?;

        // event_schedule_details テーブルの列変更
        manager
            .alter_table(
                Table::alter()
                    .table(EventScheduleDetails::Table)
                    .rename_column(
                        EventScheduleDetails::MessageId,
                        EventScheduleDetails::MessageTextId,
                    )
                    .to_owned(),
            )
            .await?;

        // battle_recruitments テーブルの変更
        // target_id を quest_id に変更
        manager
            .alter_table(
                Table::alter()
                    .table(BattleRecruitments::Table)
                    .rename_column(BattleRecruitments::TargetId, BattleRecruitments::QuestId)
                    .to_owned(),
            )
            .await?;

        // expiry_date を quest_start_at に変更
        manager
            .alter_table(
                Table::alter()
                    .table(BattleRecruitments::Table)
                    .rename_column(
                        BattleRecruitments::ExpiryDate,
                        BattleRecruitments::QuestStartAt,
                    )
                    .to_owned(),
            )
            .await?;

        // is_recruiting 列追加
        manager
            .alter_table(
                Table::alter()
                    .table(BattleRecruitments::Table)
                    .add_column(boolean(BattleRecruitments::IsRecruiting).default(true))
                    .to_owned(),
            )
            .await?;

        // is_canceled 列追加
        manager
            .alter_table(
                Table::alter()
                    .table(BattleRecruitments::Table)
                    .add_column(boolean(BattleRecruitments::IsCanceled).default(false))
                    .to_owned(),
            )
            .await?;

        // quests テーブルの変更
        // target_id 列削除
        manager
            .alter_table(
                Table::alter()
                    .table(Quests::Table)
                    .drop_column(Quests::TargetId)
                    .to_owned(),
            )
            .await?;

        // quest_name を name に変更
        manager
            .alter_table(
                Table::alter()
                    .table(Quests::Table)
                    .rename_column(Quests::QuestName, Quests::Name)
                    .to_owned(),
            )
            .await?;

        // default_battle_type を default_battle_style に変更
        manager
            .alter_table(
                Table::alter()
                    .table(Quests::Table)
                    .rename_column(Quests::DefaultBattleType, Quests::DefaultBattleStyle)
                    .to_owned(),
            )
            .await?;

        // recruit_count 列追加
        manager
            .alter_table(
                Table::alter()
                    .table(Quests::Table)
                    .add_column(integer(Quests::RecruitCount).default(0))
                    .to_owned(),
            )
            .await?;

        // available_battle_styles 列追加
        manager
            .alter_table(
                Table::alter()
                    .table(Quests::Table)
                    .add_column(integer(Quests::AvailableBattleStyles).default(0))
                    .to_owned(),
            )
            .await?;

        // quest_aliases テーブルの変更
        // target_id を quest_id に変更
        manager
            .alter_table(
                Table::alter()
                    .table(QuestAliases::Table)
                    .rename_column(QuestAliases::TargetId, QuestAliases::QuestId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // up の逆操作を実装
        // quest_aliases の変更を戻す
        manager
            .alter_table(
                Table::alter()
                    .table(QuestAliases::Table)
                    .rename_column(QuestAliases::QuestId, QuestAliases::TargetId)
                    .to_owned(),
            )
            .await?;

        // quests テーブルの変更を戻す
        manager
            .alter_table(
                Table::alter()
                    .table(Quests::Table)
                    .drop_column(Quests::AvailableBattleStyles)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Quests::Table)
                    .drop_column(Quests::RecruitCount)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Quests::Table)
                    .rename_column(Quests::DefaultBattleStyle, Quests::DefaultBattleType)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Quests::Table)
                    .rename_column(Quests::Name, Quests::QuestName)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Quests::Table)
                    .add_column(integer(Quests::TargetId).default(0))
                    .to_owned(),
            )
            .await?;

        // battle_recruitments テーブルの変更を戻す
        manager
            .alter_table(
                Table::alter()
                    .table(BattleRecruitments::Table)
                    .drop_column(BattleRecruitments::IsCanceled)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BattleRecruitments::Table)
                    .drop_column(BattleRecruitments::IsRecruiting)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BattleRecruitments::Table)
                    .rename_column(
                        BattleRecruitments::QuestStartAt,
                        BattleRecruitments::ExpiryDate,
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BattleRecruitments::Table)
                    .rename_column(BattleRecruitments::QuestId, BattleRecruitments::TargetId)
                    .to_owned(),
            )
            .await?;

        // event_schedule_details テーブルの変更を戻す
        manager
            .alter_table(
                Table::alter()
                    .table(EventScheduleDetails::Table)
                    .rename_column(
                        EventScheduleDetails::MessageTextId,
                        EventScheduleDetails::MessageId,
                    )
                    .to_owned(),
            )
            .await?;

        // message_texts テーブルの変更を戻す
        manager
            .alter_table(
                Table::alter()
                    .table(MessageTexts::Table)
                    .rename_column(MessageTexts::MessageTextId, MessageTexts::MessageId)
                    .to_owned(),
            )
            .await?;

        // notifications テーブルの変更を戻す
        manager
            .alter_table(
                Table::alter()
                    .table(Notifications::Table)
                    .add_column(integer_null(Notifications::ParentScheduleDetailId))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Notifications::Table)
                    .add_column(integer_null(Notifications::ParentScheduleId))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Notifications::Table)
                    .rename_column(Notifications::MessageTextId, Notifications::MessageId)
                    .to_owned(),
            )
            .await?;

        // notifications を schedules に名前変更
        manager
            .rename_table(
                Table::rename()
                    .table(Notifications::Table, Schedules::Table)
                    .to_owned(),
            )
            .await?;

        // 削除されたテーブルを再作成（実際の本番環境では注意が必要）
        // 注意: 実際のdown migration では、テーブル構造の詳細を正確に再現する必要があります

        Ok(())
    }
}

// テーブル識別子の定義
#[derive(DeriveIden)]
enum BattleRecruitments {
    Table,
    TargetId,
    QuestId,
    ExpiryDate,
    QuestStartAt,
    IsRecruiting,
    IsCanceled,
}

#[derive(DeriveIden)]
enum Quests {
    Table,
    TargetId,
    QuestName,
    Name,
    DefaultBattleType,
    DefaultBattleStyle,
    RecruitCount,
    AvailableBattleStyles,
}

#[derive(DeriveIden)]
enum QuestAliases {
    Table,
    TargetId,
    QuestId,
}

#[derive(DeriveIden)]
enum EventScheduleDetails {
    Table,
    MessageId,
    MessageTextId,
}

#[derive(DeriveIden)]
enum MessageTexts {
    Table,
    MessageId,
    MessageTextId,
}

#[derive(DeriveIden)]
enum Schedules {
    Table,
}

#[derive(DeriveIden)]
enum Notifications {
    Table,
    MessageId,
    MessageTextId,
    ParentScheduleId,
    ParentScheduleDetailId,
}

#[derive(DeriveIden)]
enum Bosses {
    Table,
}

#[derive(DeriveIden)]
enum RecruitmentParticipants {
    Table,
}

#[derive(DeriveIden)]
enum Users {
    Table,
}
