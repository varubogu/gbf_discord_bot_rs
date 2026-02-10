# 募集スケジュールファサード 結合テスト計画書

## 対象ファイル

- `src/facades/recruitment/recruitment_schedule_facade.rs`
- `src/facades/recruitment/recruitment_schedule_list.rs`

## テスト方針

- 実際のPostgreSQLデータベースを使用する結合テスト（`#[ignore]`付き）
- Gateway依存なし（DB操作のみ）
- テストデータ（quest, battle_style等のマスターデータ含む）は各テストのArrangeで投入し、テスト間の独立性を担保する
- テスト後に必ずクリーンアップを実施する

---

## 対象関数とテストケース

### 1. `create_recruitment_schedule`（スケジュール作成）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 1-1 | 正常系：基本的なスケジュール作成 | 有効なquest_alias、quest_start_time、days、recruit_start_time | `ScheduleCreationResult`が返り、DBにスケジュールが保存される |
| 1-2 | 正常系：battle_style_id指定あり | 有効なbattle_style_idを指定 | 攻略方法付きスケジュールが作成される |
| 1-3 | 正常系：note指定あり | noteパラメータを指定 | メモ付きスケジュールが作成される |
| 1-4 | 正常系：dismissal_times指定あり | 解散時刻文字列を指定 | 解散時刻付きスケジュールが作成される |
| 1-5 | 異常系：存在しないquest_alias | 存在しないクエスト名 | `AppError::NotFound`が返る |
| 1-6 | 異常系：無効な時刻フォーマット | quest_start_time="invalid" | `AppError::Business`（時刻形式エラー）が返る |

### 2. `list_recruitment_schedules`（スケジュール一覧取得）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 2-1 | 正常系：全スケジュール表示 | 複数スケジュールが登録済み、show_all=true | 全スケジュールが返る |
| 2-2 | 正常系：自分のスケジュールのみ表示 | 複数ユーザーのスケジュールあり、show_all=false | 指定user_idのスケジュールのみ返る |
| 2-3 | 正常系：スケジュール未登録時 | スケジュールが0件 | 空のVecが返る |

### 3. `delete_recruitment_schedule`（スケジュール削除）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 3-1 | 正常系：作成者による削除 | スケジュール作成者と同一user_id | スケジュールがDBから削除される |
| 3-2 | 正常系：管理者による削除 | is_admin=true、異なるuser_id | スケジュールがDBから削除される |
| 3-3 | 異常系：権限のない削除 | 作成者でなくis_admin=false | `AppError::Business`（権限なし）が返る |
| 3-4 | 異常系：存在しないスケジュールの削除 | 存在しないschedule_id | `AppError::Business`（未存在）が返る |

### 4. `toggle_recruitment_schedule`（スケジュール有効/無効切替）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 4-1 | 正常系：有効→無効への切替 | enabled=trueのスケジュール、作成者 | enabled=falseに更新される |
| 4-2 | 正常系：無効→有効への切替 | enabled=falseのスケジュール、作成者 | enabled=trueに更新される |
| 4-3 | 正常系：管理者による切替 | is_admin=true、異なるuser_id | 切替が正常に行われる |
| 4-4 | 異常系：権限のない切替 | 作成者でなくis_admin=false | `AppError::Business`（権限なし）が返る |

### 5. `get_schedules_for_autocomplete`（スケジュール候補取得）

| No | ケース | 前提条件 | 期待結果 |
|----|--------|----------|----------|
| 5-1 | 正常系：自分のスケジュール候補取得 | user_idに紐づくスケジュールあり | AutocompleteOptionのVecが返る |
| 5-2 | 正常系：スケジュール未登録時 | 対象user_idのスケジュールなし | 空のVecが返る |
