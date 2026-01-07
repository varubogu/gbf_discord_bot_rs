# スプレッドシートデータ変換仕様書

> **✅ 実装状況: UUID自動採番と書き戻しは実装済み**
>
> このドキュメントに記載されているUUID自動採番機能とスプレッドシート書き戻し機能は実装されています。
>
> **実装の確認:**
> - UUID自動生成: `src/services/spreadsheet/spreadsheet_reader_service.rs`
>   - 空のUUID列を検出して新規UUIDを生成
>   - 生成情報（シート名、行番号、列番号、UUID）を追跡
> - 書き戻し機能: `src/facades/spreadsheet/core/import_facade.rs`
>   - トランザクションコミット前にスプレッドシートにUUIDを書き込み
>   - 書き戻し失敗時はトランザクションをロールバック

## 概要

Googleスプレッドシート（文字列）とPostgreSQL（型付きデータ）間のデータ変換ルールを定義します。双方向の変換仕様とバリデーションルールを規定します。

## 変換の基本方針

- **型安全性**: PostgreSQL型に厳密に従った変換
- **エラー時の挙動**: 変換失敗時は該当行をスキップし、ログに記録
- **NULL値の扱い**: 空文字列をNULLとして扱う
- **デフォルト値**: テーブル定義のデフォルト値を尊重

## PostgreSQL型 → スプレッドシート変換

### 基本型変換

| PostgreSQL型 | Rust型 | スプレッドシート表現 | 変換例 |
|------------|--------|------------------|-------|
| INTEGER | i32 | 数値文字列 | `123` → `"123"` |
| BIGINT | i64 | 数値文字列 | `9876543210` → `"9876543210"` |
| VARCHAR / TEXT | String | そのまま | `"クエスト名"` → `"クエスト名"` |
| BOOLEAN | bool | "true" / "false" | `true` → `"true"` |
| TIMESTAMP | DateTime<Utc> | RFC3339文字列 | `2025-01-15 12:00:00` → `"2025-01-15T12:00:00+09:00"` |
| UUID | Uuid | UUID文字列 | `uuid` → `"550e8400-e29b-41d4-a716-446655440000"` |

### NULL値の変換

| PostgreSQL値 | スプレッドシート表現 |
|------------|------------------|
| NULL | 空文字列 `""` |
| NOT NULL | 値の文字列表現 |

### 変換ロジック（概念）

```rust
pub fn postgres_to_spreadsheet(value: &sea_orm::Value) -> String {
    match value {
        sea_orm::Value::Int(Some(v)) => v.to_string(),
        sea_orm::Value::BigInt(Some(v)) => v.to_string(),
        sea_orm::Value::String(Some(v)) => v.clone(),
        sea_orm::Value::Bool(Some(v)) => v.to_string(),
        sea_orm::Value::ChronoDateTimeUtc(Some(dt)) => dt.to_rfc3339(),
        sea_orm::Value::Uuid(Some(uuid)) => uuid.to_string(),
        _ => String::new(), // NULL値は空文字列
    }
}
```

---

## スプレッドシート → PostgreSQL型変換

### Integer / BigInteger変換

**入力**: 数値文字列
**出力**: `i32` / `i64`

**変換ルール**:
```rust
pub fn parse_integer(value: &str, field: &str) -> Result<i32, ValidationError> {
    if value.is_empty() {
        return Ok(0); // 空文字列はデフォルト値0（NULLABLEの場合はNone）
    }

    value.parse::<i32>().map_err(|_| ValidationError::TypeConversionError {
        field: field.to_string(),
        value: value.to_string(),
        expected_type: "Integer".to_string(),
    })
}
```

**バリデーション**:
- 空文字列 → デフォルト値または`NULL`
- 数値以外 → `ValidationError::TypeConversionError`
- 範囲外 → `ValidationError::ValueOutOfRange`

**サンプル**:
| 入力 | 出力 | 備考 |
|------|------|------|
| `"123"` | `123` | 正常 |
| `""` | `NULL` または `0` | NULLABLEかどうかで異なる |
| `"abc"` | エラー | TypeConversionError |
| `"9999999999"` | エラー | ValueOutOfRange (i32超過) |

---

### String / Text変換

**入力**: 文字列
**出力**: `String`

**変換ルール**:
```rust
pub fn parse_string(value: &str, field: &str, nullable: bool) -> Result<Option<String>, ValidationError> {
    if value.is_empty() {
        if nullable {
            return Ok(None);
        } else {
            return Err(ValidationError::RequiredFieldMissing {
                field: field.to_string(),
            });
        }
    }

    Ok(Some(value.to_string()))
}
```

**バリデーション**:
- 空文字列 + NOT NULL → `ValidationError::RequiredFieldMissing`
- 空文字列 + NULLABLE → `NULL`
- 文字列 → そのまま

**サンプル**:
| 入力 | NULLABLE | 出力 | 備考 |
|------|----------|------|------|
| `"クエスト名"` | - | `Some("クエスト名")` | 正常 |
| `""` | true | `None` | NULL |
| `""` | false | エラー | RequiredFieldMissing |

---

### Boolean変換

**入力**: "true" / "false" / "1" / "0"
**出力**: `bool`

**変換ルール**:
```rust
pub fn parse_boolean(value: &str, field: &str) -> Result<bool, ValidationError> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "t" => Ok(true),
        "false" | "0" | "no" | "f" | "" => Ok(false),
        _ => Err(ValidationError::TypeConversionError {
            field: field.to_string(),
            value: value.to_string(),
            expected_type: "Boolean (true/false/1/0)".to_string(),
        }),
    }
}
```

**サンプル**:
| 入力 | 出力 | 備考 |
|------|------|------|
| `"true"` | `true` | 正常 |
| `"false"` | `false` | 正常 |
| `"1"` | `true` | 許容 |
| `"0"` | `false` | 許容 |
| `""` | `false` | 空文字列は`false` |
| `"abc"` | エラー | TypeConversionError |

---

### DateTime変換

**入力**: 日時文字列（複数フォーマット対応）
**出力**: `DateTime<Utc>`

**対応フォーマット**:
1. RFC3339: `"2025-01-15T12:00:00+09:00"`
2. ISO8601: `"2025-01-15T12:00:00Z"`
3. スペース区切り: `"2025-01-15 12:00:00"`
4. 日付のみ: `"2025-01-15"` → `00:00:00` として解釈

**変換ルール**:
```rust
use chrono::{DateTime, NaiveDateTime, Utc};

pub fn parse_datetime(value: &str, field: &str) -> Result<DateTime<Utc>, ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::RequiredFieldMissing {
            field: field.to_string(),
        });
    }

    // RFC3339形式を試行
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Ok(dt.with_timezone(&Utc));
    }

    // スペース区切り形式を試行
    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc));
    }

    // 日付のみ形式を試行
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        let naive_dt = naive_date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc));
    }

    Err(ValidationError::DateTimeFormatError {
        value: value.to_string(),
        supported_formats: "RFC3339, YYYY-MM-DD HH:MM:SS, YYYY-MM-DD".to_string(),
    })
}
```

**サンプル**:
| 入力 | 出力 | フォーマット |
|------|------|------------|
| `"2025-01-15T12:00:00+09:00"` | `2025-01-15 03:00:00 UTC` | RFC3339 |
| `"2025-01-15 12:00:00"` | `2025-01-15 12:00:00 UTC` | スペース区切り |
| `"2025-01-15"` | `2025-01-15 00:00:00 UTC` | 日付のみ |
| `""` | エラー | RequiredFieldMissing |
| `"invalid"` | エラー | DateTimeFormatError |

---

### UUID変換と自動採番

**入力**: UUID文字列
**出力**: `Uuid`

UUID型カラムでは、スプレッドシート上の値が空の場合に自動的に新規UUIDを生成し、データベースに登録後、スプレッドシートにも書き戻します。

#### UUID自動採番の目的

- **ユーザビリティ向上**: UUIDを手動で入力する必要をなくす
- **データ整合性保証**: スプレッドシートとデータベースのUUIDを常に一致させる
- **重複防止**: 毎回異なるUUIDを自動生成することで、主キー重複を防ぐ

#### 変換ルール

```rust
use uuid::Uuid;

pub fn parse_uuid(value: &str, field: &str) -> Result<Uuid, ValidationError> {
    if value.is_empty() || value.eq_ignore_ascii_case("null") {
        // 空文字列またはNULLの場合は新規UUID生成
        return Ok(Uuid::new_v4());
    }

    // 既存のUUID文字列はそのまま解析
    Uuid::parse_str(value).map_err(|_| ValidationError::UuidFormatError {
        value: value.to_string(),
    })
}
```

#### スプレッドシートへの書き戻し処理

UUID自動生成後、以下の手順でスプレッドシートに書き戻されます：

1. **読み込み時にUUID生成**: スプレッドシート読み込み時、空のUUID列を検出して新規UUIDを生成
2. **生成情報の追跡**: 生成されたUUID、シート名、行番号、列番号を記録
3. **データベースへの登録**: 生成されたUUIDを含むデータをトランザクション内でデータベースに登録
4. **スプレッドシート書き戻し**: データベースコミット前に、Google Sheets APIを使用して該当セルにUUIDを書き込み
5. **トランザクションコミット**: スプレッドシート書き戻しが成功した場合のみデータベースをコミット

#### エラーハンドリング

UUID書き戻しに失敗した場合の挙動：

- **ロールバック**: データベーストランザクションをロールバック
- **エラー通知**: ユーザーに「UUID書き戻し失敗によりDB登録もキャンセルした」旨を通知
- **理由**: 次回読み込み時に同じ空行が再度処理され、異なるUUIDが生成されることでPK重複エラーを防ぐため

```rust
// UUID書き戻し失敗時の処理（概念）
if let Err(e) = write_back_uuids_to_spreadsheet(&generated_uuids).await {
    error!("UUID書き戻しに失敗したため、トランザクションをロールバックします");
    txn.rollback().await?;
    return Err(FacadeError::ExternalService {
        source: ExternalServiceError::GoogleSheetsApiError {
            message: format!(
                "UUID書き戻しに失敗しました。次回読み込み時のID不整合を防ぐため、DB登録もロールバックしました: {}",
                e
            ),
        },
    });
}
```

#### サンプル

**スプレッドシート上の入力**:
| quest_id | quest_name | recruit_count |
|----------|------------|--------------|
| (空) | プロトバハムートHL | 30 |
| 550e8400-e29b-41d4-a716-446655440000 | アルティメットバハムートHL | 18 |

**処理後**:
| quest_id | quest_name | recruit_count |
|----------|------------|--------------|
| 7b3f2a9c-8e45-4d1f-b3c9-1a5e8f6d2c4b | プロトバハムートHL | 30 |
| 550e8400-e29b-41d4-a716-446655440000 | アルティメットバハムートHL | 18 |

**変換サマリ**:
| 入力 | 出力 | 備考 |
|------|------|------|
| `"550e8400-e29b-41d4-a716-446655440000"` | 対応するUUID | 既存UUIDをそのまま使用 |
| `""` | `Uuid::new_v4()` | 新規UUID自動生成 + スプレッドシート書き戻し |
| `"null"` | `Uuid::new_v4()` | 新規UUID自動生成 + スプレッドシート書き戻し |
| `"invalid-uuid"` | エラー | UuidFormatError |

#### 実装上の注意点

- **トランザクション順序**: スプレッドシート書き戻し → データベースコミットの順序を厳守
- **書き戻し失敗時の完全ロールバック**: 部分的な成功を許容しない
- **A1記法への変換**: 列番号（0始まり）をA1記法の列文字列（"A", "B", "AA"など）に変換
- **ログ出力**: 生成されたUUID数、書き戻し成功/失敗をログに記録

---

## 外部キー制約のバリデーション

外部キーカラムの値は、参照先テーブルに存在するかを検証します。

**検証ロジック**:
```rust
pub async fn validate_foreign_key(
    txn: &DatabaseTransaction,
    field: &str,
    value: i64,
    reference_table: &str,
    reference_column: &str,
) -> Result<(), ValidationError> {
    // 参照先テーブルに該当値が存在するか確認
    let exists = check_exists(txn, reference_table, reference_column, value).await?;

    if !exists {
        return Err(ValidationError::ForeignKeyViolation {
            field: field.to_string(),
            reference_table: reference_table.to_string(),
            value: value.to_string(),
        });
    }

    Ok(())
}
```

**例**: `quests.default_battle_type` → `battle_types.type_id`

| quests.default_battle_type | battle_types.type_id | 結果 |
|---------------------------|---------------------|------|
| `1` | `1, 2, 3` が存在 | OK |
| `999` | 存在しない | ForeignKeyViolation エラー |

---

## カスタム変換ルール

### カンマ区切り文字列 → 配列

一部のカラム（例: `quests.use_battle_type`）はカンマ区切りの文字列として格納されます。

**変換ルール**:
```rust
pub fn parse_comma_separated_integers(value: &str) -> Vec<i32> {
    value
        .split(',')
        .filter_map(|s| s.trim().parse::<i32>().ok())
        .collect()
}
```

**サンプル**:
| 入力 | 出力 | 備考 |
|------|------|------|
| `"1,2,3"` | `[1, 2, 3]` | 正常 |
| `"1, 2, 3"` | `[1, 2, 3]` | スペース許容 |
| `"1,abc,3"` | `[1, 3]` | 不正な値はスキップ |
| `""` | `[]` | 空配列 |

---

## guild_id自動付与

ギルド固有テーブル（`guild_*`）では、`guild_id`カラムがスプレッドシートに含まれていない場合、現在のギルドIDを自動付与します。

**自動付与ロジック**:
```rust
pub fn apply_guild_id(
    row_data: &mut HashMap<String, String>,
    guild_id: i64,
    has_guild_id_column: bool,
) -> Result<(), ValidationError> {
    if !has_guild_id_column {
        // guild_idカラムがない場合、自動付与
        row_data.insert("guild_id".to_string(), guild_id.to_string());
        return Ok(());
    }

    // guild_idカラムがある場合、値を検証
    if let Some(value) = row_data.get("guild_id") {
        let parsed_guild_id = value.parse::<i64>().map_err(|_| {
            ValidationError::TypeConversionError {
                field: "guild_id".to_string(),
                value: value.to_string(),
                expected_type: "BigInteger".to_string(),
            }
        })?;

        if parsed_guild_id != guild_id {
            return Err(ValidationError::GuildIdMismatch {
                expected: guild_id.to_string(),
                actual: parsed_guild_id.to_string(),
            });
        }
    }

    Ok(())
}
```

---

## エラー時の挙動

### データ変換エラーの処理方針

**原則**: エラー行をスキップし、ログに記録、処理は継続

```rust
pub fn convert_rows(
    rows: Vec<HashMap<String, String>>,
    table_def: &TableDefinition,
) -> (Vec<ConvertedRow>, Vec<ConversionError>) {
    let mut converted = Vec::new();
    let mut errors = Vec::new();

    for (index, row) in rows.iter().enumerate().skip(2) { // 1,2行目はヘッダー
        match convert_row(row, table_def) {
            Ok(converted_row) => converted.push(converted_row),
            Err(e) => {
                warn!(
                    table = %table_def.table_name_en,
                    row_index = index + 1, // 1-indexed
                    error = %e,
                    "行の変換に失敗しました（スキップします）"
                );
                errors.push(ConversionError {
                    table_name: table_def.table_name_en.clone(),
                    row_index: index + 1,
                    error: e,
                });
            }
        }
    }

    (converted, errors)
}
```

### ユーザーへのフィードバック

変換エラーが発生した場合、Discordユーザーにエラー詳細を通知します。

**エラーメッセージ例**:
```
❌ エラー: データ変換中にエラーが発生しました

以下の行で変換エラーが発生しました:
- questsテーブル 5行目: recruit_count は数値である必要があります (値: "abc")
- questsテーブル 8行目: start_at の日時形式が不正です (値: "2025-13-40")

合計 2 行をスキップし、残りのデータを登録しました。
```

---

## パフォーマンス考慮事項

### バルク変換

行ごとの変換を並行処理することでパフォーマンスを向上します。

```rust
use rayon::prelude::*;

pub fn convert_rows_parallel(
    rows: Vec<HashMap<String, String>>,
    table_def: &TableDefinition,
) -> (Vec<ConvertedRow>, Vec<ConversionError>) {
    let results: Vec<_> = rows
        .par_iter()
        .enumerate()
        .skip(2)
        .map(|(index, row)| {
            match convert_row(row, table_def) {
                Ok(converted) => Ok((index, converted)),
                Err(e) => Err((index, e)),
            }
        })
        .collect();

    let mut converted = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok((_, row)) => converted.push(row),
            Err((index, e)) => errors.push(ConversionError {
                table_name: table_def.table_name_en.clone(),
                row_index: index + 1,
                error: e,
            }),
        }
    }

    (converted, errors)
}
```

---

## テスト戦略

### 単体テスト例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer_valid() {
        assert_eq!(parse_integer("123", "test_field").unwrap(), 123);
    }

    #[test]
    fn test_parse_integer_invalid() {
        let result = parse_integer("abc", "test_field");
        assert!(matches!(result, Err(ValidationError::TypeConversionError { .. })));
    }

    #[test]
    fn test_parse_datetime_rfc3339() {
        let result = parse_datetime("2025-01-15T12:00:00+09:00", "test_field");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_datetime_space_separated() {
        let result = parse_datetime("2025-01-15 12:00:00", "test_field");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_comma_separated() {
        let result = parse_comma_separated_integers("1,2,3");
        assert_eq!(result, vec![1, 2, 3]);
    }
}
```

---

## 関連ドキュメント

- [エラー型定義](../error_types.md)
- [Service層設計](service_layer.md)
- [データベーステーブル設計](../../database/README.md)
- [機能概要](../../features/google_spreadsheet.md)
