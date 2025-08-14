use crate::types::{PoiseContext, BattleType};
use crate::services::battle_recruitment::recruitment::{
    NewRecruitmentService, UpdateRecruitmentService, ParticipantsService, 
    CancelRecruitmentService, StartRecruitmentService
};
use crate::repository::Database;
use crate::utils::database::DatabaseServiceExt;
use std::sync::Arc;
use tracing::{info, warn, error};
use chrono::Local;

/// 新しい募集を開始する
pub(crate) async fn new(ctx: &PoiseContext<'_>, quest_alias: &str, battle_type: BattleType) -> Result<(), String> {
    info!("battle_recruitment::new - 新しい募集を開始します");
    
    // 他の関連操作...
    // repository.set_recruitment_end_message(recruitment.id, some_message_id).await?;

    Ok(())
}

/// Builder パターンを使った例（execute_in_transactionを使用）
pub(crate) async fn new_with_builder(ctx: &PoiseContext<'_>, quest_alias: &str, battle_type: BattleType) -> Result<(), String> {
    info!("battle_recruitment::new_with_builder - 新しい募集を開始します");

    let db_service = &ctx.data().db;

    // ラムダ式を使用したトランザクション管理
    let battle_recruitment = db_service.execute_in_transaction(|_txn| async move {
        // リポジトリインスタンスを作成
        let repository = Database::new().await
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("リポジトリ作成エラー: {}", e))) as crate::types::PoiseError)?;

        // 募集を作成
        let recruitment = repository.battle_recruitment.create(
            ctx.guild_id().unwrap().get() as i64,
            ctx.channel_id().get() as i64,
            12345, // TODO: 実際のメッセージIDを使用
            1,     // TODO: 実際のtarget_idを使用
            battle_type as i32,
            chrono::Utc::now() + chrono::Duration::hours(1),
        ).await
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("募集作成エラー: {}", e))) as crate::types::PoiseError)?;

        // 他の関連操作...
        // repository.set_recruitment_end_message(recruitment.id, some_message_id).await?;

        Ok(recruitment)
    }).await
    .map_err(|e| format!("トランザクションエラー: {}", e))?;

    info!("新しい募集が正常に作成されました: id={}", battle_recruitment.id);
    Ok(())
}

/// 募集内容を更新する
pub(crate) async fn information_update(ctx: &poise::serenity_prelude::Context, guild_id: u64, channel_id: u64, message_id: u64, new_content: Option<String>) -> Result<(), String> {
    info!("battle_recruitment::information_update - 募集内容を更新します");
    
    // データベース接続
    let db = match Database::new().await {
        Ok(database) => Arc::new(database),
        Err(e) => {
            error!("データベース接続エラー: {}", e);
            return Err(format!("データベース接続エラー: {}", e));
        }
    };

    // UpdateRecruitmentServiceのインスタンス作成
    let service = UpdateRecruitmentService::new(db);
    
    // 募集メッセージ更新処理を実行
    match service.update_recruitment_message(ctx, guild_id, channel_id, message_id, new_content, None).await {
        Ok(_) => {
            info!("募集内容が正常に更新されました: message_id={}", message_id);
            Ok(())
        },
        Err(e) => {
            error!("募集内容更新エラー: {}", e);
            Err(e)
        }
    }
}

/// 参加者を更新する
pub(crate) async fn member_update(ctx: &poise::serenity_prelude::Context, guild_id: u64, channel_id: u64, message_id: u64) -> Result<(), String> {
    info!("battle_recruitment::member_update - 参加者を更新します");
    
    // ParticipantsServiceのインスタンス作成
    let service = ParticipantsService::new();
    
    // 募集メッセージのリアクションとメンバーを取得
    let participants = match service.get_reactions_and_members(message_id).await {
        Ok(participants) => participants,
        Err(e) => {
            error!("参加者取得エラー: {}", e);
            return Err(e);
        }
    };
    
    // DBから募集情報を取得
    if let Err(e) = service.get_recruitment_from_db(guild_id, channel_id, message_id).await {
        error!("DB募集情報取得エラー: {}", e);
        return Err(e);
    }
    
    // 参加者メッセージを作成
    let participant_message = match service.create_participant_message(&participants, "サンプルクエスト").await {
        Ok(message) => message,
        Err(e) => {
            error!("参加者メッセージ作成エラー: {}", e);
            return Err(e);
        }
    };
    
    // メッセージを更新
    match service.update_message(channel_id, message_id, &participant_message).await {
        Ok(_) => {
            info!("参加者更新処理が完了しました: message_id={}", message_id);
            Ok(())
        },
        Err(e) => {
            error!("メッセージ更新エラー: {}", e);
            Err(e)
        }
    }
}

/// 募集をキャンセルする
pub(crate) async fn cancel(ctx: &poise::serenity_prelude::Context, guild_id: u64, channel_id: u64, message_id: u64) -> Result<(), String> {
    info!("battle_recruitment::cancel - 募集をキャンセルします");
    
    // CancelRecruitmentServiceのインスタンス作成
    let service = CancelRecruitmentService::new();
    
    // DBから募集情報を取得
    if let Err(e) = service.get_recruitment_from_db(guild_id, channel_id, message_id).await {
        error!("DB募集情報取得エラー: {}", e);
        return Err(e);
    }
    
    // リアクションから参加者一覧取得
    let participants = match service.get_participants_from_reactions(message_id).await {
        Ok(participants) => participants,
        Err(e) => {
            error!("参加者取得エラー: {}", e);
            return Err(e);
        }
    };
    
    // 募集メッセージをキャンセル済みメッセージに変えるためのメッセージ作成
    let cancelled_message = match service.create_cancelled_message("元の募集内容").await {
        Ok(message) => message,
        Err(e) => {
            error!("キャンセル済みメッセージ作成エラー: {}", e);
            return Err(e);
        }
    };
    
    // キャンセル通知メッセージ作成（参加者にメンションを含む）
    let notification_message = match service.create_cancel_notification(&participants).await {
        Ok(message) => message,
        Err(e) => {
            error!("キャンセル通知メッセージ作成エラー: {}", e);
            return Err(e);
        }
    };
    
    // 元の募集メッセージに返信する形でメッセージを送信
    match service.send_cancel_reply(channel_id, message_id, &notification_message).await {
        Ok(_) => {
            info!("募集キャンセル処理が完了しました: message_id={}", message_id);
            Ok(())
        },
        Err(e) => {
            error!("キャンセル返信送信エラー: {}", e);
            Err(e)
        }
    }
}

/// 開始時間になった
pub(crate) async fn start(ctx: &poise::serenity_prelude::Context, guild_id: u64, channel_id: u64, message_id: u64, recruitment_id: i64) -> Result<(), String> {
    info!("battle_recruitment::start - 募集を開始します");
    
    // StartRecruitmentServiceのインスタンス作成
    let service = StartRecruitmentService::new();
    
    // DBから募集情報を取得
    if let Err(e) = service.get_recruitment_from_db(guild_id, channel_id, message_id).await {
        error!("DB募集情報取得エラー: {}", e);
        return Err(e);
    }
    
    // リアクションから参加者一覧取得
    let participants = match service.get_participants_from_reactions(message_id).await {
        Ok(participants) => participants,
        Err(e) => {
            error!("参加者取得エラー: {}", e);
            return Err(e);
        }
    };
    
    // 開始メッセージを作成（参加者へのメンション含む）
    let start_message = match service.create_start_message("サンプルクエスト", &participants).await {
        Ok(message) => message,
        Err(e) => {
            error!("開始メッセージ作成エラー: {}", e);
            return Err(e);
        }
    };
    
    // 元の募集メッセージに返信する形でメッセージを送信
    if let Err(e) = service.send_start_reply(channel_id, message_id, &start_message).await {
        error!("開始返信送信エラー: {}", e);
        return Err(e);
    }
    
    // 募集を開始済み状態に更新
    match service.mark_recruitment_as_started(recruitment_id).await {
        Ok(_) => {
            info!("募集開始処理が完了しました: message_id={}", message_id);
            Ok(())
        },
        Err(e) => {
            error!("募集開始済み状態更新エラー: {}", e);
            Err(e)
        }
    }
}