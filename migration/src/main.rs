use sea_orm_migration::prelude::*;
use std::env;
use std::process;

#[async_std::main]
async fn main() {
    prepare_migration_url();
    cli::run_cli(migration::Migrator).await;
}

fn prepare_migration_url() {
    match env::var("MIGRATION_URL") {
        Ok(url) => {
            env::set_var("DATABASE_URL", url);
        }
        Err(env::VarError::NotPresent) => {
            eprintln!("環境変数 MIGRATION_URL が設定されていません。マイグレーション用の接続URLを準備してください。");
            process::exit(1);
        }
        Err(env::VarError::NotUnicode(_)) => {
            eprintln!("環境変数 MIGRATION_URL に無効な値が指定されています。URLを再確認してください。");
            process::exit(1);
        }
    }
}
