use crate::repository::ChannelTypeRepository;
use crate::repository::database::channel_type_repository::SeaOrmChannelTypeRepository;
use crate::types::Result;
use crate::types::discord::AutocompleteOption;
use sea_orm::ConnectionTrait;

/// チャンネル種別クエリサービス
///
/// オートコンプリート等、読み取り系のユースケースを担当。
pub struct ChannelTypeQueryService;

impl ChannelTypeQueryService {
    pub fn new() -> Self {
        Self
    }

    /// オートコンプリート用にチャンネル種別一覧を取得（最大25件）
    pub async fn get_channel_types_for_autocomplete<C: ConnectionTrait + Send + Sync>(
        &self,
        db: &C,
    ) -> Result<Vec<AutocompleteOption>> {
        let repo = SeaOrmChannelTypeRepository::new();
        let items = repo.get_all(db).await?;

        let mut choices: Vec<AutocompleteOption> = items
            .into_iter()
            .map(|ct| AutocompleteOption::new(ct.name.clone(), ct.id.to_string()))
            .collect();
        choices.truncate(25);
        Ok(choices)
    }
}

impl Default for ChannelTypeQueryService {
    fn default() -> Self {
        Self::new()
    }
}
