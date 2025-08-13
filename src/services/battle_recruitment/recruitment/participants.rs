// use std::sync::Arc;
// use std::collections::{HashMap, HashSet};
// use poise::serenity_prelude::all::{Context, Message, User, UserId, ReactionType};
// use tracing::{error, info};
// 
// use crate::repository::Database;
// use crate::models::battle_recruitment::BattleRecruitment;
// use crate::types::BattleType;
// 
// pub struct ParticipantsService {
//     db: Arc<Database>,
// }
// 
// impl ParticipantsService {
//     pub fn new(db: Arc<Database>) -> Self {
//         Self { db }
//     }
// 
//     /// 募集の参加者情報を取得・管理する
//     /// Python版のafter_reaction.pyのget_reaction_users()に相当
//     pub async fn get_recruitment_participants(
//         &self,
//         ctx: &Context,
//         message: &Message,
//         battle_type: BattleType,
//     ) -> Result<Vec<User>, String> {
//         // リアクションユーザーを取得
//         let reaction_users = self.get_reaction_users(ctx, message, battle_type).await?;
//         
//         // Bot自身を除外
//         let bot_user_id = ctx.cache.current_user().id;
//         let participants: Vec<User> = reaction_users
//             .into_iter()
//             .filter(|user| user.id != bot_user_id)
//             .collect();
// 
//         info!("Found {} participants", participants.len());
//         Ok(participants)
//     }
// 
//     /// 特定の募集の参加者数をチェック
//     pub async fn check_recruitment_capacity(
//         &self,
//         ctx: &Context,
//         message: &Message,
//         recruitment: &BattleRecruitment,
//         battle_type: BattleType,
//     ) -> Result<bool, String> {
//         let participants = self.get_recruitment_participants(ctx, message, battle_type).await?;
//         
//         // クエスト情報から定員を取得
//         let quest = match self.db.get_quest_by_target_id(recruitment.target_id).await {
//             Ok(Some(quest)) => quest,
//             Ok(None) => {
//                 error!("Quest not found for target_id: {}", recruitment.target_id);
//                 return Ok(false);
//             },
//             Err(e) => {
//                 error!("Error fetching quest: {:?}", e);
//                 return Err(format!("Database error: {}", e));
//             }
//         };
// 
//         // デフォルトの定員数（実際の実装では quest から取得すべき）
//         let capacity = 6; // 一般的なGBFマルチバトルの定員
// 
//         info!("Participants: {}/{}", participants.len(), capacity);
//         Ok(participants.len() >= capacity)
//     }
// 
//     /// 参加者のユニークリストを取得
//     pub async fn get_unique_participants(
//         &self,
//         ctx: &Context,
//         message: &Message,
//         battle_type: BattleType,
//     ) -> Result<Vec<User>, String> {
//         let participants = self.get_recruitment_participants(ctx, message, battle_type).await?;
//         
//         // UserIdでユニーク化
//         let mut seen_ids = HashSet::new();
//         let unique_participants: Vec<User> = participants
//             .into_iter()
//             .filter(|user| seen_ids.insert(user.id))
//             .collect();
// 
//         Ok(unique_participants)
//     }
// 
//     /// 参加者リストをメンション形式で取得
//     pub async fn get_participants_mentions(
//         &self,
//         ctx: &Context,
//         message: &Message,
//         battle_type: BattleType,
//     ) -> Result<String, String> {
//         let participants = self.get_unique_participants(ctx, message, battle_type).await?;
//         
//         let mentions = participants
//             .iter()
//             .map(|user| format!("<@{}>", user.id))
//             .collect::<Vec<_>>()
//             .join(" ");
// 
//         Ok(mentions)
//     }
// 
//     /// メッセージからリアクションユーザーを取得
//     /// Python版のget_reaction_users()に相当
//     async fn get_reaction_users(
//         &self,
//         ctx: &Context,
//         message: &Message,
//         battle_type: BattleType,
//     ) -> Result<Vec<User>, String> {
//         let target_reactions = battle_type.reactions();
//         let mut users = Vec::new();
// 
//         for message_reaction in &message.reactions {
//             // 対象のリアクションかチェック
//             let is_target_reaction = target_reactions.iter().any(|target| {
//                 match target {
//                     crate::types::ReactionType::Unicode(emoji) => {
//                         message_reaction.reaction_type.unicode_eq(emoji)
//                     },
//                     crate::types::ReactionType::Custom { name, .. } => {
//                         message_reaction.reaction_type.as_data() == *name
//                     },
//                 }
//             });
// 
//             if is_target_reaction {
//                 // このリアクションのユーザーを取得
//                 match message_reaction.users(&ctx.http, None, None).await {
//                     Ok(reaction_users) => {
//                         users.extend(reaction_users);
//                     },
//                     Err(e) => {
//                         error!("Error getting reaction users: {:?}", e);
//                         // エラーが発生しても他のリアクションの処理を続行
//                     }
//                 }
//             }
//         }
// 
//         Ok(users)
//     }
// 
//     /// 参加者数の統計情報を取得
//     pub async fn get_participation_stats(
//         &self,
//         ctx: &Context,
//         message: &Message,
//         battle_type: BattleType,
//     ) -> Result<HashMap<String, usize>, String> {
//         let mut stats = HashMap::new();
//         let target_reactions = battle_type.reactions();
// 
//         for message_reaction in &message.reactions {
//             let is_target_reaction = target_reactions.iter().any(|target| {
//                 match target {
//                     crate::types::ReactionType::Unicode(emoji) => {
//                         message_reaction.reaction_type.unicode_eq(emoji)
//                     },
//                     crate::types::ReactionType::Custom { name, .. } => {
//                         message_reaction.reaction_type.as_data() == *name
//                     },
//                 }
//             });
// 
//             if is_target_reaction {
//                 let reaction_key = match &message_reaction.reaction_type {
//                     poise::serenity_prelude::ReactionType::Unicode(emoji) => emoji.clone(),
//                     poise::serenity_prelude::ReactionType::Custom { name, .. } => {
//                         name.clone().unwrap_or_else(|| "unknown".to_string())
//                     },
//                     _ => "unknown".to_string(),
//                 };
// 
//                 // Bot以外のユーザー数をカウント
//                 let bot_user_id = ctx.cache.current_user().id;
//                 let user_count = if message_reaction.count > 0 {
//                     // Botが含まれている場合は1を引く
//                     match message_reaction.users(&ctx.http, None, None).await {
//                         Ok(users) => users.iter().filter(|u| u.id != bot_user_id).count(),
//                         Err(_) => message_reaction.count as usize,
//                     }
//                 } else {
//                     0
//                 };
// 
//                 stats.insert(reaction_key, user_count);
//             }
//         }
// 
//         Ok(stats)
//     }
// }