# 統一日時パーサー設計書

## 概要

`unified_datetime_parser`は、ビットフラグベースの柔軟な日時解析システムです。複数のパーサー（`datetime_parser`, `TimeParserService`, `DismissalTimeParserService`）を統合し、一貫したインターフェースを提供します。

## 設計思想

### 問題点

従来、以下の3つのパーサーが独立して存在していました:

1. **datetime_parser** - クエスト出発日時用（絶対日時のみ）
2. **TimeParserService** - HH:MM形式専用
3. **DismissalTimeParserService** - 解散時刻用（相対・絶対両方）

これらは:
- インターフェースが統一されていない
- 機能の重複がある
- 拡張が困難
- テストが分散している

### 解決策

ビットフラグによる機能のON/OFF切り替えで、単一のパーサーで全ユースケースに対応:

```rust
pub struct DateTimeParseFlags {
    FULL_DATETIME      // 完全日時: "2025/11/15 21:00"
    DATETIME_NO_YEAR   // 年なし日時: "12/11 14:00"
    DATE_ONLY          // 日付のみ: "11/15"
    TIME_ONLY          // 時刻のみ: "21:00", "21時"
    JAPANESE_DATETIME  // 日本語: "1月2日3時4分"
    NUMERIC_PATTERNS   // 数字: "1230", "10111230"
    RELATIVE_TIME      // 相対: "2時間前", "1day"
}
```

## 使用方法

### 基本的な使い方

```rust
use crate::services::unified_datetime_parser::{
    parse_datetime, DateTimeParseOptions, ParsedDateTime,
};

// オプションを作成
let options = DateTimeParseOptions::for_quest_departure(timezone);

// パース
let results = parse_datetime("2025/11/15 21:00", &options)?;

// 結果を取得
match &results[0] {
    ParsedDateTime::Absolute(dt) => println!("絶対日時: {}", dt),
    ParsedDateTime::Relative { days, hours, minutes } => {
        println!("相対時刻: {}日{}時間{}分前", days, hours, minutes)
    }
    ParsedDateTime::Time(t) => println!("時刻: {}", t),
}
```

### ユースケース別のプリセット

#### 1. クエスト出発日時

```rust
let options = DateTimeParseOptions::for_quest_departure(timezone);
```

**許可パターン**:
- ✅ 完全日時: `"2025/11/15 21:00"`
- ✅ 年なし: `"12/11 14:00"`
- ✅ 日付のみ: `"11/15"` (デフォルト21時)
- ✅ 時刻のみ: `"21:00"`, `"21時"`
- ✅ 日本語: `"1月2日3時4分"`, `"午後9時半"`
- ✅ 数字: `"1230"`, `"10111230"`, `"30 1230"`
- ❌ 相対時刻

#### 2. 解散時刻

```rust
let options = DateTimeParseOptions::for_dismissal_time(
    timezone,
    quest_departure_time
);
```

**許可パターン**:
- ✅ 全ての絶対日時パターン
- ✅ 相対時刻: `"1時間前"`, `"2日前"`, `"90分前"`
- ✅ カンマ区切りで最大3つ: `"1時間前, 21:00, 2日前"`

#### 3. 定期募集開始時刻

```rust
let options = DateTimeParseOptions::for_schedule_start_time(
    timezone,
    quest_start_time
);
```

**許可パターン**:
- ✅ 時刻: `"21:00"`, `"21時"`, `"午後9時半"`
- ✅ 数字: `"1230"`
- ✅ 相対時刻: `"2時間前"`, `"1h"` (クエスト開始時刻を基準)
- ❌ 日付指定

#### 4. HH:MM厳格モード

```rust
let options = DateTimeParseOptions::strict_hhmm_only(timezone);
```

**許可パターン**:
- ✅ `"22:00"`, `"09:30"` のみ
- ❌ その他すべて

### カスタムオプション

```rust
let options = DateTimeParseOptions {
    flags: DateTimeParseFlags::TIME_ONLY
         | DateTimeParseFlags::RELATIVE_TIME,
    timezone,
    relative_base: Some(RelativeBase::Time(base_time)),
    default_time: None,
    allow_multiple: false,
    max_count: 1,
};
```

## 実装詳細

### 解析フロー

```
入力文字列
    ↓
allow_multiple? → カンマで分割
    ↓
各パートをparse_single
    ↓
1. strict mode? → HH:MM厳格チェック
2. RELATIVE_TIME有効? → 相対時刻試行
3. その他 → 絶対日時試行 (既存datetime_parserを使用)
    ↓
ParsedDateTime返却
```

### 解析結果の型

```rust
pub enum ParsedDateTime {
    /// 絶対日時 (UTC)
    Absolute(DateTime<Utc>),

    /// 相対時刻（基準時刻からのオフセット）
    Relative {
        days: i32,
        hours: i32,
        minutes: i32,
    },

    /// NaiveTime (定期募集開始時刻など)
    Time(NaiveTime),
}
```

### 相対時刻の基準

```rust
pub enum RelativeBase {
    /// DateTime基準 (解散時刻など)
    DateTime(DateTime<Utc>),

    /// NaiveTime基準 (定期募集など)
    Time(NaiveTime),
}
```

## 対応パターン一覧

### 絶対日時パターン

| パターン | 例 | フラグ |
|---------|---|-------|
| 完全日時 (/) | `2025/11/15 21:00` | FULL_DATETIME |
| 完全日時 (-) | `2025-11-15 21:00` | FULL_DATETIME |
| 年なし (/) | `12/11 14:00` | DATETIME_NO_YEAR |
| 年なし (-) | `12-11 14:00` | DATETIME_NO_YEAR |
| 日付のみ (/) | `11/15` | DATE_ONLY |
| 日付のみ (-) | `11-15` | DATE_ONLY |
| 時刻 (:) | `21:00`, `9:30` | TIME_ONLY |
| 日本語完全 | `1月2日3時4分` | JAPANESE_DATETIME |
| 日本語時刻 | `21時`, `午後9時半` | JAPANESE_DATETIME |
| 日本語日付 | `1月2日` | JAPANESE_DATETIME |
| 4桁時刻 | `1230` | NUMERIC_PATTERNS |
| 8桁日時 | `10111230` | NUMERIC_PATTERNS |
| 日+4桁時刻 | `30 1230` | NUMERIC_PATTERNS |

### 相対時刻パターン

| 言語 | 単位 | 例 |
|-----|------|---|
| 日本語 | 日 | `1日前`, `2日` |
| 日本語 | 時間 | `1時間前`, `2時間` |
| 日本語 | 分 | `90分前`, `30分` |
| English | day | `1day`, `2days` |
| English | hour | `1hour`, `2hours`, `1h` |
| English | minute | `90minutes`, `90mins`, `90m` |

## テスト

```bash
# 統一パーサーのテストを実行
cargo test --lib unified_datetime_parser

# 特定のテスト
cargo test --lib test_quest_departure_absolute_datetime
cargo test --lib test_dismissal_time_multiple
cargo test --lib test_relative_time_english
```

## datetime_parser のメタデータ対応

`datetime_parser` モジュールは v0.5.0 からメタデータ（入力に含まれていた日時要素の情報）を返すようになりました。

### DateTimeComponents 構造体

```rust
pub struct DateTimeComponents {
    pub has_year: bool,   // 年が入力に含まれていたか
    pub has_month: bool,  // 月が入力に含まれていたか
    pub has_day: bool,    // 日が入力に含まれていたか
    pub has_time: bool,   // 時刻が入力に含まれていたか
}
```

### 自動補正機能

`parse_event_date()` は過去日時を自動的に未来に補正します:
- **年が未指定**で過去になる場合 → 翌年に補正
- **月が未指定**で過去になる場合 → 翌月に補正
- 時刻のみの場合は既にパーサー内で翌日補正済み

#### 補正の例

```rust
use crate::services::datetime_parser::parse_event_date;

// 今日が12/29の場合
let result = parse_event_date("28 1000", timezone)?;
// → 翌月の1/28 10:00 に補正される（月が未指定で過去のため）

let result = parse_event_date("1/4", timezone)?;
// → 翌年の1/4 21:00 に補正される（年が未指定で過去のため）

let result = parse_event_date("2025/1/4", timezone)?;
// → 2025/1/4 21:00（年が指定されているため補正なし）
```

### 手動で補正を制御する場合

自動補正が不要な場合は `parse_event_date_with_components()` を使用:

```rust
use crate::services::datetime_parser::parse_event_date_with_components;
use chrono::Utc;

let result = parse_event_date_with_components("12/28", timezone)?;
// result.datetime は補正前の値
// result.components でどの要素が入力されたかを確認可能

// 独自の補正ロジックを実装
let corrected = if !result.components.has_year && result.datetime < Utc::now() {
    // カスタム補正処理
    result.datetime + chrono::Duration::days(365)
} else {
    result.datetime
};
```

### パターン別のメタデータ

| 入力パターン | has_year | has_month | has_day | has_time |
|-------------|----------|-----------|---------|----------|
| `2025/11/15 21:00` | ✅ | ✅ | ✅ | ✅ |
| `12/11 14:00` | ❌ | ✅ | ✅ | ✅ |
| `11/15` | ❌ | ✅ | ✅ | ❌ |
| `2025/11/15` | ✅ | ✅ | ✅ | ❌ |
| `21:00` | ❌ | ❌ | ❌ | ✅ |
| `30 2100` | ❌ | ❌ | ✅ | ✅ |
| `1月2日3時4分` | ❌ | ✅ | ✅ | ✅ |
| `1月2日` | ❌ | ✅ | ✅ | ❌ |
| `午後9時半` | ❌ | ❌ | ❌ | ✅ |

## 移行ガイド

### 既存コードからの移行

#### datetime_parser::parse_event_date から

**Before**:
```rust
let dt = datetime_parser::parse_event_date(input, timezone)?;
```

**After (メタデータ不要な場合)**:
```rust
let dt = datetime_parser::parse_event_date(input, timezone)?;
// 後方互換性のため、既存のコードはそのまま動作します
```

**After (メタデータが必要な場合)**:
```rust
let result = datetime_parser::parse_event_date_with_components(input, timezone)?;
let dt = result.datetime;
let components = result.components;

// componentsフラグを使って補正処理を実装
if !components.has_year && dt < Utc::now() {
    dt = dt + chrono::Duration::days(365);
}
```

**After (unified_datetime_parser を使う場合)**:
```rust
let options = DateTimeParseOptions::for_quest_departure(timezone);
let results = parse_datetime(input, &options)?;
let dt = match &results[0] {
    ParsedDateTime::Absolute(dt) => dt,
    _ => return Err("絶対日時が必要です".into()),
};
```

#### TimeParserService から

**Before**:
```rust
let service = TimeParserService::new();
let time = service.parse_time_string(input)?;
```

**After**:
```rust
let options = DateTimeParseOptions::strict_hhmm_only(timezone);
let results = parse_datetime(input, &options)?;
let time = match &results[0] {
    ParsedDateTime::Time(t) => t,
    _ => return Err("時刻が必要です".into()),
};
```

#### DismissalTimeParserService から

**Before**:
```rust
let parser = DismissalTimeParserService::new();
let parsed = parser.parse(input, departure_time, timezone, max_days)?;
```

**After**:
```rust
let options = DateTimeParseOptions::for_dismissal_time(timezone, departure_time);
let results = parse_datetime(input, &options)?;
// resultsは Vec<ParsedDateTime> として使用
```

## 拡張可能性

- より詳細なエラーメッセージ（どのパターンを試したか）
- パターン無効化時の詳細エラー
- パフォーマンス最適化（無効なパターンのスキップ）

## 関連ファイル

- `src/services/unified_datetime_parser.rs` - メイン実装
- `src/services/datetime_parser.rs` - 既存パーサー（絶対日時）
- `src/services/recruitment/dismissal_time_parser_service.rs` - 既存パーサー（解散時刻）
- `src/services/recruitment/schedule/time_parser_service.rs` - 既存パーサー（HH:MM）
