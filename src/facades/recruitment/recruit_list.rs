use crate::infrastructure::database::session::set_current_guild_id;
use crate::repository::BattleRecruitmentsRepository;
use crate::services::recruitment::quest_query_service::QuestQueryService;
use crate::services::timezone_service::TimezoneService;
use crate::types::Result;
use crate::types::{AppError, AppState};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use sea_orm::TransactionTrait;
use tracing::{error, info};

/// 募集一覧の1件分のデータ
#[derive(Debug, Clone)]
pub struct RecruitmentListItem {
    /// クエスト名（取得失敗時は「不明なクエスト」）
    pub quest_name: String,
    /// クエスト出発日時（UTC）
    pub quest_start_at: DateTime<Utc>,
    /// Discord チャンネルID
    pub channel_id: u64,
    /// Discord メッセージID
    pub message_id: u64,
}

/// 募集一覧ファサードの返却型
#[derive(Debug)]
pub struct RecruitListResult {
    /// 募集一覧（出発日時昇順）
    pub items: Vec<RecruitmentListItem>,
    /// ギルドのタイムゾーン（日時表示用）
    pub timezone: Tz,
}

/// 現在募集中のバトル一覧を取得するファサード
///
/// # 処理の流れ
/// 1. guild_db でトランザクションを開始
/// 2. set_current_guild_id で RLS 設定
/// 3. TimezoneService でギルドタイムゾーンを取得
/// 4. BattleRecruitmentsRepository::get_active_by_guild_with_txn で一覧取得
/// 5. ロールバック（読み取り専用）
/// 6. QuestQueryService で各クエスト名を取得（global_db 経由、RLS 不要）
///
/// # DB 接続の使い分け
/// - `app_state.guild_db()` … worker スキーマ（battle_recruitments, guild_settings）
/// - `app_state.global_db()` … master スキーマ（quests）
pub async fn list_active_recruitments(
    app_state: &AppState,
    guild_id: i64,
) -> Result<RecruitListResult> {
    info!(guild_id = guild_id, "募集一覧の取得を開始します");

    // guild_db トランザクションを開始
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLS 設定
    if let Err(e) = set_current_guild_id(&txn, guild_id).await {
        let _ = txn.rollback().await;
        return Err(AppError::Database(e));
    }

    // タイムゾーン取得
    let timezone_service = TimezoneService::new(app_state.repositories.guild_settings);
    let timezone = match timezone_service
        .get_guild_timezone_with_txn(&txn, guild_id)
        .await
    {
        Ok(tz) => tz,
        Err(e) => {
            let _ = txn.rollback().await;
            return Err(e);
        }
    };

    // 募集中バトル一覧取得
    let recruitments = match app_state
        .repositories
        .battle_recruitments
        .get_active_by_guild_with_txn(&txn, guild_id)
        .await
    {
        Ok(list) => list,
        Err(e) => {
            let _ = txn.rollback().await;
            return Err(e);
        }
    };

    info!(
        guild_id = guild_id,
        count = recruitments.len(),
        "募集一覧の取得に成功しました"
    );

    // 読み取り専用のためロールバック
    let _ = txn.rollback().await;

    // クエスト名を取得（global_db 経由）
    let quest_service = QuestQueryService::new(app_state.repositories.quest);
    let global_db = app_state.global_db();

    let mut items = Vec::with_capacity(recruitments.len());
    for r in recruitments {
        let quest_name = match quest_service.get_quest_by_id(global_db, r.quest_id).await {
            Ok(quest) => quest.name,
            Err(e) => {
                error!(
                    error = %e,
                    quest_id = r.quest_id,
                    "クエスト名の取得に失敗しました。フォールバックを使用します"
                );
                "不明なクエスト".to_string()
            }
        };

        items.push(RecruitmentListItem {
            quest_name,
            quest_start_at: r.quest_start_at,
            channel_id: r.channel_id,
            message_id: r.message_id,
        });
    }

    Ok(RecruitListResult { items, timezone })
}
