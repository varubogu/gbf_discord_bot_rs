# チャンネル管理ファサード 結合テスト計画書

## 対象ファイル

- `src/facades/channel/channel_management_facade.rs`

## テスト方針

- 実際のPostgreSQLデータベースを使用する結合テスト（`#[ignore]`付き）
- Gateway依存なし（DB操作のみ）
- テストデータは各テスト内で作成・テスト後にクリーンアップ

---

## 対象関数とテストケース

### 1. `get_channel_types_for_autocomplete`

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 1-1 | 正常系：チャンネル種別一覧を取得 | channel_typesテーブルにデータが存在する | `Vec<AutocompleteOption>`が返り、各要素にdisplay_nameとvalueが含まれる |
| 1-2 | 正常系：空のテーブルからの取得 | channel_typesテーブルが空 | 空の`Vec`が返る |

### 2. `register_channel`

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 2-1 | 正常系：新規ギルド・新規チャンネル登録 | ギルド未登録、有効なchannel_type_id | `ChannelRegistrationResult`が返り、ギルドとチャンネルがDBに保存される |
| 2-2 | 正常系：既存ギルドへのチャンネル登録 | ギルド登録済み、有効なchannel_type_id | チャンネルのみ新規登録される |
| 2-3 | 正常系：既存チャンネルの上書き登録 | 同じguild_id・channel_type_idで既にチャンネル登録済み | channel_idが新しい値に更新される |
| 2-4 | 異常系：存在しないchannel_type_id | 存在しないchannel_type_idを指定 | `AppError::NotFound`が返る |
| 2-5 | 正常系：登録結果にsettings_displayが含まれる | 正常登録後 | 結果の`settings_display`に登録済みチャンネル情報が反映されている |

### 3. `unregister_channel`

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 3-1 | 正常系：チャンネル登録解除 | 対象チャンネルが登録済み | `ChannelUnregistrationResult`が返り、DBからチャンネルが削除される |
| 3-2 | 異常系：未登録チャンネルの解除 | 対象チャンネルが未登録 | `AppError::NotFound`が返る |
| 3-3 | 異常系：存在しないchannel_type_id | 存在しないchannel_type_idを指定 | `AppError::NotFound`が返る |
| 3-4 | 正常系：解除結果にold_channel_idが含まれる | 正常解除後 | 結果のold_channel_idが解除前のchannel_idと一致する |

### 4. `show_channel_settings`

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 4-1 | 正常系：チャンネル設定表示（登録あり） | 複数チャンネルが登録済み | 登録済みチャンネルの情報が`ChannelSettingsDisplay`に含まれる |
| 4-2 | 正常系：チャンネル設定表示（登録なし） | 対象ギルドにチャンネル未登録 | 空の設定情報が返る |
