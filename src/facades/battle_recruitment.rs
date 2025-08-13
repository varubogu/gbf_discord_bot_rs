use crate::types::PoiseContext;

/// 新しい募集を開始する
pub(crate) async fn new(ctx: &PoiseContext<'_>) {
    // パラメータのクエストからクエスト情報を取得
    // パラメータから日時を解析
    // クエストと日時からメッセージを作成
    // メッセージを送信
    // データベースに登録
    // メッセージにリアクションを付与
}

/// 募集内容を更新する
pub(crate) async fn information_update(ctx: &poise::serenity_prelude::Context) {
    // パラメータのクエストからクエスト情報を取得
    // パラメータから日時を解析
    // クエストと日時からメッセージを作成
    // メッセージを更新
    // データベースを更新
}

/// 参加者を更新する
pub(crate) async fn member_update(ctx: &poise::serenity_prelude::Context) {
    // 募集メッセージのリアクションとメンバーを取得
    // リアクションとメンバーからメッセージを作成
    // クエストと日時からメッセージを作成
}

/// 募集をキャンセルする
pub(crate) async fn cancel(ctx: &poise::serenity_prelude::Context) {
    // リアクションから参加者一覧取得
    // 募集メッセージをキャンセル済みメッセージに変えるためのメッセージ作成
    // キャンセル通知メッセージ作成（参加者にメンションを含む）
    // 元の募集メッセージに返信する形で「この募集はキャンセルされました」とメッセージを送信
}

/// 開始時間になった
pub(crate) async fn start(ctx: &poise::serenity_prelude::Context) {

}