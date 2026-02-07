# ギルド設定ファサード 結合テスト計画書

## 対象ファイル

- `src/facades/guild_settings/guild_settings_facade.rs`

## テスト方針

- 実際のPostgreSQLデータベースを使用する結合テスト（`#[ignore]`付き）
- Gateway依存なし（DB操作のみ）
- `get_timezones_for_autocomplete`はDB不要のため`#[ignore]`なしでテスト可能

---

## 対象関数とテストケース

### 1. `get_timezones_for_autocomplete`

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 1-1 | 正常系：部分文字列でタイムゾーン候補取得 | `partial = "Asia/T"` | `Asia/Tokyo`等のマッチするタイムゾーンが返る |
| 1-2 | 正常系：空文字列での候補取得 | `partial = ""` | 全タイムゾーンの先頭候補が返る（最大25件） |
| 1-3 | 正常系：マッチなしの候補取得 | `partial = "XXXXXX"` | 空の`Vec`が返る |

### 2. `get_timezone`

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 2-1 | 正常系：設定済みタイムゾーンの取得 | guild_settingsに`America/New_York`が設定済み | `America/New_York`の`Tz`が返る |
| 2-2 | 正常系：未設定時のデフォルト値 | guild_settingsが未設定 | `Asia/Tokyo`（デフォルト）が返る |

### 3. `get_guild_settings`

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 3-1 | 正常系：設定済みギルド設定の取得 | guild_settingsにtimezoneとlocaleが設定済み | `Some(GuildSettingsResult)`が返り、正しい値が含まれる |
| 3-2 | 正常系：未設定ギルド設定の取得 | 対象ギルドの設定が未登録 | `None`が返る |

### 4. `set_timezone`

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 4-1 | 正常系：新規タイムゾーン設定 | guild_settingsが未設定 | `TimezoneSetResult`が返り、DBに設定が保存される |
| 4-2 | 正常系：タイムゾーン変更 | 既に`Asia/Tokyo`が設定済み | `America/New_York`に更新され、DBに反映される |
| 4-3 | 異常系：無効なタイムゾーン文字列 | `timezone_str = "Invalid/Timezone"` | バリデーションエラーが返る |
| 4-4 | 正常系：ロケール変更 | 既に`ja`ロケール設定済み | `en`ロケールに更新される |
