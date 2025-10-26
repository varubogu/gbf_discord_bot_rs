/// テーブル定義サービス
///
/// スプレッドシートの「テーブル名」シートを解析し、テーブル定義を取得します。
/// 設計書: docs/develop/design/spreadsheet/service_layer.md

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::{BusinessRuleError, ExternalServiceError};

/// テーブル定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    /// テーブル日本語名（シート名として使用）
    pub table_name_jp: String,
    /// テーブル物理名（PostgreSQLテーブル名）
    pub table_name_en: String,
    /// 処理方向
    pub table_io: TableIO,
    /// テーブルタイプ
    pub table_type: TableType,
}

/// テーブルIO方向
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableIO {
    /// スプレッドシート → PostgreSQL（読み込み専用）
    In,
    /// PostgreSQL → スプレッドシート（書き込み専用）
    Out,
    /// 双方向（読み書き両対応）
    Both,
}

impl TableIO {
    /// 文字列から変換
    pub fn from_str(s: &str) -> Result<Self, BusinessRuleError> {
        match s.to_lowercase().as_str() {
            "in" => Ok(TableIO::In),
            "out" => Ok(TableIO::Out),
            "in,out" | "out,in" | "both" => Ok(TableIO::Both),
            _ => Err(BusinessRuleError::TableDefinitionError {
                table_name: "テーブル名シート".to_string(),
                reason: format!("不正なtable_io値: {}", s),
            }),
        }
    }

    /// 読み込み可能か
    pub fn can_import(&self) -> bool {
        matches!(self, TableIO::In | TableIO::Both)
    }

    /// 書き込み可能か
    pub fn can_export(&self) -> bool {
        matches!(self, TableIO::Out | TableIO::Both)
    }
}

/// テーブルタイプ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableType {
    /// 参照データ、マスターデータ
    Reference,
    /// トランザクションデータ
    Transaction,
    /// 履歴データ
    History,
}

impl TableType {
    /// 文字列から変換
    pub fn from_str(s: &str) -> Result<Self, BusinessRuleError> {
        match s.to_lowercase().as_str() {
            "reference" => Ok(TableType::Reference),
            "transaction" => Ok(TableType::Transaction),
            "history" => Ok(TableType::History),
            _ => Err(BusinessRuleError::TableDefinitionError {
                table_name: "テーブル名シート".to_string(),
                reason: format!("不正なtable_type値: {}", s),
            }),
        }
    }
}

/// テーブル定義サービストレイト
#[async_trait]
pub trait TableDefinitionServiceTrait: Send + Sync {
    /// 「テーブル名」シートからテーブル定義を解析
    ///
    /// # Arguments
    /// * `rows` - 「テーブル名」シートの行データ（1行目: ヘッダー、2行目以降: データ）
    async fn parse_table_definitions(
        &self,
        rows: Vec<Vec<String>>,
    ) -> Result<Vec<TableDefinition>, BusinessRuleError>;
}

/// テーブル定義サービス実装
#[derive(Clone)]
pub struct TableDefinitionService;

impl TableDefinitionService {
    pub fn new() -> Self {
        Self
    }

    /// 行データをTableDefinitionに変換
    fn parse_row(row: &[String]) -> Result<TableDefinition, BusinessRuleError> {
        // 列数チェック（最低4列必要: table_name_jp, table_name_en, table_io, table_type）
        if row.len() < 4 {
            return Err(BusinessRuleError::TableDefinitionError {
                table_name: "テーブル名シート".to_string(),
                reason: format!("列数が不足しています（必要: 4列, 実際: {}列）", row.len()),
            });
        }

        let table_name_jp = row[0].trim().to_string();
        let table_name_en = row[1].trim().to_string();
        let table_io_str = row[2].trim();
        let table_type_str = row[3].trim();

        // 必須フィールドチェック
        if table_name_jp.is_empty() {
            return Err(BusinessRuleError::TableDefinitionError {
                table_name: "テーブル名シート".to_string(),
                reason: "table_name_jpが空です".to_string(),
            });
        }

        if table_name_en.is_empty() {
            return Err(BusinessRuleError::TableDefinitionError {
                table_name: "テーブル名シート".to_string(),
                reason: "table_name_enが空です".to_string(),
            });
        }

        // table_ioとtable_typeをパース
        let table_io = TableIO::from_str(table_io_str)?;
        let table_type = TableType::from_str(table_type_str)?;

        Ok(TableDefinition {
            table_name_jp,
            table_name_en,
            table_io,
            table_type,
        })
    }
}

impl Default for TableDefinitionService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TableDefinitionServiceTrait for TableDefinitionService {
    async fn parse_table_definitions(
        &self,
        rows: Vec<Vec<String>>,
    ) -> Result<Vec<TableDefinition>, BusinessRuleError> {
        if rows.is_empty() {
            return Err(BusinessRuleError::TableDefinitionError {
                table_name: "テーブル名シート".to_string(),
                reason: "シートが空です".to_string(),
            });
        }

        // 1行目はヘッダーなのでスキップ、2行目以降をパース
        let mut definitions = Vec::new();
        for (index, row) in rows.iter().enumerate().skip(1) {
            match Self::parse_row(row) {
                Ok(def) => {
                    tracing::debug!(
                        table_name = %def.table_name_en,
                        table_io = ?def.table_io,
                        "テーブル定義を解析しました"
                    );
                    definitions.push(def);
                }
                Err(e) => {
                    tracing::warn!(
                        row_index = index + 1,
                        error = %e,
                        "テーブル定義の解析に失敗しました（スキップします）"
                    );
                    // エラー行はスキップして継続
                }
            }
        }

        if definitions.is_empty() {
            return Err(BusinessRuleError::TableDefinitionError {
                table_name: "テーブル名シート".to_string(),
                reason: "有効なテーブル定義が見つかりません".to_string(),
            });
        }

        tracing::info!(
            table_count = definitions.len(),
            "{}個のテーブル定義を読み込みました",
            definitions.len()
        );

        Ok(definitions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_table_definitions_success() {
        let service = TableDefinitionService::new();

        let rows = vec![
            vec![
                "table_name_jp".to_string(),
                "table_name_en".to_string(),
                "table_io".to_string(),
                "table_type".to_string(),
            ],
            vec![
                "クエスト情報".to_string(),
                "quests".to_string(),
                "in,out".to_string(),
                "reference".to_string(),
            ],
            vec![
                "マルチバトル戦術".to_string(),
                "battle_types".to_string(),
                "in".to_string(),
                "reference".to_string(),
            ],
        ];

        let result = service.parse_table_definitions(rows).await;
        assert!(result.is_ok());

        let definitions = result.unwrap();
        assert_eq!(definitions.len(), 2);
        assert_eq!(definitions[0].table_name_en, "quests");
        assert_eq!(definitions[0].table_io, TableIO::Both);
        assert_eq!(definitions[1].table_name_en, "battle_types");
        assert_eq!(definitions[1].table_io, TableIO::In);
    }

    #[tokio::test]
    async fn test_parse_table_definitions_empty() {
        let service = TableDefinitionService::new();
        let rows: Vec<Vec<String>> = vec![];

        let result = service.parse_table_definitions(rows).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_table_io_from_str() {
        assert_eq!(TableIO::from_str("in").unwrap(), TableIO::In);
        assert_eq!(TableIO::from_str("out").unwrap(), TableIO::Out);
        assert_eq!(TableIO::from_str("in,out").unwrap(), TableIO::Both);
        assert_eq!(TableIO::from_str("both").unwrap(), TableIO::Both);
        assert!(TableIO::from_str("invalid").is_err());
    }

    #[test]
    fn test_table_type_from_str() {
        assert_eq!(TableType::from_str("reference").unwrap(), TableType::Reference);
        assert_eq!(TableType::from_str("transaction").unwrap(), TableType::Transaction);
        assert_eq!(TableType::from_str("history").unwrap(), TableType::History);
        assert!(TableType::from_str("invalid").is_err());
    }
}
