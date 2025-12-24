use sea_orm::DatabaseBackend;
use sea_orm::{ConnectionTrait, DbErr, Statement};

/// 現在のギルドIDをセッション変数に設定
///
/// RLSポリシーで使用されるため、トランザクション開始直後に必ず呼び出す必要があります。
/// この関数は、PostgreSQLのセッション変数`app.current_guild_id`を設定し、
/// Row Level Security (RLS)ポリシーによるギルドデータの分離を実現します。
///
/// # 引数
/// * `conn` - データベース接続またはトランザクション
/// * `guild_id` - 設定するギルドID
///
/// # エラー
/// セッション変数の設定に失敗した場合、`DbErr`を返します。
///
/// # 使用例
/// ```no_run
/// use gbf_discord_bot_rs::infrastructure::database::db_helper::set_current_guild_id;
/// use sea_orm::TransactionTrait;
/// # async fn example(db: &sea_orm::DatabaseConnection, guild_id: i64) -> Result<(), sea_orm::DbErr> {
/// let txn = db.begin().await?;
/// set_current_guild_id(&txn, guild_id).await?;
/// // ... トランザクション内での処理
/// txn.commit().await?;
/// # Ok(())
/// # }
/// ```
pub async fn set_current_guild_id<C>(conn: &C, guild_id: i64) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    // SET LOCALを使用することで、トランザクション内でのみ有効な変数として設定
    // トランザクション終了後は自動的にリセットされる
    let sql = format!("SET LOCAL app.current_guild_id = '{guild_id}'");

    conn.execute(Statement::from_string(DatabaseBackend::Postgres, sql))
        .await?;

    Ok(())
}
