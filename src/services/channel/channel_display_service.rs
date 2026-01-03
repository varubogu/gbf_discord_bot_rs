use crate::repository::database::channel_type_repository::ChannelTypeRepository;
use crate::repository::database::guild_channel_repository::SeaOrmGuildChannelRepository;
use crate::types::Result;
use sea_orm::DatabaseTransaction;
use std::collections::HashMap;

/// チャンネル設定表示結果
#[derive(Debug, Clone)]
pub struct ChannelSettingsDisplay {
    pub settings: Vec<ChannelTypeSetting>,
}

/// チャンネル種別設定
#[derive(Debug, Clone)]
pub struct ChannelTypeSetting {
    pub channel_type_name: String,
    pub channel_id: Option<i64>,
}

/// チャンネル設定表示サービス
///
/// チャンネル設定の表示用データ整形を担当するサービス。
pub struct ChannelDisplayService;

impl ChannelDisplayService {
    pub fn new() -> Self {
        Self
    }

    /// ギルドのチャンネル設定を取得して表示用に整形
    ///
    /// # 引数
    /// - `txn`: データベーストランザクション
    /// - `guild_id`: ギルドID
    ///
    /// # 戻り値
    /// チャンネル設定表示データ
    pub async fn get_channel_settings(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<ChannelSettingsDisplay> {
        let channel_type_repo = ChannelTypeRepository::new();
        let guild_channel_repo = SeaOrmGuildChannelRepository::new();

        // 全チャンネル種別を取得
        let all_channel_types = channel_type_repo.get_all(txn).await?;

        // ギルドのチャンネル設定を取得
        let guild_channels = guild_channel_repo
            .get_all_by_guild_with_txn(txn, guild_id)
            .await?;

        // チャンネルIDでマップを作成
        let channel_map: HashMap<i32, i64> = guild_channels
            .iter()
            .map(|gc| (gc.channel_type, gc.channel_id))
            .collect();

        // 設定データを作成
        let settings = all_channel_types
            .into_iter()
            .map(|ct| ChannelTypeSetting {
                channel_type_name: ct.name.clone(),
                channel_id: channel_map.get(&ct.id).copied(),
            })
            .collect();

        Ok(ChannelSettingsDisplay { settings })
    }

    /// 設定表示データを文字列に整形
    ///
    /// # 引数
    /// - `display`: チャンネル設定表示データ
    ///
    /// # 戻り値
    /// 整形されたメッセージ文字列
    pub fn format_settings(&self, display: &ChannelSettingsDisplay) -> String {
        let mut lines = Vec::new();

        for setting in &display.settings {
            if let Some(channel_id) = setting.channel_id {
                lines.push(format!(
                    "• **{}**: <#{}>\n",
                    setting.channel_type_name, channel_id
                ));
            } else {
                lines.push(format!("• **{}**: 未設定\n", setting.channel_type_name));
            }
        }

        lines.join("")
    }
}

impl Default for ChannelDisplayService {
    fn default() -> Self {
        Self::new()
    }
}
