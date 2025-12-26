/// スキーマユーティリティモジュール
///
/// このモジュールは、テーブル名からスキーマ名を取得する機能を提供します。
/// 実装はbuild.rsで自動生成されます。
// 自動生成されたコードを読み込む
mod generated;

// 自動生成された関数を再エクスポート
pub use generated::{get_entity_table_ref, get_schema_name};
