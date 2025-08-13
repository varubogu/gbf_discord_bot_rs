// use std::sync::Arc;
// use poise::serenity_prelude::all::{Context, Message, ChannelId};
// use tracing::{error, info, warn};
// 
// use crate::repository::Database;
// use crate::models::battle_recruitment::BattleRecruitment;
// 
// pub struct CancelRecruitmentService {
//     db: Arc<Database>,
// }
// 
// impl CancelRecruitmentService {
//     pub fn new(db: Arc<Database>) -> Self {
//         Self { db }
//     }
// 
//     /// 募集をキャンセルする
//     /// 指定されたメッセージの募集を削除し、キャンセル通知を送信
//     pub async fn cancel_recruitment_by_message(
//         &self,
//         ctx: &Context,
//         guild_id: u64,
//         channel_id: u64,
//         message_id: u64,
//     ) -> Result<(), String> {
//         // データベースから募集情報を取得
//         let recruitment = match self.db.get_battle_recruitment(
//             guild_id as i64,
//             channel_id as i64,
//             message_id as i64,
//         ).await {
//             Ok(Some(recruitment)) => recruitment,
//             Ok(None) => {
//                 warn!("Recruitment not found for message: {}", message_id);
//                 return Err("募集が見つかりませんでした。".to_string());
//             },
//             Err(e) => {
//                 error!("Error fetching recruitment: {:?}", e);
//                 return Err("データベースエラーが発生しました。".to_string());
//             }
//         };
// 
//         // 募集を削除
//         self.delete_recruitment(recruitment.id).await?;
// 
//         // キャンセル通知を送信
//         self.send_cancel_notification(ctx, channel_id, &recruitment).await?;
// 
//         info!("Successfully cancelled recruitment: {}", recruitment.id);
//         Ok(())
//     }
// 
//     /// 募集IDで募集をキャンセルする
//     pub async fn cancel_recruitment_by_id(
//         &self,
//         ctx: &Context,
//         recruitment_id: i64,
//     ) -> Result<(), String> {
//         // 募集情報を取得
//         let recruitment = match self.db.get_battle_recruitment_by_id(recruitment_id).await {
//             Ok(Some(recruitment)) => recruitment,
//             Ok(None) => {
//                 warn!("Recruitment not found: {}", recruitment_id);
//                 return Err("募集が見つかりませんでした。".to_string());
//             },
//             Err(e) => {
//                 error!("Error fetching recruitment: {:?}", e);
//                 return Err("データベースエラーが発生しました。".to_string());
//             }
//         };
// 
//         // 募集を削除
//         self.delete_recruitment(recruitment_id).await?;
// 
//         // キャンセル通知を送信
//         self.send_cancel_notification(ctx, recruitment.channel_id as u64, &recruitment).await?;
// 
//         info!("Successfully cancelled recruitment: {}", recruitment_id);
//         Ok(())
//     }
// 
//     /// 期限切れの募集を一括キャンセル
//     pub async fn cancel_expired_recruitments(&self, ctx: &Context) -> Result<usize, String> {
//         // 期限切れの募集を取得
//         let expired_recruitments = match self.db.get_expired_recruitments().await {
//             Ok(recruitments) => recruitments,
//             Err(e) => {
//                 error!("Error fetching expired recruitments: {:?}", e);
//                 return Err("期限切れ募集の取得に失敗しました。".to_string());
//             }
//         };
// 
//         let mut cancelled_count = 0;
// 
//         for recruitment in expired_recruitments {
//             match self.delete_recruitment(recruitment.id).await {
//                 Ok(_) => {
//                     // 期限切れ通知を送信
//                     if let Err(e) = self.send_expiry_notification(
//                         ctx, 
//                         recruitment.channel_id as u64, 
//                         &recruitment
//                     ).await {
//                         error!("Failed to send expiry notification: {}", e);
//                         // 通知の失敗は継続処理を止めない
//                     }
//                     cancelled_count += 1;
//                     info!("Cancelled expired recruitment: {}", recruitment.id);
//                 },
//                 Err(e) => {
//                     error!("Failed to cancel expired recruitment {}: {}", recruitment.id, e);
//                     // 一つの失敗で全体を止めずに継続
//                 }
//             }
//         }
// 
//         info!("Cancelled {} expired recruitments", cancelled_count);
//         Ok(cancelled_count)
//     }
// 
//     /// ユーザーの募集を全てキャンセル（管理者機能）
//     pub async fn cancel_user_recruitments(
//         &self,
//         ctx: &Context,
//         guild_id: u64,
//         user_id: u64,
//     ) -> Result<usize, String> {
//         // ユーザーの募集を取得（実際の実装では作成者フィールドが必要）
//         // 今回は簡単のため、全募集から対象を絞り込む代わりにエラーを返す
//         warn!("cancel_user_recruitments is not implemented yet");
//         Err("ユーザー別キャンセル機能は未実装です。".to_string())
//     }
// 
//     /// データベースから募集を削除
//     async fn delete_recruitment(&self, recruitment_id: i64) -> Result<(), String> {
//         match self.db.delete_battle_recruitment(recruitment_id).await {
//             Ok(_) => Ok(()),
//             Err(e) => {
//                 error!("Error deleting recruitment: {:?}", e);
//                 Err("募集の削除に失敗しました。".to_string())
//             }
//         }
//     }
// 
//     /// キャンセル通知を送信
//     async fn send_cancel_notification(
//         &self,
//         ctx: &Context,
//         channel_id: u64,
//         recruitment: &BattleRecruitment,
//     ) -> Result<(), String> {
//         let cancel_message = format!(
//             "募集がキャンセルされました。\n募集ID: {}",
//             recruitment.id
//         );
// 
//         match ChannelId::from(channel_id).say(&ctx.http, cancel_message).await {
//             Ok(_) => {
//                 info!("Sent cancel notification for recruitment: {}", recruitment.id);
//                 Ok(())
//             },
//             Err(e) => {
//                 error!("Failed to send cancel notification: {:?}", e);
//                 Err("キャンセル通知の送信に失敗しました。".to_string())
//             }
//         }
//     }
// 
//     /// 期限切れ通知を送信
//     async fn send_expiry_notification(
//         &self,
//         ctx: &Context,
//         channel_id: u64,
//         recruitment: &BattleRecruitment,
//     ) -> Result<(), String> {
//         let expiry_message = format!(
//             "募集が期限切れになりました。\n募集ID: {}",
//             recruitment.id
//         );
// 
//         match ChannelId::from(channel_id).say(&ctx.http, expiry_message).await {
//             Ok(_) => {
//                 info!("Sent expiry notification for recruitment: {}", recruitment.id);
//                 Ok(())
//             },
//             Err(e) => {
//                 error!("Failed to send expiry notification: {:?}", e);
//                 Err("期限切れ通知の送信に失敗しました。".to_string())
//             }
//         }
//     }
// }