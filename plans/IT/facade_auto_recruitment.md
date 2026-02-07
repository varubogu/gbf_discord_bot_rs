# 自動募集ファサード 結合テスト計画書

## 対象ファイル

- `src/facades/auto_recruitment/quest_selection_facade.rs`
- `src/facades/auto_recruitment/time_selection_facade.rs`
- `src/facades/auto_recruitment/status_facade.rs`
- `src/facades/auto_recruitment/matching_check_facade.rs`
- `src/facades/auto_recruitment/category_setup_facade.rs`

## テスト方針

- 実際のPostgreSQLデータベースを使用する結合テスト（`#[ignore]`付き）
- `category_setup_facade`はGateway依存のため`MockDiscordGateway`を使用
- `matching_check_facade`はスタブ関数（ログ出力のみ）のため正常系のみテスト
- テストデータは各テスト内で作成・テスト後にクリーンアップ

---

## 対象関数とテストケース

### 1. `handle_quest_selection`（クエスト選択）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 1-1 | 正常系：クエスト選択登録 | 自動募集が登録済み、有効なquest_ids | `QuestSelectionResult::Registered`が返り、DBに選択が保存される |
| 1-2 | 正常系：クエスト選択の上書き | 既に選択済み、新しいquest_ids | 古い選択が削除され新しい選択が保存される |
| 1-3 | 異常系：自動募集未登録 | 対象ギルドに自動募集が未登録 | エラーが返る |

### 2. `handle_time_selection`（時間帯選択）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 2-1 | 正常系：時間帯選択登録 | 自動募集が登録済み、有効なmonth/day/hours | `TimeSelectionResult::Registered`が返り、DBに選択が保存される |
| 2-2 | 正常系：時間帯選択の上書き | 同一月日で既に選択済み、新しいhours | 古い選択が削除され新しい選択が保存される |
| 2-3 | 正常系：複数時間帯の選択 | hours=[21, 22, 23] | 3レコードがDBに作成される |
| 2-4 | 異常系：自動募集未登録 | 対象ギルドに自動募集が未登録 | エラーが返る |

### 3. `get_participation_status`（参加状況取得）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 3-1 | 正常系：クエスト・時間帯あり | クエスト選択・時間帯選択が登録済み | `ParticipationStatus`にquest_idsとtime_slotsが含まれる |
| 3-2 | 正常系：選択なし | 選択が未登録だが自動募集は登録済み | 空のquest_idsとtime_slotsが返る |
| 3-3 | 異常系：自動募集未登録 | 対象ギルドに自動募集が未登録 | エラーが返る |

### 4. `check_and_notify_after_quest_selection`（クエスト選択後通知）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 4-1 | 正常系：スタブ動作確認 | 任意のパラメータ | `Ok(())`が返る（ログ出力のみ） |

### 5. `check_and_notify_after_time_selection`（時間帯選択後通知）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 5-1 | 正常系：スタブ動作確認 | 任意のパラメータ | `Ok(())`が返る（ログ出力のみ） |

### 6. `register_category`（カテゴリ登録）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 6-1 | 正常系：カテゴリ登録 | 有効なcategory_id、days=3、MockGateway設定済み | `CategoryRegistrationResult`が返り、DBにカテゴリ・チャンネルが登録される |
| 6-2 | 異常系：days範囲外（1以下） | days=1 | バリデーションエラーが返る |
| 6-3 | 異常系：days範囲外（8以上） | days=8 | バリデーションエラーが返る |
| 6-4 | 異常系：既に登録済み | 同一ギルドで自動募集が登録済み | エラーが返る |

### 7. `unregister_category`（カテゴリ登録解除）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 7-1 | 正常系：カテゴリ登録解除 | 自動募集が登録済み、MockGateway設定済み | DBからカテゴリ・チャンネルが削除される |
| 7-2 | 異常系：カテゴリチャンネル内でのコマンド実行 | command_channel_idがカテゴリ内のチャンネル | `InCategoryChannelError`が返る |
| 7-3 | 異常系：未登録ギルドでの解除 | 自動募集が未登録 | エラーが返る |

### 8. `change_days`（日数変更）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 8-1 | 正常系：日数増加 | 現在3日→new_days=5、MockGateway設定済み | チャンネルが2つ追加される |
| 8-2 | 正常系：日数減少 | 現在5日→new_days=3、MockGateway設定済み | チャンネルが2つ削除される |
| 8-3 | 異常系：同じ日数への変更 | 現在3日→new_days=3 | エラーが返る |
| 8-4 | 異常系：範囲外の日数 | new_days=1 or 8 | バリデーションエラーが返る |
