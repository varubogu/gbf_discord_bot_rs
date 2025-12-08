# タイムゾーン処理ルール

## 基本方針

### UTC統一原則

**すべての日時データはUTCで保存・処理する**

- データベースに保存する日時は**必ずUTC**
- サービス層・リポジトリ層での処理は**すべてUTC**
- タイムゾーン変換は**表示層（プレゼンテーション層）のみ**で行う

### サーバーごとのタイムゾーン設定

**各Discordサーバー（Guild）ごとに異なるタイムゾーンを設定可能**

- デフォルトタイムゾーンは`Asia/Tokyo`（JST）
- ユーザー入力はサーバー設定のタイムゾーンとして解釈し、UTCに変換
- 表示時はサーバー設定のタイムゾーンに変換
- スプレッドシート読み込みもサーバー設定のタイムゾーンに従う

## 理由

1. **データの一貫性**: 異なる地域・サーバーでも同じ時刻を指す
2. **サマータイム対応**: UTCならサマータイムの影響を受けない
3. **計算の簡潔性**: タイムゾーンを気にせず時刻計算ができる
4. **グローバル対応**: 多地域対応が可能
5. **柔軟性**: サーバーごとの利用者環境に合わせた運用が可能

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

**ユーザー入力は必ずサーバー設定のタイムゾーンとして解釈し、即座にUTCに変換する。**

```rust
use chrono::{DateTime, Utc};
use chrono_tz::Tz;

// サーバー設定のタイムゾーンを取得
let guild_tz: Tz = timezone_service.get_guild_timezone(guild_id).await?;
// 例: Asia/Tokyo, America/New_York, Europe/London など

// ユーザー入力: "2025/01/01 19:00" (サーバー設定のタイムゾーン)
let user_input_str = "2025/01/01 19:00";

// datetime_parserでサーバー設定のタイムゾーンとして解釈し、UTCに変換
let utc_datetime: DateTime<Utc> = datetime_parser::parse_event_date(user_input_str, guild_tz)?;
// Asia/Tokyoの場合: UTC 2025-01-01 10:00
// America/New_York (EST)の場合: UTC 2025-01-02 00:00

// この後の処理はすべてUTCで行う
```

#### datetime_parserサービスの実装例

```rust
use chrono::{DateTime, Utc, NaiveDateTime, TimeZone};
use chrono_tz::Tz;

pub fn parse_event_date(date_str: &str, timezone: Tz) -> Result<DateTime<Utc>> {
    // 1. 文字列をNaiveDateTimeにパース
    let naive_dt = NaiveDateTime::parse_from_str(date_str, "%Y/%m/%d %H:%M")?;

    // 2. サーバー設定のタイムゾーンとして解釈
    let tz_dt = timezone.from_local_datetime(&naive_dt)
        .single()
        .ok_or("曖昧な時刻またはサマータイム切り替え時刻です")?;

    // 3. UTCに変換
    Ok(tz_dt.with_timezone(&Utc))
}
```

### 3. 表示時の変換

ユーザーに表示する際のみ、サーバー設定のタイムゾーンに変換する。

```rust
use chrono::{DateTime, Utc};
use chrono_tz::Tz;

// サーバー設定のタイムゾーンを取得
let guild_tz: Tz = timezone_service.get_guild_timezone(guild_id).await?;

// DBから取得したUTC日時
let utc_datetime: DateTime<Utc> = notification.schedule_datetime;

// 表示時のみサーバー設定のタイムゾーンに変換
let local_datetime = utc_datetime.with_timezone(&guild_tz);

// フォーマット（タイムゾーン略称を含める）
let display_text = local_datetime.format("%Y/%m/%d %H:%M:%S %Z");
// Asia/Tokyoの場合: "2025/01/01 19:00:00 JST"
// America/New_Yorkの場合: "2025/01/01 14:00:00 EST"
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
  - 入力時: 文字列 → サーバー設定のタイムゾーン → `DateTime<Utc>`
  - 出力時: `DateTime<Utc>` → サーバー設定のタイムゾーン

```rust
use chrono_tz::Tz;

// サーバー設定のタイムゾーンを取得
let guild_tz: Tz = timezone_service.get_guild_timezone(guild_id).await?;

// コマンド引数をパース（サーバー設定のタイムゾーンで解釈）
let utc_datetime = datetime_parser::parse_event_date(input_str, guild_tz)?;

// Facadeに渡す
facade.create_recruitment(utc_datetime).await?;

// 表示時はサーバー設定のタイムゾーンに変換
fn format_for_display(utc_datetime: DateTime<Utc>, guild_tz: Tz) -> String {
    let local = utc_datetime.with_timezone(&guild_tz);
    local.format("%Y/%m/%d %H:%M:%S %Z").to_string()
}
```

## 実例

### 募集作成のフロー

```rust
use chrono::{DateTime, Utc, Duration};
use chrono_tz::Tz;

// 1. サーバー設定のタイムゾーンを取得（プレゼンテーション層）
let guild_id = ctx.guild_id().unwrap();
let guild_tz: Tz = timezone_service.get_guild_timezone(guild_id).await?;
// 例: Asia/Tokyo

// 2. ユーザー入力をパース（プレゼンテーション層）
// ユーザー: "2025-01-01 19:00" (Asia/Tokyo)
let input_str = "2025-01-01 19:00";
let utc_datetime = datetime_parser::parse_event_date(input_str, guild_tz)?;
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
let local_display = utc_datetime.with_timezone(&guild_tz);
println!("出発時刻: {}", local_display.format("%Y/%m/%d %H:%M %Z"));
// Asia/Tokyoの場合: "出発時刻: 2025/01/01 19:00 JST"
```

## よくある間違い

### ❌ 間違い1: LocalをDBに保存

```rust
// NG
let now = Local::now();
model.created_at = now;  // サーバーのタイムゾーンに依存
```

### ❌ 間違い2: タイムゾーンオフセットをハードコード

```rust
// NG: JSTを決め打ち
let jst = utc + Duration::hours(9);

// NG: 固定オフセットを使用
let fixed_offset = FixedOffset::east_opt(9 * 3600).unwrap();
let local = utc.with_timezone(&fixed_offset);
```

正しくは、サーバー設定のタイムゾーンを取得して使用する：

```rust
// ✅ OK: サーバー設定のタイムゾーンを使用
let guild_tz: Tz = timezone_service.get_guild_timezone(guild_id).await?;
let local_display = utc_datetime.with_timezone(&guild_tz);
```

### ❌ 間違い3: タイムゾーン変換を複数箇所で行う

```rust
// NG: Service層で表示用の変換
service.format_datetime_local(utc, guild_tz);  // Service層の責務外

// ✅ OK: Presentation層でのみ変換
fn format_for_display(utc: DateTime<Utc>, guild_tz: Tz) -> String {
    let local = utc.with_timezone(&guild_tz);
    local.format("%Y/%m/%d %H:%M %Z").to_string()
}
```

### ❌ 間違い4: タイムゾーン未取得でパース

```rust
// NG: タイムゾーンを取得せずにJST決め打ち
let utc = datetime_parser::parse_event_date(input_str)?;

// ✅ OK: サーバー設定のタイムゾーンを取得してからパース
let guild_tz = timezone_service.get_guild_timezone(guild_id).await?;
let utc = datetime_parser::parse_event_date(input_str, guild_tz)?;
```

## タイムゾーン設定管理

### guild_timezones テーブル

各Discordサーバーのタイムゾーン設定を保存する。

- `guild_id`: Discord Guild ID（主キー）
- `timezone`: IANA タイムゾーン名（例: `Asia/Tokyo`, `America/New_York`）
- デフォルト値: `Asia/Tokyo`

### タイムゾーン取得サービス

```rust
pub struct TimezoneService {
    repository: Arc<GuildTimezoneRepository>,
}

impl TimezoneService {
    pub async fn get_guild_timezone(&self, guild_id: GuildId) -> Result<Tz> {
        // 1. DBから取得を試みる
        match self.repository.find_by_guild_id(guild_id).await? {
            Some(settings) => {
                // 2. タイムゾーン文字列をTz型に変換
                settings.timezone.parse::<Tz>()
                    .map_err(|_| Error::InvalidTimezone)
            },
            None => {
                // 3. 未設定の場合はデフォルト（Asia/Tokyo）を返す
                Ok(chrono_tz::Asia::Tokyo)
            }
        }
    }

    pub async fn set_guild_timezone(&self, guild_id: GuildId, timezone: Tz) -> Result<()> {
        // タイムゾーン設定を保存
        self.repository.upsert(guild_id, timezone.name()).await
    }
}
```

### タイムゾーン設定の変更

タイムゾーン設定を変更しても、既存のDB保存データ（UTC）には影響しない。表示のみが変わる。

```rust
// 変更前: Asia/Tokyo
// DB保存: 2025-01-01 10:00 UTC
// 表示: 2025/01/01 19:00 JST

// 変更後: America/New_York
// DB保存: 2025-01-01 10:00 UTC（変わらない）
// 表示: 2025/01/01 05:00 EST（表示だけ変わる）
```

## チェックリスト

実装時は以下を確認：

- [ ] DBに保存する日時は`DateTime<Utc>`型か
- [ ] `Local::now()`を使っていないか
- [ ] サーバー設定のタイムゾーンを取得しているか
- [ ] ユーザー入力をサーバー設定のタイムゾーンで解釈し、即座にUTCに変換しているか
- [ ] Service層・Repository層でタイムゾーン変換していないか
- [ ] 表示時のみサーバー設定のタイムゾーンへの変換を行っているか
- [ ] タイムゾーン変換時にコメントで明示しているか
- [ ] タイムゾーンオフセット（`+ Duration::hours(9)`等）をハードコードしていないか

## 参考

- Chrono documentation: https://docs.rs/chrono/
- Chrono-TZ documentation: https://docs.rs/chrono-tz/
- IANA Time Zone Database: https://www.iana.org/time-zones
- UTC vs Local: https://en.wikipedia.org/wiki/Coordinated_Universal_Time
