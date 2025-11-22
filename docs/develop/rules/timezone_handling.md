# タイムゾーン処理ルール

## 基本方針

### UTC統一原則

**すべての日時データはUTCで保存・処理する**

- データベースに保存する日時は**必ずUTC**
- サービス層・リポジトリ層での処理は**すべてUTC**
- タイムゾーン変換は**表示層（プレゼンテーション層）のみ**で行う

## 理由

1. **データの一貫性**: 異なる地域・サーバーでも同じ時刻を指す
2. **サマータイム対応**: UTCならサマータイムの影響を受けない
3. **計算の簡潔性**: タイムゾーンを気にせず時刻計算ができる
4. **グローバル対応**: 将来的に多地域対応が容易

## 実装ルール

### 1. データベース

```rust
// ✅ 正しい: UTCで保存
use chrono::{DateTime, Utc};

pub struct Model {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub schedule_datetime: DateTime<Utc>,
    pub expiry_date: DateTime<Utc>,
}
```

```rust
// ❌ 間違い: Localは使わない
use chrono::Local;

pub struct Model {
    pub created_at: DateTime<Local>,  // NG
}
```

### 2. ユーザー入力の変換

**ユーザー入力は必ずJST（日本時間）として解釈し、即座にUTCに変換する。**

```rust
use chrono::{DateTime, FixedOffset, Utc};

// datetime_parserサービスを使用
// ユーザー入力: "2025/01/01 19:00" (JST)
let user_input_str = "2025/01/01 19:00";

// datetime_parserでJSTとして解釈し、UTCに変換
let utc_datetime: DateTime<Utc> = datetime_parser::parse_event_date(user_input_str)?;
// 結果: UTC 2025-01-01 10:00

// この後の処理はすべてUTCで行う
```

#### datetime_parserサービスの実装例

```rust
use chrono::{FixedOffset, DateTime, Utc, NaiveDateTime, TimeZone};

const JST_OFFSET_HOURS: i32 = 9;

fn jst() -> FixedOffset {
    FixedOffset::east_opt(JST_OFFSET_HOURS * 3600).unwrap()
}

pub fn parse_event_date(date_str: &str) -> Result<DateTime<Utc>> {
    // 1. 文字列をNaiveDateTimeにパース
    let naive_dt = NaiveDateTime::parse_from_str(date_str, "%Y/%m/%d %H:%M")?;

    // 2. JSTとして解釈
    let jst_dt = jst().from_local_datetime(&naive_dt).unwrap();

    // 3. UTCに変換
    Ok(jst_dt.with_timezone(&Utc))
}
```

### 3. 表示時の変換

ユーザーに表示する際のみ、JSTに変換する。

```rust
use chrono::{DateTime, Duration, Utc};

// DBから取得したUTC日時
let utc_datetime: DateTime<Utc> = notification.schedule_datetime;

// 表示時のみJSTに変換（UTC+9時間）
let jst_datetime = utc_datetime + Duration::hours(9);

// フォーマット
let display_text = jst_datetime.format("%Y/%m/%d %H:%M:%S");
// 例: "2025/01/01 19:00:00"
```

### 4. 現在時刻の取得

```rust
use chrono::Utc;

// ✅ 正しい
let now = Utc::now();

// ❌ 間違い
let now = Local::now();  // NG: サーバーのタイムゾーンに依存
```

## 層別の責務

### Repository層・Service層

- **入力**: `DateTime<Utc>`
- **出力**: `DateTime<Utc>`
- **変換**: なし（すべてUTCのまま処理）

```rust
pub async fn create_notification(
    &self,
    schedule_datetime: DateTime<Utc>,  // UTC
) -> Result<()> {
    // UTCのまま保存
    // ...
}
```

### Facade層

- **入力**: `DateTime<Utc>` または `DateTime<Local>`（後者は即座に変換）
- **出力**: `DateTime<Utc>`
- **変換**: Local→UTCのみ（必要な場合）

```rust
pub async fn create_recruitment(
    &self,
    event_date: Option<DateTime<Local>>,  // ユーザー入力
) -> Result<()> {
    // 即座にUTCに変換
    let utc_event_date = event_date.map(|d| d.with_timezone(&Utc));

    // 以降はUTCで処理
    // ...
}
```

### Presentation層（Events/Commands）

- **入力**: ユーザーからの文字列入力
- **出力**: Discord表示用文字列
- **変換**:
  - 入力時: 文字列 → `DateTime<Local>` → `DateTime<Utc>`
  - 出力時: `DateTime<Utc>` → JST表示

```rust
// コマンド引数をパース
let event_date_local: DateTime<Local> = parse_user_input(input_str)?;

// 即座にUTCに変換してFacadeに渡す
let event_date_utc = event_date_local.with_timezone(&Utc);

// 表示時はJSTに変換
fn format_for_display(utc_datetime: DateTime<Utc>) -> String {
    let jst = utc_datetime + Duration::hours(9);
    jst.format("%Y/%m/%d %H:%M:%S (JST)").to_string()
}
```

## 実例

### 募集作成のフロー

```rust
// 1. ユーザー入力（プレゼンテーション層）
// ユーザー: "2025-01-01 19:00" (JST)
let input_str = "2025-01-01 19:00";
let local_datetime = parse_datetime_jst(input_str)?; // DateTime<Local>

// 2. UTCに変換（プレゼンテーション層）
let utc_datetime = local_datetime.with_timezone(&Utc);
// 結果: 2025-01-01 10:00 UTC

// 3. Facade層に渡す
facade.create_recruitment(utc_datetime).await?;

// 4. Service層・Repository層
// すべてUTCのまま処理
service.save_recruitment(utc_datetime).await?;

// 5. DBに保存
// created_at: 2025-01-01 10:xx UTC (現在時刻)
// expiry_date: 2025-01-01 10:00 UTC
// 一貫性が保たれる！

// 6. 通知を登録（5分前）
let notify_time = utc_datetime - Duration::minutes(5);
// 2025-01-01 09:55 UTC

// 7. 表示時（プレゼンテーション層）
let jst_display = utc_datetime + Duration::hours(9);
println!("出発時刻: {} (JST)", jst_display.format("%Y/%m/%d %H:%M"));
// 表示: "出発時刻: 2025/01/01 19:00 (JST)"
```

## よくある間違い

### ❌ 間違い1: LocalをDBに保存

```rust
// NG
let now = Local::now();
model.created_at = now;  // サーバーのタイムゾーンに依存
```

### ❌ 間違い2: JSTオフセットをハードコード

```rust
// NG
let jst = utc + Duration::hours(9);  // 将来的にサマータイムがあるとバグる
```

正しくは、表示層でのみ変換を行い、明示的にコメントする：

```rust
// ✅ OK
// 表示用にJST変換（UTC+9）
let jst_display = utc_datetime + Duration::hours(9);
```

### ❌ 間違い3: タイムゾーン変換を複数箇所で行う

```rust
// NG: Service層で表示用の変換
service.format_datetime_jst(utc);  // Service層の責務外

// ✅ OK: Presentation層でのみ変換
fn format_for_display(utc: DateTime<Utc>) -> String {
    let jst = utc + Duration::hours(9);
    jst.format("%Y/%m/%d %H:%M (JST)").to_string()
}
```

## チェックリスト

実装時は以下を確認：

- [ ] DBに保存する日時は`DateTime<Utc>`型か
- [ ] `Local::now()`を使っていないか
- [ ] ユーザー入力を受け取った直後にUTCに変換しているか
- [ ] Service層・Repository層でタイムゾーン変換していないか
- [ ] 表示時のみJST変換を行っているか
- [ ] JSTへの変換時にコメントで明示しているか

## 参考

- Chrono documentation: https://docs.rs/chrono/
- UTC vs Local: https://en.wikipedia.org/wiki/Coordinated_Universal_Time
