use sea_orm_migration::prelude::*;
use std::env;
use std::process;

#[async_std::main]
async fn main() {
    prepare_database_url();
    cli::run_cli(migration::Migrator).await;
}

fn prepare_database_url() {
    // 個別の環境変数から接続URLを構築
    let db_host = env::var("DB_HOST").unwrap_or_else(|_| {
        eprintln!("エラー: 環境変数 DB_HOST が設定されていません");
        process::exit(1);
    });

    let db_port = env::var("DB_PORT").unwrap_or_else(|_| {
        eprintln!("エラー: 環境変数 DB_PORT が設定されていません");
        process::exit(1);
    });

    let db_name = env::var("DB_NAME").unwrap_or_else(|_| {
        eprintln!("エラー: 環境変数 DB_NAME が設定されていません");
        process::exit(1);
    });

    // マイグレーション実行は常にADMINロールを使用
    let db_user = env::var("ADMIN_DB_USER").unwrap_or_else(|_| {
        eprintln!("エラー: 環境変数 ADMIN_DB_USER が設定されていません");
        eprintln!("ヒント: マイグレーション実行には ADMIN_DB_USER と ADMIN_DB_PASSWORD が必要です");
        process::exit(1);
    });

    let db_password = env::var("ADMIN_DB_PASSWORD").unwrap_or_else(|_| {
        eprintln!("エラー: 環境変数 ADMIN_DB_PASSWORD が設定されていません");
        process::exit(1);
    });

    eprintln!("マイグレーション実行: ADMINロールを使用 (user: {})", db_user);

    // 接続URLを構築
    let database_url = format!(
        "postgresql://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, db_name
    );

    // DATABASE_URLとして設定
    env::set_var("DATABASE_URL", database_url);
}
