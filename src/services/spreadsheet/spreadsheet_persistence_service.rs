/// スプレッドシートデータ永続化Service
///
/// スプレッドシートから読み込んだデータをPostgreSQLに永続化する処理を担当
/// 責務:
/// - データベースへのINSERT/UPSERT/DELETE処理
/// - 参照整合性を考慮した削除処理
/// - PostgresValueからSeaValueへの変換
use chrono::Utc;
use sea_orm::sea_query::{
    Alias, ArrayType, Expr, PostgresQueryBuilder, Query, Value as SeaValue,
};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseTransaction, DbErr, Statement};

use crate::errors::FacadeError;
use crate::services::spreadsheet::{
    ColumnSchema, PostgresType, PostgresValue, RowData, get_entity_table_ref, get_schema_name,
};

/// データ永続化結果
#[derive(Debug)]
pub struct PersistResult {
    /// 挿入された行数
    pub inserted_rows: usize,
    /// 警告メッセージ
    pub warnings: Vec<String>,
}

/// スプレッドシートデータ永続化Service
pub struct SpreadsheetPersistenceService;

impl SpreadsheetPersistenceService {
    pub fn new() -> Self {
        Self
    }

    /// テーブルデータを永続化
    ///
    /// # 引数
    /// - `txn`: データベーストランザクション
    /// - `table_name`: テーブル名
    /// - `schema`: カラムスキーマ
    /// - `rows`: 行データ
    /// - `guild_id`: ギルドID（ギルド固有テーブルの場合）
    ///
    /// # 戻り値
    /// 挿入行数と警告メッセージ
    pub async fn persist_table_data(
        &self,
        txn: &DatabaseTransaction,
        table_name: &str,
        schema: &[ColumnSchema],
        rows: &[RowData],
        guild_id: Option<i64>,
    ) -> Result<PersistResult, FacadeError> {
        let mut warnings = Vec::new();
        let table_ref = get_entity_table_ref(table_name);

        // UPSERT対象テーブル以外は全削除してから挿入
        if !self.should_use_upsert(table_name) {
            let mut delete = Query::delete();
            delete.from_table(table_ref.clone());
            let (delete_sql, delete_values) = delete.build(PostgresQueryBuilder);

            txn.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                delete_sql,
                delete_values,
            ))
            .await
            .map_err(FacadeError::from)?;
        }

        if rows.is_empty() {
            // UPSERT対象テーブルの場合、空でも全削除が必要
            if self.should_use_upsert(table_name) {
                self.delete_unreferenced_records(txn, table_name, &[])
                    .await?;
            }
            return Ok(PersistResult {
                inserted_rows: 0,
                warnings,
            });
        }

        // created_atとupdated_atを除外したスキーマを作成
        let filtered_schema: Vec<_> = schema
            .iter()
            .filter(|col| col.column_name != "created_at" && col.column_name != "updated_at")
            .collect();

        // guild_idが指定されている場合、INSERT時にguild_idカラムを追加
        let insert_columns: Vec<Alias> = if guild_id.is_some() {
            let mut columns = vec![Alias::new("guild_id")];
            columns.extend(
                filtered_schema
                    .iter()
                    .map(|col| Alias::new(col.column_name.clone())),
            );
            columns
        } else {
            filtered_schema
                .iter()
                .map(|col| Alias::new(col.column_name.clone()))
                .collect()
        };

        let mut insert = Query::insert();
        insert
            .into_table(table_ref.clone())
            .columns(insert_columns.into_iter());

        let mut inserted_rows = 0usize;

        for row in rows {
            if row.values.len() != schema.len() {
                warnings.push(format!(
                    "テーブル「{}」行{}: 列数が一致しないためスキップしました (期待{}列/実際{}列)",
                    table_name,
                    row.row_number,
                    schema.len(),
                    row.values.len()
                ));
                continue;
            }

            let mut filtered_values: Vec<_> = Vec::new();

            // guild_idが指定されている場合、先頭にguild_idを追加
            if let Some(gid) = guild_id {
                filtered_values.push(Expr::value(SeaValue::BigInt(Some(gid))));
            }

            // スキーマの各列に対応する値を追加（created_atとupdated_atは既に除外されている）
            for (value, column) in row.values.iter().zip(schema.iter()) {
                if column.column_name != "created_at" && column.column_name != "updated_at" {
                    filtered_values.push(Expr::value(Self::postgres_value_to_sea_value(
                        value, column,
                    )));
                }
            }

            insert
                .values(filtered_values)
                .map_err(|err| FacadeError::Database {
                    source: DbErr::Custom(format!(
                        "テーブル「{table_name}」のINSERT値生成に失敗しました: {err}"
                    )),
                })?;

            inserted_rows += 1;
        }

        if inserted_rows == 0 {
            return Ok(PersistResult {
                inserted_rows: 0,
                warnings,
            });
        }

        // UPSERT対象テーブルの場合はON CONFLICT句を追加
        if self.should_use_upsert(table_name) {
            let (insert_sql, insert_values) = insert.build(PostgresQueryBuilder);

            // ON CONFLICT句を手動で追加
            let upsert_sql = self.build_upsert_query(table_name, &insert_sql, &filtered_schema);

            txn.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                upsert_sql,
                insert_values,
            ))
            .await
            .map_err(FacadeError::from)?;

            // スプレッドシートに存在しないレコードを削除（参照されていないもののみ）
            let inserted_ids: Vec<&PostgresValue> = rows
                .iter()
                .filter_map(|row| row.values.first())
                .collect();

            self.delete_unreferenced_records(txn, table_name, &inserted_ids)
                .await?;
        } else {
            let (insert_sql, insert_values) = insert.build(PostgresQueryBuilder);
            txn.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                insert_sql,
                insert_values,
            ))
            .await
            .map_err(FacadeError::from)?;
        }

        Ok(PersistResult {
            inserted_rows,
            warnings,
        })
    }

    /// テーブルがUPSERT方式で保存すべきかを判定
    ///
    /// 参照マスタテーブルで、他のテーブルから外部キー参照されている場合は
    /// DELETE + INSERTではなくUPSERTを使用する必要があります。
    fn should_use_upsert(&self, table_name: &str) -> bool {
        matches!(
            table_name,
            "battle_styles"  // battle_recruitments, quests, battle_recruitment_schedulesから参照
            | "quests"       // quest_aliases, battle_recruitments, battle_recruitment_schedulesから参照
            | "elements"     // recruitment_participantsから参照
            | "channel_types" // guild_channelsから参照
        )
    }

    /// UPSERT用のSQLクエリを構築
    ///
    /// INSERT文にON CONFLICT句を追加して、既存レコードがあれば更新、なければ挿入します。
    fn build_upsert_query(
        &self,
        table_name: &str,
        insert_sql: &str,
        filtered_schema: &[&ColumnSchema],
    ) -> String {
        // テーブルごとのプライマリキーを定義
        let primary_key = match table_name {
            "channel_types" => "id",
            _ => "id", // デフォルトはid
        };

        // ON CONFLICT (primary_key) DO UPDATE SET column1 = EXCLUDED.column1, column2 = EXCLUDED.column2, ...
        let update_columns: Vec<String> = filtered_schema
            .iter()
            .filter(|col| col.column_name != primary_key) // プライマリキーは更新しない
            .map(|col| format!("{} = EXCLUDED.{}", col.column_name, col.column_name))
            .collect();

        format!(
            "{} ON CONFLICT ({}) DO UPDATE SET {}",
            insert_sql,
            primary_key,
            update_columns.join(", ")
        )
    }

    /// スプレッドシートに存在しないレコードを削除
    ///
    /// 他のテーブルから参照されていないレコードのみ削除します。
    /// 参照されているレコードは削除せず、警告として記録します。
    async fn delete_unreferenced_records(
        &self,
        txn: &DatabaseTransaction,
        table_name: &str,
        inserted_ids: &[&PostgresValue],
    ) -> Result<(), FacadeError> {
        // スプレッドシートに存在するIDのリストを作成（全テーブル共通）
        let id_list: Vec<i32> = inserted_ids
            .iter()
            .filter_map(|v| match v {
                PostgresValue::Integer(id) => Some(*id),
                _ => None,
            })
            .collect();

        match table_name {
            "battle_styles" => {
                self.delete_unreferenced_battle_styles(txn, &id_list)
                    .await?;
            }
            "quests" => {
                self.delete_unreferenced_quests(txn, &id_list).await?;
            }
            "elements" => {
                self.delete_unreferenced_elements(txn, &id_list).await?;
            }
            "channel_types" => {
                self.delete_unreferenced_channel_types(txn, &id_list)
                    .await?;
            }
            _ => {
                // その他のテーブルは何もしない
            }
        }
        Ok(())
    }

    /// battle_stylesテーブルから未参照レコードを削除
    async fn delete_unreferenced_battle_styles(
        &self,
        txn: &DatabaseTransaction,
        id_list: &[i32],
    ) -> Result<(), FacadeError> {
        let schema_name = get_schema_name("battle_styles");
        let delete_sql = if id_list.is_empty() {
            format!(
                "DELETE FROM {schema_name}.battle_styles WHERE id NOT IN (
                    SELECT DISTINCT battle_style_id FROM worker.battle_recruitments
                    UNION SELECT DISTINCT default_battle_style_id FROM master.quests
                    UNION SELECT DISTINCT battle_style_id FROM guild_master.battle_recruitment_schedules
                )"
            )
        } else {
            let placeholders: Vec<String> = (1..=id_list.len()).map(|i| format!("${i}")).collect();
            format!(
                "DELETE FROM {}.battle_styles WHERE id NOT IN ({}) AND id NOT IN (
                    SELECT DISTINCT battle_style_id FROM worker.battle_recruitments
                    UNION SELECT DISTINCT default_battle_style_id FROM master.quests
                    UNION SELECT DISTINCT battle_style_id FROM guild_master.battle_recruitment_schedules
                )",
                schema_name,
                placeholders.join(", ")
            )
        };

        self.execute_delete(txn, &delete_sql, id_list).await?;
        tracing::debug!("battle_stylesテーブルから未参照レコードを削除しました");
        Ok(())
    }

    /// questsテーブルから未参照レコードを削除
    async fn delete_unreferenced_quests(
        &self,
        txn: &DatabaseTransaction,
        id_list: &[i32],
    ) -> Result<(), FacadeError> {
        let schema_name = get_schema_name("quests");
        let delete_sql = if id_list.is_empty() {
            format!(
                "DELETE FROM {schema_name}.quests WHERE id NOT IN (
                    SELECT DISTINCT quest_id FROM master.quest_aliases
                    UNION SELECT DISTINCT quest_id FROM worker.battle_recruitments
                    UNION SELECT DISTINCT quest_id FROM guild_master.battle_recruitment_schedules
                )"
            )
        } else {
            let placeholders: Vec<String> = (1..=id_list.len()).map(|i| format!("${i}")).collect();
            format!(
                "DELETE FROM {}.quests WHERE id NOT IN ({}) AND id NOT IN (
                    SELECT DISTINCT quest_id FROM master.quest_aliases
                    UNION SELECT DISTINCT quest_id FROM worker.battle_recruitments
                    UNION SELECT DISTINCT quest_id FROM guild_master.battle_recruitment_schedules
                )",
                schema_name,
                placeholders.join(", ")
            )
        };

        self.execute_delete(txn, &delete_sql, id_list).await?;
        tracing::debug!("questsテーブルから未参照レコードを削除しました");
        Ok(())
    }

    /// elementsテーブルから未参照レコードを削除
    async fn delete_unreferenced_elements(
        &self,
        txn: &DatabaseTransaction,
        id_list: &[i32],
    ) -> Result<(), FacadeError> {
        let schema_name = get_schema_name("elements");
        let delete_sql = if id_list.is_empty() {
            format!(
                "DELETE FROM {schema_name}.elements WHERE id NOT IN (
                    SELECT DISTINCT element_id FROM worker.recruitment_participants
                )"
            )
        } else {
            let placeholders: Vec<String> = (1..=id_list.len()).map(|i| format!("${i}")).collect();
            format!(
                "DELETE FROM {}.elements WHERE id NOT IN ({}) AND id NOT IN (
                    SELECT DISTINCT element_id FROM worker.recruitment_participants
                )",
                schema_name,
                placeholders.join(", ")
            )
        };

        self.execute_delete(txn, &delete_sql, id_list).await?;
        tracing::debug!("elementsテーブルから未参照レコードを削除しました");
        Ok(())
    }

    /// channel_typesテーブルから未参照レコードを削除
    async fn delete_unreferenced_channel_types(
        &self,
        txn: &DatabaseTransaction,
        id_list: &[i32],
    ) -> Result<(), FacadeError> {
        let schema_name = get_schema_name("channel_types");
        let guild_schema = get_schema_name("guild_channels");
        let delete_sql = if id_list.is_empty() {
            format!(
                "DELETE FROM {schema_name}.channel_types WHERE id NOT IN (SELECT DISTINCT channel_type FROM {guild_schema}.guild_channels)"
            )
        } else {
            let placeholders: Vec<String> = (1..=id_list.len()).map(|i| format!("${i}")).collect();
            format!(
                "DELETE FROM {}.channel_types WHERE id NOT IN ({}) AND id NOT IN (SELECT DISTINCT channel_type FROM {}.guild_channels)",
                schema_name,
                placeholders.join(", "),
                guild_schema
            )
        };

        self.execute_delete(txn, &delete_sql, id_list).await?;
        tracing::debug!("channel_typesテーブルから未参照レコードを削除しました");
        Ok(())
    }

    /// DELETE文を実行
    async fn execute_delete(
        &self,
        txn: &DatabaseTransaction,
        delete_sql: &str,
        id_list: &[i32],
    ) -> Result<(), FacadeError> {
        if id_list.is_empty() {
            txn.execute(Statement::from_string(
                DatabaseBackend::Postgres,
                delete_sql,
            ))
            .await
            .map_err(FacadeError::from)?;
        } else {
            let values: Vec<SeaValue> = id_list.iter().map(|id| SeaValue::Int(Some(*id))).collect();
            txn.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                delete_sql,
                values,
            ))
            .await
            .map_err(FacadeError::from)?;
        }
        Ok(())
    }

    /// PostgresValueからSeaValueへの変換
    fn postgres_value_to_sea_value(value: &PostgresValue, column: &ColumnSchema) -> SeaValue {
        match value {
            PostgresValue::Null => match column.postgres_type {
                PostgresType::Integer => SeaValue::Int(None),
                PostgresType::BigInt => SeaValue::BigInt(None),
                PostgresType::Boolean => SeaValue::Bool(None),
                PostgresType::Timestamp => SeaValue::ChronoDateTime(None),
                PostgresType::TimestampTz => SeaValue::ChronoDateTimeUtc(None),
                PostgresType::Date => SeaValue::ChronoDate(None),
                PostgresType::Uuid => SeaValue::Uuid(None),
                PostgresType::Json | PostgresType::JsonB => SeaValue::Json(None),
                PostgresType::IntegerArray => SeaValue::Array(ArrayType::Int, None),
                PostgresType::TextArray => SeaValue::Array(ArrayType::String, None),
                _ => SeaValue::String(None),
            },
            PostgresValue::Integer(v) => SeaValue::Int(Some(*v)),
            PostgresValue::BigInt(v) => SeaValue::BigInt(Some(*v)),
            PostgresValue::Text(v) => SeaValue::String(Some(Box::new(v.clone()))),
            PostgresValue::Boolean(v) => SeaValue::Bool(Some(*v)),
            PostgresValue::Timestamp(v) => SeaValue::ChronoDateTime(Some(Box::new(*v))),
            PostgresValue::TimestampTz(v) => {
                // ローカルタイムゾーン（JST）をUTCに正しく変換
                let utc = v.with_timezone(&Utc);
                SeaValue::ChronoDateTimeUtc(Some(Box::new(utc)))
            }
            PostgresValue::Date(v) => SeaValue::ChronoDate(Some(Box::new(*v))),
            PostgresValue::Uuid(v) => SeaValue::Uuid(Some(Box::new(*v))),
            PostgresValue::Json(v) => SeaValue::Json(Some(Box::new(v.clone()))),
            PostgresValue::IntegerArray(v) => SeaValue::Array(
                ArrayType::Int,
                Some(Box::new(
                    v.iter().map(|n| SeaValue::Int(Some(*n))).collect(),
                )),
            ),
            PostgresValue::TextArray(v) => SeaValue::Array(
                ArrayType::String,
                Some(Box::new(
                    v.iter()
                        .map(|s| SeaValue::String(Some(Box::new(s.clone()))))
                        .collect(),
                )),
            ),
        }
    }
}

impl Default for SpreadsheetPersistenceService {
    fn default() -> Self {
        Self::new()
    }
}
