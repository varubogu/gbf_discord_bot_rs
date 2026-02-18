use crate::infrastructure::database::repositories::guild::SeaOrmGuildMessageTextRepository;
use crate::infrastructure::database::repositories::master_data::SeaOrmMessageTextRepository;
use crate::services::message::MessageService;

/// アプリケーション標準のメッセージサービス型
pub type AppMessageService =
    MessageService<SeaOrmGuildMessageTextRepository, SeaOrmMessageTextRepository>;

/// アプリケーション標準のメッセージサービスを生成する
pub fn create_message_service() -> AppMessageService {
    MessageService::new(
        SeaOrmGuildMessageTextRepository::new(),
        SeaOrmMessageTextRepository::new(),
    )
}
