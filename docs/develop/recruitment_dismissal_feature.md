# マルチ募集解散機能 設計書

> **✅ 実装完了レポート**
>
> **実装完了日**: 2025-12-26
>
> **完了した実装:**
> - 人数不足による自動解散機能（Dismissal）の実装完了
> - DismissalTaskExecutor の実装完了
> - task_type=5 (Dismissal) と task_type=2 (Dissolution) の違いを明確化
>
> **DismissalとDissolutionの違い:**
> - **Dismissal (task_type=5)**: 人数不足による自動解散（このドキュメントの機能）
> - **Dissolution (task_type=2)**: クエスト開始時刻による強制解散
>
> **設計変更:**
> - `DismissalTimeParserService` は `unified_datetime_parser` に統合されました

## 概要

マルチ募集において、出発時刻より前に人数が集まらなかった場合に自動的に解散（キャンセル）する機能です（実装完了）。

### 目的

- メンバー全員を身内で固めたい場合に、事前に指定した時刻で人数チェックを行い、集まっていなければ自動的に募集をキャンセルする
- 解散は通常のキャンセルと同じ扱いだが、「人数不足で解散した」ことを明示的に通知する

### 基本動作

1. ユーザーは募集作成時に「解散時刻」を最大3つまで指定できる
2. 指定した解散時刻になったら、参加者数をチェック
3. 定員に達していなければ、通常のキャンセル処理を実行し、解散通知メッセージを送信
4. 定員に達していれば、解散処理はスキップ

---

## 要件定義

### 機能要件

#### FR-1: 解散時刻パラメータの追加

- マルチ募集コマンド（`/recruit_new`、`/recruit_new_v2`）に解散時刻パラメータを追加
- 定期募集コマンド（`/recruit_schedule_add`）にも解散時刻パラメータを追加
- パラメータ名: `dismissal_times`
- 入力形式: 文字列（任意項目）
- カンマ区切りで最大3つまで指定可能

#### FR-2: 解散時刻の入力形式

解散時刻は以下の2つの形式で指定可能:

1. **日時入力または時刻入力**
   - 電子機器での表現と日本語表現に対応
   - 例: `21:00`, `21時半`, `12/22 21:00`, `12月22日 21時半`
   - 既存の `parse_event_date()` 関数を活用

2. **出発前のn時間前という相対指定**
   - 電子機器での表現、日本語表現、英語表現に対応
   - 数字と単位の間にスペースがあってもOK
   - 指定可能な単位: 日（day/days）、時（hour/hours/h）、分（minute/minutes/min/m）
   - 例: `1日`, `1日前`, `1days`, `1day`, `1時間前`, `1hours`, `1 hours`, `1hour`, `1 hour`, `1h`, `90分前`, `90分`, `90minutes`, `90min`, `90m`

#### FR-3: 解散時刻のバリデーション

- カンマ分割した結果、空文字やスペースのみの文字は無視（データとしてカウントしない）
- データを4つ以上入力した場合 → 入力エラー
- 1つでも解析できない文字があった場合 → 入力エラー
- 解散時刻が出発時刻から見て丸7日（24h×7）を超えて指定された場合 → エラー
  - 例: 出発が12/22 22:00の場合、解散として指定できるのは12/15 22:00はOK、12/15 21:59以前はNG
- 最大日数は環境変数 `DISMISSAL_MAX_DAYS` で設定可能（デフォルト: 7日）

#### FR-4: 解散時刻の解釈ルール

**時刻のみの場合:**
- 出発日時が「12/22 22:00」で解散時刻が「21:00」の場合 → 12/22 21:00（1時間前）
- 出発日時が「12/22 22:00」で解散時刻が「23:00」の場合 → 12/21 23:00（前日の23時）

**日付のみの場合:**
- その日付の23:59:59を解散時刻とする

**日時の場合:**
- 指定された日時そのまま

#### FR-5: 解散処理の実行

- 解散時刻になったら、scheduled_tasksから起動
- 参加者数をチェックし、定員に達していなければ以下を実行:
  1. 募集をキャンセル状態に更新（`is_canceled = true`）
  2. 募集メッセージを通常のキャンセルと同じ形式で更新
  3. 解散通知メッセージを送信（シンプルな内容: 「人数が集まらなかったため解散しました」）
  4. 募集に紐づく他の通知（出発5分前、出発時刻）を削除

#### FR-6: 通知メッセージ

新しいメッセージテキストIDを追加:
- `RecruitmentDismissalNotification`: 「人数が集まらなかったため、この募集は解散しました。」

---

## データベース設計

### テーブル追加

#### 1. worker.battle_recruitment_dismissals

マルチ募集の解散時刻を保存するテーブル。

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| id | SERIAL | PRIMARY KEY | ID |
| recruitment_id | INTEGER | NOT NULL, FK (battle_recruitments.id) | 募集ID |
| input_value | TEXT | NOT NULL | ユーザー入力値（解析前の文字列） |
| input_type | INTEGER | NOT NULL | 入力タイプ（1: 絶対日時, 2: 相対時刻） |
| dismissal_datetime | TIMESTAMPTZ | NULL | 解散日時（絶対日時の場合のみ） |
| relative_days | INTEGER | NULL | 相対日数（相対時刻の場合のみ） |
| relative_hours | INTEGER | NULL | 相対時間数（相対時刻の場合のみ） |
| relative_minutes | INTEGER | NULL | 相対分数（相対時刻の場合のみ） |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

**インデックス:**
- `idx_battle_recruitment_dismissals_recruitment_id` ON `recruitment_id`

**外部キー:**
- `recruitment_id` → `worker.battle_recruitments(id)` ON DELETE CASCADE

**入力タイプ定義:**
- `1`: 絶対日時（dismissal_datetimeに値を格納）
- `2`: 相対時刻（relative_days, relative_hours, relative_minutesに値を格納）

#### 2. guild_master.battle_recruitment_schedule_dismissals

定期募集の解散時刻を保存するテーブル。

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| id | SERIAL | PRIMARY KEY | ID |
| schedule_id | INTEGER | NOT NULL, FK (battle_recruitment_schedules.id) | 定期募集ID |
| input_value | TEXT | NOT NULL | ユーザー入力値（解析前の文字列） |
| input_type | INTEGER | NOT NULL | 入力タイプ（1: 絶対日時, 2: 相対時刻） |
| dismissal_time | TIME | NULL | 解散時刻（絶対時刻の場合のみ、UTC） |
| relative_days | INTEGER | NULL | 相対日数（相対時刻の場合のみ） |
| relative_hours | INTEGER | NULL | 相対時間数（相対時刻の場合のみ） |
| relative_minutes | INTEGER | NULL | 相対分数（相対時刻の場合のみ） |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

**インデックス:**
- `idx_battle_recruitment_schedule_dismissals_schedule_id` ON `schedule_id`

**外部キー:**
- `schedule_id` → `guild_master.battle_recruitment_schedules(id)` ON DELETE CASCADE

**注意点:**
- 定期募集の場合、絶対時刻は時刻のみ（TIME型）で保存
- 日時は定期募集の性質上、毎回異なるため保存しない

#### 3. worker.scheduled_task_dismissals

scheduled_tasksと解散タスクの紐付けテーブル。

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| task_id | INTEGER | PRIMARY KEY, FK (scheduled_tasks.id) | タスクID |
| recruitment_dismissal_id | INTEGER | NOT NULL, FK (battle_recruitment_dismissals.id) | 解散設定ID |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |

**外部キー:**
- `task_id` → `worker.scheduled_tasks(id)` ON DELETE CASCADE
- `recruitment_dismissal_id` → `worker.battle_recruitment_dismissals(id)` ON DELETE CASCADE

---

## アーキテクチャ設計

### レイヤー構成

Clean Architectureに従い、以下のレイヤーで実装:

```
events (コマンドハンドラー)
  ↓
facades (トランザクション管理)
  ↓
services (ビジネスロジック)
  ↓
repository (データアクセス)
```

### 主要コンポーネント

#### 1. Services層

##### DismissalTimeParserService

解散時刻の文字列をパースするサービス。

**責務:**
- ユーザー入力文字列を解析し、絶対日時または相対時刻に変換
- 入力形式のバリデーション

**主要メソッド:**
```rust
pub struct DismissalTimeParserService;

impl DismissalTimeParserService {
    /// 解散時刻文字列をパース
    /// 戻り値: Vec<ParsedDismissalTime>
    pub fn parse(
        input: &str,
        departure_time: DateTime<Utc>,
        timezone: Tz,
        max_days: i32,
    ) -> Result<Vec<ParsedDismissalTime>>;
}

pub enum ParsedDismissalTime {
    Absolute {
        input_value: String,
        datetime: DateTime<Utc>,
    },
    Relative {
        input_value: String,
        days: i32,
        hours: i32,
        minutes: i32,
    },
}
```

**処理フロー:**
1. カンマで分割
2. 各要素をトリム、空文字は無視
3. 4つ以上あればエラー
4. 各要素を以下の順でパース:
   a. 相対時刻パターン（例: "1時間前", "90分", "1day"）
   b. 絶対日時パターン（既存の `parse_event_date()` を使用）
5. パース成功したら、7日以内かチェック
6. すべて成功すれば `Vec<ParsedDismissalTime>` を返す

##### DismissalManagementService

解散時刻の登録・削除を管理するサービス。

**責務:**
- 解散時刻をDBに保存
- scheduled_tasksに解散タスクを登録
- 解散時刻の削除

**主要メソッド:**
```rust
pub struct DismissalManagementService;

impl DismissalManagementService {
    /// マルチ募集の解散時刻を登録
    pub async fn create_recruitment_dismissals(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        dismissal_times: Vec<ParsedDismissalTime>,
        departure_time: DateTime<Utc>,
        guild_id: i64,
        channel_id: i64,
    ) -> Result<()>;

    /// マルチ募集の解散時刻を削除
    pub async fn delete_recruitment_dismissals(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
    ) -> Result<usize>;
}
```

**処理フロー（create_recruitment_dismissals）:**
1. 各 `ParsedDismissalTime` に対して:
   a. `battle_recruitment_dismissals` テーブルにレコード作成
   b. 解散実行日時を計算（相対時刻の場合は departure_time から計算）
   c. `scheduled_tasks` テーブルにタスク作成（task_type=2: Dissolution）
   d. `scheduled_task_dismissals` テーブルに紐付け作成

##### DismissalTaskExecutor

解散タスクを実行するサービス。

**責務:**
- scheduled_tasksから起動され、解散処理を実行
- 参加者数チェック
- 定員未達の場合、キャンセル処理と通知送信

**主要メソッド:**
```rust
pub struct DismissalTaskExecutor;

impl DismissalTaskExecutor {
    /// 解散タスクを実行
    pub async fn execute(
        &self,
        task_id: i32,
        db: &DatabaseConnection,
    ) -> Result<()>;
}
```

**処理フロー:**
1. `scheduled_task_dismissals` から `recruitment_dismissal_id` を取得
2. `battle_recruitment_dismissals` から `recruitment_id` を取得
3. `battle_recruitments` から募集情報を取得
4. 既にキャンセル済みならスキップ
5. 参加者数を取得（`battle_recruitment_participants` テーブルをカウント）
6. クエストの定員を取得（`quests` テーブル）
7. 定員に達していたらスキップ
8. 定員未達の場合:
   a. 既存のキャンセル処理を呼び出し（`cancel_recruitment_by_message`）
   b. 解散通知メッセージを送信
   c. 募集に紐づく他の通知を削除（`NotificationManagementService::delete_recruitment_notifications`）

#### 2. Repository層

新しいRepositoryを追加:

- `BattleRecruitmentDismissalRepository`
- `BattleRecruitmentScheduleDismissalRepository`
- `ScheduledTaskDismissalRepository`

各Repositoryは標準的なCRUD操作を提供。

#### 3. Facades層

既存のFacadeを拡張:

##### recruitment::new_recruit

マルチ募集作成Facade。解散時刻の登録処理を追加。

**修正内容:**
- パラメータに `dismissal_times: Option<String>` を追加
- 募集作成後、`DismissalManagementService::create_recruitment_dismissals` を呼び出し

##### recruitment::schedule

定期募集作成Facade。解散時刻の登録処理を追加。

**修正内容:**
- パラメータに `dismissal_times: Option<String>` を追加
- 定期募集作成後、`BattleRecruitmentScheduleDismissalRepository` に保存
- 定期募集から実際の募集を生成する際、解散時刻も一緒に登録

#### 4. Events層

既存のコマンドハンドラーを拡張:

- `/recruit_new`: `dismissal_times` パラメータを追加
- `/recruit_new_v2`: `dismissal_times` パラメータを追加
- `/recruit_schedule_add`: `dismissal_times` パラメータを追加

---

## 実装方針

### フェーズ1: データベースとエンティティ

1. マイグレーションファイル作成（3テーブル）
2. エンティティ定義作成（SeaORM）
3. Repository実装

### フェーズ2: パーサーとサービス

1. `DismissalTimeParserService` 実装
2. 単体テスト作成（パーサー）
3. `DismissalManagementService` 実装
4. 単体テスト作成（管理サービス）

### フェーズ3: コマンド統合

1. マルチ募集コマンドに `dismissal_times` パラメータ追加
2. Facade層で解散時刻登録処理を追加
3. 統合テスト作成

### フェーズ4: 定期募集統合

1. 定期募集コマンドに `dismissal_times` パラメータ追加
2. 定期募集作成時の解散時刻保存処理追加
3. 定期募集からマルチ募集生成時の解散時刻登録処理追加
4. 統合テスト作成

### フェーズ5: 解散タスク実行

1. `DismissalTaskExecutor` 実装
2. scheduler_managerに解散タスク処理を追加
3. 統合テスト作成

---

## テスト戦略

### 単体テスト

#### DismissalTimeParserService

- 相対時刻パース（日本語、英語、数字+単位）
- 絶対日時パース（既存パーサーとの統合）
- カンマ区切り複数指定
- エラーケース（4つ以上、7日超過、不正形式）

#### DismissalManagementService

- 解散時刻の登録（絶対/相対）
- scheduled_tasksへの登録
- 解散時刻の削除

#### DismissalTaskExecutor

- 定員未達時の解散処理
- 定員達成時のスキップ
- キャンセル済みのスキップ

### 統合テスト

- マルチ募集作成 → 解散時刻登録 → scheduled_tasks確認
- 定期募集作成 → 解散時刻保存 → マルチ募集生成 → scheduled_tasks確認
- 解散タスク実行 → キャンセル確認 → 通知送信確認

---

## 環境変数

### master.environments テーブル

新しい環境変数を追加:

| キー | デフォルト値 | 説明 |
|-----|-------------|------|
| DISMISSAL_MAX_DAYS | 7 | 解散時刻の最大指定可能日数 |

プログラム内で以下のように取得:

```rust
const DEFAULT_DISMISSAL_MAX_DAYS: i32 = 7;

// environments テーブルから取得、なければデフォルト値
let max_days = environment_repo
    .get_value(db, "DISMISSAL_MAX_DAYS")
    .await?
    .and_then(|v| v.parse::<i32>().ok())
    .unwrap_or(DEFAULT_DISMISSAL_MAX_DAYS);
```

---

## メッセージ定義

### 新規メッセージテキストID

`MessageTextId` enumに以下を追加:

```rust
pub enum MessageTextId {
    // ... 既存 ...

    // Recruitment dismissal messages
    RecruitmentDismissalNotification,
}
```

### メッセージ内容（デフォルト）

| メッセージID | 内容 |
|-------------|------|
| RecruitmentDismissalNotification | 人数が集まらなかったため、この募集は解散しました。 |

yamlファイルへの追加:

```yaml
recruitment:
  # ... 既存 ...
  dismissal_notification: "人数が集まらなかったため、この募集は解散しました。"
```

---

## マイグレーション

### マイグレーションファイル

```
migration/src/mXXXXXXXXXX_add_recruitment_dismissal.rs
```

以下の3つのテーブルを作成:
1. `worker.battle_recruitment_dismissals`
2. `guild_master.battle_recruitment_schedule_dismissals`
3. `worker.scheduled_task_dismissals`

---

## 非機能要件

### パフォーマンス

- 解散時刻パースは募集作成時に1回のみ実行
- scheduled_tasksのポーリング間隔は既存と同じ（1分間隔）
- 解散タスク実行時のDB問い合わせは最小限に

### セキュリティ

- RLSポリシーを適用（guild_idによるアクセス制御）
- ユーザー入力の解散時刻はバリデーション必須

### 可用性

- 解散タスク実行失敗時はログ出力してスキップ（他のタスクに影響を与えない）
- トランザクションで整合性を保証

---

## ロールバック計画

### データベース

マイグレーションのdown関数で3つのテーブルを削除:

```rust
async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
        .drop_table(Table::drop().table(ScheduledTaskDismissals::Table).to_owned())
        .await?;
    manager
        .drop_table(Table::drop().table(BattleRecruitmentScheduleDismissals::Table).to_owned())
        .await?;
    manager
        .drop_table(Table::drop().table(BattleRecruitmentDismissals::Table).to_owned())
        .await?;
    Ok(())
}
```

### コード

- コマンドパラメータから `dismissal_times` を削除
- Facade、Service層の解散関連コードを削除
- エンティティとRepositoryを削除

---

## 今後の拡張

### 将来的な改善案

1. **解散通知のカスタマイズ**
   - 参加者数や定員を含むメッセージ
   - ギルド毎のカスタムメッセージ

2. **解散時刻の変更機能**
   - 既存募集の解散時刻を変更するコマンド

3. **解散履歴の保存**
   - 解散理由や参加者数を履歴として保存

4. **複数条件での解散**
   - 特定の属性が足りない場合など、より複雑な条件

---

## 参考資料

### 既存実装の参考箇所

- 出発日時パース: `src/services/datetime_parser.rs`
- 通知管理: `src/services/schedule/notification_management_service.rs`
- キャンセル処理: `src/services/recruitment/cancel.rs`, `src/facades/recruitment/cancel.rs`
- scheduled_tasks実行: `src/services/schedule/dissolution_task_executor.rs`（既存の解散処理、参考になる）

### 関連テーブル

- `worker.battle_recruitments`: マルチ募集
- `guild_master.battle_recruitment_schedules`: 定期募集
- `worker.notifications`: 通知
- `worker.scheduled_tasks`: スケジュールタスク
- `worker.notification_rel_battle_recruitments`: 通知と募集の紐付け

---

## 付録

### 用語集

| 用語 | 説明 |
|-----|------|
| 解散 | 人数不足により募集をキャンセルすること |
| 解散時刻 | 人数チェックを行う時刻 |
| 相対時刻 | 出発時刻からの相対的な時間（例: 1時間前） |
| 絶対日時 | 具体的な日時（例: 12/22 21:00） |

### エラーメッセージ一覧

| エラーケース | メッセージ |
|-------------|-----------|
| 4つ以上指定 | 解散時刻は最大3つまで指定できます |
| パース失敗 | 解散時刻の形式が正しくありません: {input} |
| 7日超過 | 解散時刻は出発時刻の7日前までしか指定できません |
| 出発時刻より後 | 解散時刻は出発時刻より前である必要があります |
