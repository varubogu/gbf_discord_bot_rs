use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "master", table_name = "channel_types")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub name: String,
    pub memo: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// ギルドチャンネル種別
///
/// `guild_master.guild_channels` テーブルの `channel_type` カラムに格納される値。
/// master.channel_types テーブルのマスターレコードと対応する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GuildChannelType {
    /// イベントスケジュール通知チャンネル（id=1）
    EventNotification = 1,
    /// マルチ募集チャンネル（id=2）
    MultiRecruitment = 2,
    /// 管理者通知チャンネル（id=5）
    /// bot実行中のエラーや設定不足を管理者（gbf_bot_controlロール保持者）に通知するチャンネル
    AdminNotification = 5,
}

impl GuildChannelType {
    /// i32 値に変換
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// i32 値から変換（未知の値は None）
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::EventNotification),
            2 => Some(Self::MultiRecruitment),
            5 => Some(Self::AdminNotification),
            _ => None,
        }
    }
}
