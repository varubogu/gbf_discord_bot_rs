use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 募集出発リマインダーメッセージを登録
        let insert = Query::insert()
            .into_table(MessageTexts::Table)
            .columns([
                MessageTexts::Id,
                MessageTexts::MessageJp,
                MessageTexts::MessageEn,
            ])
            .values_panic([
                "RECRUIT_DEPARTURE_REMINDER".into(),
                "⏰ まもなく出発時刻です！参加者の方はご準備ください。".into(),
                "⏰ Departure time is approaching! Please prepare if you are participating.".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // メッセージを削除
        let delete = Query::delete()
            .from_table(MessageTexts::Table)
            .and_where(Expr::col(MessageTexts::Id).eq("RECRUIT_DEPARTURE_REMINDER"))
            .to_owned();

        manager.exec_stmt(delete).await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum MessageTexts {
    Table,
    Id,
    MessageJp,
    MessageEn,
}
