//! マッチングチェックFacade
//!
//! 時間選択・クエスト選択後のマッチングチェックと通知を行う
//!
//! ## 注意
//!
//! 新しい設計では、マッチング処理は10秒間隔の周期タスクで実行されます。
//! このfacadeは互換性のために残していますが、即時マッチングは行いません。
//! 周期タスク（`AutoMatchingTaskExecutor`）でマッチングが検出されます。

use crate::types::Result;
use tracing::info;

/// 時間選択後のマッチングチェックと通知
///
/// ## 注意
///
/// 新しい設計では、マッチング処理は10秒間隔の周期タスクで実行されます。
/// この関数は互換性のために残していますが、即時マッチングは行いません。
/// ユーザーの選択は既にDBに保存されており、周期タスクでマッチングが検出されます。
#[allow(unused_variables)]
pub async fn check_and_notify_after_time_selection(
    guild_id: u64,
    user_id: u64,
    month: i32,
    day: i32,
    hours: Vec<i32>,
) -> Result<()> {
    info!(
        guild_id,
        user_id,
        month,
        day,
        hour_count = hours.len(),
        "時間選択が完了しました。マッチングは周期タスクで検出されます。"
    );

    // マッチングは10秒間隔の周期タスク（AutoMatchingTaskExecutor）で実行されます
    // この関数では何も行わず、即座に返します

    Ok(())
}

/// クエスト選択後のマッチングチェックと通知
///
/// ## 注意
///
/// 新しい設計では、マッチング処理は10秒間隔の周期タスクで実行されます。
/// この関数は互換性のために残していますが、即時マッチングは行いません。
/// ユーザーの選択は既にDBに保存されており、周期タスクでマッチングが検出されます。
#[allow(unused_variables)]
pub async fn check_and_notify_after_quest_selection(
    guild_id: u64,
    user_id: u64,
    quest_ids: Vec<i32>,
) -> Result<()> {
    info!(
        guild_id,
        user_id,
        quest_count = quest_ids.len(),
        "クエスト選択が完了しました。マッチングは周期タスクで検出されます。"
    );

    // マッチングは10秒間隔の周期タスク（AutoMatchingTaskExecutor）で実行されます
    // この関数では何も行わず、即座に返します

    Ok(())
}
