use crate::gateway::DiscordGuildGateway;
use crate::repository::GuildEnvironmentRepository;
use crate::types::constants::ELEMENT_EMOJIS;
use crate::types::discord::{DiscordGuildId, GuildEmoji};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};

/// 環境変数キー定数
pub const ELEMENT_FIRE_KEY: &str = "ELEMENT_FIRE";
pub const ELEMENT_WATER_KEY: &str = "ELEMENT_WATER";
pub const ELEMENT_EARTH_KEY: &str = "ELEMENT_EARTH";
pub const ELEMENT_WIND_KEY: &str = "ELEMENT_WIND";
pub const ELEMENT_LIGHT_KEY: &str = "ELEMENT_LIGHT";
pub const ELEMENT_DARK_KEY: &str = "ELEMENT_DARK";

const ELEMENT_KEYS: [&str; 6] = [
    ELEMENT_FIRE_KEY,
    ELEMENT_WATER_KEY,
    ELEMENT_EARTH_KEY,
    ELEMENT_WIND_KEY,
    ELEMENT_LIGHT_KEY,
    ELEMENT_DARK_KEY,
];

/// 属性スタンプ解決結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementEmojis {
    pub fire: String,  // 火
    pub water: String, // 水
    pub earth: String, // 土
    pub wind: String,  // 風
    pub light: String, // 光
    pub dark: String,  // 闇
}

impl ElementEmojis {
    /// デフォルト値を使用して初期化
    pub fn default_emojis() -> Self {
        Self {
            fire: ELEMENT_EMOJIS[0].to_string(),
            water: ELEMENT_EMOJIS[1].to_string(),
            earth: ELEMENT_EMOJIS[2].to_string(),
            wind: ELEMENT_EMOJIS[3].to_string(),
            light: ELEMENT_EMOJIS[4].to_string(),
            dark: ELEMENT_EMOJIS[5].to_string(),
        }
    }

    /// インデックスで属性絵文字を取得（1-6）
    pub fn get_by_index(&self, index: usize) -> &str {
        match index {
            1 => &self.fire,
            2 => &self.water,
            3 => &self.earth,
            4 => &self.wind,
            5 => &self.light,
            6 => &self.dark,
            _ => ELEMENT_EMOJIS[0], // フォールバック
        }
    }

    /// 配列形式で取得（0-5）
    pub fn as_array(&self) -> [&str; 6] {
        [
            &self.fire,
            &self.water,
            &self.earth,
            &self.wind,
            &self.light,
            &self.dark,
        ]
    }
}

/// ギルド環境変数サービス
pub struct GuildEnvironmentService<R>
where
    R: GuildEnvironmentRepository,
{
    repository: Arc<R>,
}

impl<R: GuildEnvironmentRepository> GuildEnvironmentService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// ギルド固有の属性絵文字を取得（フォールバック機能付き）
    /// - 環境変数が存在しない場合: デフォルト値を使用
    /// - 絵文字が不正な形式の場合: デフォルト値を使用してログ警告
    /// - カスタム絵文字がサーバーに存在しない場合: デフォルト値を使用してログ警告
    pub async fn get_element_emojis<C, G>(
        &self,
        db: &C,
        gateway: &G,
        guild_id: i64,
    ) -> crate::types::Result<ElementEmojis>
    where
        C: sea_orm::ConnectionTrait,
        G: DiscordGuildGateway,
    {
        debug!(guild_id = guild_id, "属性絵文字設定を取得します");

        // 6つの環境変数を一括取得（パフォーマンス最適化）
        let env_map = self
            .repository
            .get_multiple_by_guild(db, guild_id, &ELEMENT_KEYS)
            .await
            .map_err(crate::types::AppError::Database)?;

        // サーバーの絵文字一覧を取得（カスタム絵文字の検証用）
        let guild_emojis = Self::fetch_guild_emojis(gateway, guild_id as u64).await;

        // デフォルト値でElementEmojisを初期化
        let mut emojis = ElementEmojis::default_emojis();
        let mut custom_count = 0;

        // カスタム値で上書き（存在し、かつ有効な絵文字の場合のみ）
        // :emoji_name: 形式の場合は <:emoji_name:id> に自動変換
        if let Some(fire) = env_map.get(ELEMENT_FIRE_KEY) {
            if let Some(resolved_emoji) = Self::resolve_emoji(fire, &guild_emojis) {
                emojis.fire = resolved_emoji;
                custom_count += 1;
            } else {
                warn!(
                    guild_id = guild_id,
                    key = ELEMENT_FIRE_KEY,
                    value = fire,
                    "絵文字が使用できないためデフォルト値を使用します（形式不正またはサーバーに存在しない）"
                );
            }
        }

        if let Some(water) = env_map.get(ELEMENT_WATER_KEY) {
            if let Some(resolved_emoji) = Self::resolve_emoji(water, &guild_emojis) {
                emojis.water = resolved_emoji;
                custom_count += 1;
            } else {
                warn!(
                    guild_id = guild_id,
                    key = ELEMENT_WATER_KEY,
                    value = water,
                    "絵文字が使用できないためデフォルト値を使用します（形式不正またはサーバーに存在しない）"
                );
            }
        }

        if let Some(earth) = env_map.get(ELEMENT_EARTH_KEY) {
            if let Some(resolved_emoji) = Self::resolve_emoji(earth, &guild_emojis) {
                emojis.earth = resolved_emoji;
                custom_count += 1;
            } else {
                warn!(
                    guild_id = guild_id,
                    key = ELEMENT_EARTH_KEY,
                    value = earth,
                    "絵文字が使用できないためデフォルト値を使用します（形式不正またはサーバーに存在しない）"
                );
            }
        }

        if let Some(wind) = env_map.get(ELEMENT_WIND_KEY) {
            if let Some(resolved_emoji) = Self::resolve_emoji(wind, &guild_emojis) {
                emojis.wind = resolved_emoji;
                custom_count += 1;
            } else {
                warn!(
                    guild_id = guild_id,
                    key = ELEMENT_WIND_KEY,
                    value = wind,
                    "絵文字が使用できないためデフォルト値を使用します（形式不正またはサーバーに存在しない）"
                );
            }
        }

        if let Some(light) = env_map.get(ELEMENT_LIGHT_KEY) {
            if let Some(resolved_emoji) = Self::resolve_emoji(light, &guild_emojis) {
                emojis.light = resolved_emoji;
                custom_count += 1;
            } else {
                warn!(
                    guild_id = guild_id,
                    key = ELEMENT_LIGHT_KEY,
                    value = light,
                    "絵文字が使用できないためデフォルト値を使用します（形式不正またはサーバーに存在しない）"
                );
            }
        }

        if let Some(dark) = env_map.get(ELEMENT_DARK_KEY) {
            if let Some(resolved_emoji) = Self::resolve_emoji(dark, &guild_emojis) {
                emojis.dark = resolved_emoji;
                custom_count += 1;
            } else {
                warn!(
                    guild_id = guild_id,
                    key = ELEMENT_DARK_KEY,
                    value = dark,
                    "絵文字が使用できないためデフォルト値を使用します（形式不正またはサーバーに存在しない）"
                );
            }
        }

        if custom_count > 0 {
            debug!(
                guild_id = guild_id,
                custom_count = custom_count,
                "カスタム属性絵文字を適用しました"
            );
        } else {
            debug!(
                guild_id = guild_id,
                "カスタム属性絵文字が設定されていないため、デフォルト値を使用します"
            );
        }

        Ok(emojis)
    }

    /// サーバーの絵文字一覧を取得（Gateway経由）
    /// 失敗した場合は空のHashMapを返す（フォールバック動作）
    async fn fetch_guild_emojis<G: DiscordGuildGateway>(
        gateway: &G,
        guild_id: u64,
    ) -> HashMap<u64, GuildEmoji> {
        let guild_id_obj = DiscordGuildId::new(guild_id);

        match gateway.get_emojis(guild_id_obj).await {
            Ok(emojis) => {
                debug!(
                    guild_id = guild_id,
                    emoji_count = emojis.len(),
                    "サーバー絵文字一覧を取得しました"
                );
                // GuildEmoji を HashMap に変換
                emojis
                    .into_iter()
                    .map(|emoji| (emoji.id.get(), emoji))
                    .collect()
            }
            Err(e) => {
                warn!(
                    guild_id = guild_id,
                    error = %e,
                    "サーバー絵文字の取得に失敗しました。カスタム絵文字の検証をスキップします"
                );
                HashMap::new()
            }
        }
    }

    /// 絵文字を解決して使用可能な形式に変換
    /// - Unicode絵文字: そのまま返す
    /// - <:name:id> 形式: サーバーに存在するか確認
    /// - :emoji_name: 形式: サーバーから名前で検索して <:name:id> に変換
    ///
    /// # 戻り値
    /// - Some(String): 使用可能な絵文字（変換済み）
    /// - None: 使用不可（デフォルト値にフォールバック）
    fn resolve_emoji(value: &str, guild_emojis: &HashMap<u64, GuildEmoji>) -> Option<String> {
        // 1. すでに <:name:id> または <a:name:id> 形式の場合
        if (value.starts_with("<:") || value.starts_with("<a:")) && value.ends_with('>') {
            if let Some(emoji_id) = Self::extract_custom_emoji_id(value) {
                if guild_emojis.is_empty() {
                    // サーバー絵文字の取得に失敗している場合は形式チェックのみで許可
                    debug!("サーバー絵文字が取得できていないため、形式チェックのみで許可します");
                    return Some(value.to_string());
                }

                if guild_emojis.contains_key(&emoji_id) {
                    debug!(emoji_id = emoji_id, "カスタム絵文字がサーバーに存在します");
                    return Some(value.to_string());
                } else {
                    debug!(
                        emoji_id = emoji_id,
                        "カスタム絵文字がサーバーに存在しません"
                    );
                    return None;
                }
            }
            return None;
        }

        // 2. Unicode絵文字の場合
        if value.chars().any(|c| c as u32 > 0x1F000) {
            debug!("Unicode絵文字として認識しました");
            return Some(value.to_string());
        }

        // 3. :emoji_name: 形式の場合、サーバー絵文字から名前で検索
        if value.starts_with(':') && value.ends_with(':') && value.len() > 2 {
            let emoji_name = &value[1..value.len() - 1]; // コロンを除去
            debug!(emoji_name = emoji_name, "カスタム絵文字名として検索します");

            if guild_emojis.is_empty() {
                debug!("サーバー絵文字が取得できていないため、名前による解決をスキップします");
                return None;
            }

            // サーバー絵文字から名前で検索
            for (emoji_id, emoji) in guild_emojis {
                if emoji.name == emoji_name {
                    // 見つかった！<:name:id> 形式に変換
                    let resolved = if emoji.animated {
                        format!("<a:{emoji_name}:{emoji_id}>")
                    } else {
                        format!("<:{emoji_name}:{emoji_id}>")
                    };
                    debug!(
                        emoji_name = emoji_name,
                        emoji_id = emoji_id,
                        resolved = %resolved,
                        "絵文字名をカスタム絵文字形式に変換しました"
                    );
                    return Some(resolved);
                }
            }

            debug!(
                emoji_name = emoji_name,
                "指定された名前のカスタム絵文字がサーバーに見つかりませんでした"
            );
            return None;
        }

        // 4. どの形式にも該当しない
        debug!(value = value, "不正な絵文字形式です");
        None
    }

    /// カスタム絵文字からIDを抽出
    /// 形式: <:emoji_name:123456789> または <a:emoji_name:123456789>
    /// 戻り値: Some(123456789) または None
    fn extract_custom_emoji_id(value: &str) -> Option<u64> {
        // カスタム絵文字の形式: <:name:id> または <a:name:id>
        if (value.starts_with("<:") || value.starts_with("<a:")) && value.ends_with('>') {
            // 最後のコロン以降の数字を抽出
            if let Some(last_colon_pos) = value.rfind(':') {
                let id_str = &value[last_colon_pos + 1..value.len() - 1]; // '>' を除外
                return id_str.parse::<u64>().ok();
            }
        }
        None
    }
}
