# 募集ファサード 結合テスト計画書

## 対象ファイル

- `src/facades/recruitment/new_recruit.rs`
- `src/facades/recruitment/cancel.rs`
- `src/facades/recruitment/change.rs`
- `src/facades/recruitment/participants.rs`
- `src/facades/recruitment/button_handler.rs`
- `src/facades/recruitment/quest_list.rs`
- `src/facades/recruitment/battle_style_list.rs`
- `src/facades/recruitment/role_management.rs`

## テスト方針

- 実際のPostgreSQLデータベースを使用する結合テスト（`#[ignore]`付き）
- Gateway依存の関数は`MockDiscordGateway`を使用
- テストデータ（quest, battle_style等のマスターデータ）はDBに事前登録が必要
- テストデータは各テスト内で作成・テスト後にクリーンアップ

---

## 対象関数とテストケース

### 1. `new_recruitment`（新規募集作成）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 1-1 | 正常系：ボタン版での新規募集作成 | 有効なquest_alias、use_buttons=true | `RecruitmentResult`が返り、DBに募集レコードが作成される。componentsにボタンが含まれる |
| 1-2 | 正常系：リアクション版での新規募集作成 | 有効なquest_alias、use_buttons=false | `RecruitmentResult`が返り、reaction_emojisが含まれる |
| 1-3 | 正常系：battle_style_id指定あり | 有効なbattle_style_id | 対応する攻略方法が反映された募集が作成される |
| 1-4 | 正常系：event_date指定あり | 有効なevent_date | 指定日時が開催日時として設定され、出発通知が登録される |
| 1-5 | 正常系：dismissal_times指定あり | 有効な解散時刻文字列 | 解散時刻がDBに登録され、メッセージに反映される |
| 1-6 | 異常系：存在しないquest_alias | 存在しないクエスト名を指定 | エラーが返る |
| 1-7 | 異常系：存在しないbattle_style_id | 無効なbattle_style_idを指定 | エラーが返る |

### 2. `update_message_id`（メッセージID更新）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 2-1 | 正常系：message_id更新 | 募集レコードがDB上に存在 | DBのmessage_idが更新される |
| 2-2 | 異常系：存在しないrecruitment_id | 存在しないrecruitment_id | エラーが返る |

### 3. `can_cancel`（キャンセル可否確認）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 3-1 | 正常系：キャンセル可能な募集 | 募集中・未キャンセル・開催日時前のレコードがDBに存在 | `CanCancelResult`でキャンセル可能が返る |
| 3-2 | 正常系：キャンセル済みの募集 | is_canceled=trueのレコード | キャンセル不可の結果が返る |
| 3-3 | 正常系：存在しないメッセージの募集 | DBに該当レコードなし | 募集が見つからない結果が返る |

### 4. `execute_cancel`（募集キャンセル実行）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 4-1 | 正常系：募集キャンセル | 募集中・開催日時前のレコードがDB上に存在、MockGateway設定済み | is_canceled=trueに更新、通知が削除される |
| 4-2 | 異常系：開催日時を過ぎた募集のキャンセル | quest_start_atが過去 | `AppError::Business`（開催日時を過ぎている）が返る |
| 4-3 | 異常系：存在しない募集のキャンセル | DBに該当レコードなし | `AppError::Business`が返る |

### 5. `cancel_on_message_deleted`（メッセージ削除時のキャンセル）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 5-1 | 正常系：募集メッセージ削除時のキャンセル | 募集中のレコード、MockGateway設定済み | `CancelOnDeleteResult::Cancelled`が返り、DBが更新される |
| 5-2 | 正常系：募集メッセージでない場合 | DBに該当する募集レコードなし | `CancelOnDeleteResult::NotRecruitmentMessage`が返る |
| 5-3 | 正常系：既にキャンセル済みの場合 | is_canceled=trueのレコード | `CancelOnDeleteResult::AlreadyCancelled`が返る |
| 5-4 | 正常系：開催日時を過ぎている場合 | quest_start_atが過去 | `CancelOnDeleteResult::EventDatePassed`が返る |

### 6. `change_recruitment_information`（募集情報変更）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 6-1 | 正常系：クエスト変更 | 募集中のレコード、有効なquest名、MockGateway設定済み | DBのquest_idが更新される |
| 6-2 | 正常系：開催日時変更 | 募集中のレコード、有効なevent_date | DBのquest_start_atが更新され、通知が再作成される |
| 6-3 | 正常系：攻略方法変更 | 募集中のレコード、有効なbattle_style_id | DBのbattle_style_idが更新される |
| 6-4 | 異常系：存在しない募集の変更 | DBに該当レコードなし | `AppError::NotFound`が返る |

### 7. `handle_recruitment_button`（ボタン操作）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 7-1 | 正常系：参加ボタン押下 | 募集中のレコード、custom_id="recruit_join" | 参加者がDBに追加される |
| 7-2 | 正常系：属性指定参加 | 募集中のレコード、custom_id="recruit_join_1" | 属性ID付きで参加者がDBに追加される |
| 7-3 | 正常系：全属性参加 | 募集中のレコード、custom_id="recruit_join_0" | 全属性で参加者がDBに追加される |
| 7-4 | 正常系：退出ボタン押下 | 参加済みの状態、custom_id="recruit_leave_all" | 参加者がDBから削除される |
| 7-5 | 異常系：キャンセル済み募集への操作 | is_canceled=trueのレコード | `AppError::Business`が返る |
| 7-6 | 異常系：期限切れ募集への操作 | quest_start_atが過去 | `AppError::Business`が返る |

### 8. `handle_recruitment_select_menu`（セレクトメニュー操作）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 8-1 | 正常系：複数属性選択 | 募集中のレコード、element_ids=[1,3] | 選択した属性で参加者がDBに追加される |
| 8-2 | 正常系：既存参加の上書き | 既に参加済み、新しいelement_idsを選択 | 古い参加が削除され新しい参加が追加される |
| 8-3 | 異常系：キャンセル済み募集への操作 | is_canceled=trueのレコード | `AppError::Business`が返る |

### 9. `search_quests_for_autocomplete`（クエスト検索）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 9-1 | 正常系：部分文字列でクエスト検索 | questテーブルにデータあり | マッチするクエストのAutocompleteOptionが返る |
| 9-2 | 正常系：マッチなし | 存在しないクエスト名を指定 | 空のVecが返る |

### 10. `get_battle_styles_for_autocomplete`（攻略方法一覧）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 10-1 | 正常系：攻略方法一覧取得 | battle_stylesテーブルにデータあり | 全攻略方法のAutocompleteOptionが返る |

### 11. `add_recruitment_notification_roles`（通知ロール追加）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 11-1 | 正常系：全募集用ロール追加 | quest_name_or_alias="すべて" | 全募集通知ロールがDBに追加される |
| 11-2 | 正常系：特定クエスト用ロール追加 | 有効なquest名 | クエスト個別通知ロールがDBに追加される |
| 11-3 | 正常系：重複ロールの追加 | 既に登録済みのrole_id | 重複分はスキップされ、新規追加分のみカウントされる |

### 12. `remove_recruitment_notification_roles`（通知ロール削除）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 12-1 | 正常系：ロール削除 | ロールが登録済み | 対象ロールがDBから削除され、削除件数が返る |
| 12-2 | 正常系：未登録ロールの削除 | 対象ロールが未登録 | 削除件数0が返る |

### 13. `show_recruitment_notification_roles`（通知ロール表示）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 13-1 | 正常系：全ロール設定表示 | 全募集ロール・クエスト別ロールが登録済み | `RecruitmentRoleSettings`に全情報が含まれる |
| 13-2 | 正常系：ロール未設定時の表示 | ロール未登録 | 空のロール設定が返る |
