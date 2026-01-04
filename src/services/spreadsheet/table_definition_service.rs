/// テーブル定義サービス
///
/// スプレッドシートの「テーブル名」シートを解析し、テーブル定義を取得します。
/// 設計書: docs/develop/design/spreadsheet/service_layer.md
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

use crate::errors::BusinessRuleError;

/// テーブル定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    /// シート名
    pub sheet_name: String,
    /// テーブル名
    pub table_name: String,
    /// テーブルのスコープ（グローバル/ギルドなど）
    pub table_scope: Option<String>,
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

impl FromStr for TableIO {
    type Err = BusinessRuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "in" => Ok(TableIO::In),
            "out" => Ok(TableIO::Out),
            "in,out" | "out,in" | "both" => Ok(TableIO::Both),
            _ => Err(BusinessRuleError::TableDefinitionError {
                table_name: "tablesシート".to_string(),
                reason: format!("不正なtable_io値: {s}"),
            }),
        }
    }
}

impl TableIO {
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

impl FromStr for TableType {
    type Err = BusinessRuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "reference" => Ok(TableType::Reference),
            "transaction" => Ok(TableType::Transaction),
            "history" => Ok(TableType::History),
            _ => Err(BusinessRuleError::TableDefinitionError {
                table_name: "tablesシート".to_string(),
                reason: format!("不正なtable_type値: {s}"),
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

    /// 利用可能なヘッダーマッピングを抽出
    fn build_header_mapping(header_row: &[String]) -> Result<HeaderMapping, BusinessRuleError> {
        let mut key_to_index: HashMap<String, usize> = HashMap::new();

        for (index, raw_key) in header_row.iter().enumerate() {
            let key = raw_key.trim();
            if key.is_empty() {
                continue;
            }
            key_to_index.insert(key.to_lowercase(), index);
        }

        HeaderMapping::try_from(key_to_index)
    }

    /// 行データをTableDefinitionに変換
    fn parse_row(
        row: &[String],
        mapping: &HeaderMapping,
        row_number: usize,
    ) -> Result<TableDefinition, BusinessRuleError> {
        let sheet_name =
            mapping
                .sheet_name(row)
                .ok_or_else(|| BusinessRuleError::TableDefinitionError {
                    table_name: "tablesシート".to_string(),
                    reason: format!("sheet_nameが空です（行: {row_number}）"),
                })?;

        let table_name =
            mapping
                .table_name(row)
                .ok_or_else(|| BusinessRuleError::TableDefinitionError {
                    table_name: "tablesシート".to_string(),
                    reason: format!("table_nameが空です（行: {row_number}）"),
                })?;

        let table_io_str =
            mapping
                .table_io(row)
                .ok_or_else(|| BusinessRuleError::TableDefinitionError {
                    table_name: "tablesシート".to_string(),
                    reason: format!("table_ioが空です（行: {row_number}）"),
                })?;

        let table_type_str =
            mapping
                .table_type(row)
                .ok_or_else(|| BusinessRuleError::TableDefinitionError {
                    table_name: "tablesシート".to_string(),
                    reason: format!("table_typeが空です（行: {row_number}）"),
                })?;

        let table_io = TableIO::from_str(table_io_str)?;
        let table_type = TableType::from_str(table_type_str)?;

        let table_scope = mapping.table_scope(row).map(|scope| scope.to_string());

        Ok(TableDefinition {
            sheet_name: sheet_name.to_string(),
            table_name: table_name.to_string(),
            table_scope,
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

/// ヘッダーマッピング情報
struct HeaderMapping {
    sheet_name_idx: usize,
    table_name_idx: usize,
    table_io_idx: usize,
    table_type_idx: usize,
    table_scope_idx: Option<usize>,
}

impl HeaderMapping {
    fn try_from(mut indices: HashMap<String, usize>) -> Result<Self, BusinessRuleError> {
        let sheet_name_idx = indices.remove("sheet_name").ok_or_else(|| {
            BusinessRuleError::TableDefinitionError {
                table_name: "tablesシート".to_string(),
                reason: "必須列 sheet_name が存在しません".to_string(),
            }
        })?;

        let table_name_idx = indices.remove("table_name").ok_or_else(|| {
            BusinessRuleError::TableDefinitionError {
                table_name: "tablesシート".to_string(),
                reason: "必須列 table_name が存在しません".to_string(),
            }
        })?;

        let table_io_idx =
            indices
                .remove("table_io")
                .ok_or_else(|| BusinessRuleError::TableDefinitionError {
                    table_name: "tablesシート".to_string(),
                    reason: "必須列 table_io が存在しません".to_string(),
                })?;

        let table_type_idx = indices.remove("table_type").ok_or_else(|| {
            BusinessRuleError::TableDefinitionError {
                table_name: "tablesシート".to_string(),
                reason: "必須列 table_type が存在しません".to_string(),
            }
        })?;

        let table_scope_idx = indices.remove("table_scope");

        Ok(Self {
            sheet_name_idx,
            table_name_idx,
            table_io_idx,
            table_type_idx,
            table_scope_idx,
        })
    }

    fn value_at<'a>(&self, row: &'a [String], index: usize) -> Option<&'a str> {
        row.get(index)
            .map(|value| value.trim())
            .and_then(|value| if value.is_empty() { None } else { Some(value) })
    }

    fn sheet_name<'a>(&self, row: &'a [String]) -> Option<&'a str> {
        self.value_at(row, self.sheet_name_idx)
    }

    fn table_name<'a>(&self, row: &'a [String]) -> Option<&'a str> {
        self.value_at(row, self.table_name_idx)
    }

    fn table_io<'a>(&self, row: &'a [String]) -> Option<&'a str> {
        self.value_at(row, self.table_io_idx)
    }

    fn table_type<'a>(&self, row: &'a [String]) -> Option<&'a str> {
        self.value_at(row, self.table_type_idx)
    }

    fn table_scope<'a>(&self, row: &'a [String]) -> Option<&'a str> {
        self.table_scope_idx.and_then(|idx| self.value_at(row, idx))
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
                table_name: "tablesシート".to_string(),
                reason: "シートが空です".to_string(),
            });
        }

        // ヘッダー行の解析
        let header_row = &rows[0];
        let mapping = Self::build_header_mapping(header_row)?;

        // 2行目は説明行としてスキップ、3行目以降をパース
        let mut definitions = Vec::new();
        for (index, row) in rows.iter().enumerate().skip(2) {
            let row_number = index + 1; // シート上の行番号（1始まり）

            // 行がすべて空の場合はスキップ
            if row.iter().all(|cell| cell.trim().is_empty()) {
                tracing::debug!(
                    row_index = row_number,
                    "空行のためテーブル定義をスキップします"
                );
                continue;
            }

            match Self::parse_row(row, &mapping, row_number) {
                Ok(def) => {
                    tracing::debug!(
                        table_name = %def.table_name,
                        table_io = ?def.table_io,
                        table_scope = ?def.table_scope,
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
                table_name: "tablesシート".to_string(),
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
                "sheet_name".to_string(),
                "table_name".to_string(),
                "table_scope".to_string(),
                "table_io".to_string(),
                "table_type".to_string(),
            ],
            vec![
                "シート名".to_string(),
                "テーブル名".to_string(),
                "テーブルの対象".to_string(),
                "入力・出力".to_string(),
                "テーブル種類".to_string(),
            ],
            vec![
                "マルチバトル戦術".to_string(),
                "battle_styles".to_string(),
                "guild".to_string(),
                "in".to_string(),
                "reference".to_string(),
            ],
            vec![
                "クエスト情報".to_string(),
                "quests".to_string(),
                "global".to_string(),
                "in,out".to_string(),
                "reference".to_string(),
            ],
        ];

        let result = service.parse_table_definitions(rows).await;
        assert!(result.is_ok());

        let definitions = result.unwrap();
        assert_eq!(definitions.len(), 2);
        assert_eq!(definitions[0].table_name, "battle_styles");
        assert_eq!(definitions[0].table_scope.as_deref(), Some("guild"));
        assert_eq!(definitions[0].table_io, TableIO::In);
        assert_eq!(definitions[1].table_name, "quests");
        assert_eq!(definitions[1].table_scope.as_deref(), Some("global"));
        assert_eq!(definitions[1].table_io, TableIO::Both);
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
        assert_eq!(
            TableType::from_str("reference").unwrap(),
            TableType::Reference
        );
        assert_eq!(
            TableType::from_str("transaction").unwrap(),
            TableType::Transaction
        );
        assert_eq!(TableType::from_str("history").unwrap(), TableType::History);
        assert!(TableType::from_str("invalid").is_err());
    }
}
