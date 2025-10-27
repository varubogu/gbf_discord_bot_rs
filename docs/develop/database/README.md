# データベース設計書

## 概要

GBF Discord Botのデータベーステーブル設計を定義します。PostgreSQL + SeaORMを使用し、グローバル（全ギルド共通）とギルド固有のデータを管理します。

## テーブルスコープの概念

### スコープ分類

| スコープ | 説明 | データの性質 | 参照優先順位 |
|---------|------|------------|------------|
| **All（グローバル）** | 全ギルド共通のデータ | マスターデータ、参照データ | ギルド固有データがない場合のフォールバック |
| **Guild（ギルド固有）** | ギルド単位でカスタマイズ可能なデータ | ギルド独自の設定、カスタムデータ | 最優先で参照 |
| **Community（コミュニティ）** | ギルド内のユーザー活動データ | 募集情報など動的なデータ | ギルド固有のみ |

### データ参照ロジック

```
データ取得時の優先順位：
1. ギルド固有テーブル（guild_*）にデータが存在するか確認
2. 存在しない場合、グローバルテーブル（*）を参照
3. どちらにも存在しない場合、デフォルト値またはエラー
```

## テーブルタイプ分類

| タイプ | 説明 | データの性質 | 更新頻度 |
|-------|------|------------|---------|
| **Reference** | 参照データ、マスターデータ | 静的、定義的 | 低 |
| **Transaction** | トランザクションデータ | 動的、業務処理結果 | 高 |
| **History** | 履歴データ | 追記専用、時系列データ | 中 |

## テーブル一覧

### グローバルテーブル（All Scope）

全ギルド共通の参照データおよびトランザクションデータ。

| テーブル物理名 | テーブル日本語名 | テーブルタイプ | 設計書 |
|-------------|--------------|------------|-------|
| battle_types | マルチバトル戦術 | Reference | [battle_types.md](tables/global/battle_types.md) |
| quests | クエスト情報 | Reference | [quests.md](tables/global/quests.md) |
| quest_aliases | クエスト別名 | Reference | [quest_aliases.md](tables/global/quest_aliases.md) |
| elements | 属性定義 | Reference | [elements.md](tables/global/elements.md) |
| channel_types | チャンネル種類 | Reference | [channel_types.md](tables/global/channel_types.md) |
| environments | 環境変数 | Reference | [environments.md](tables/global/environments.md) |
| messages | メッセージ定義 | Reference | [messages.md](tables/global/messages.md) |
| event_schedules | イベントスケジュール | Reference | [event_schedules.md](tables/global/event_schedules.md) |
| event_schedule_details | イベント詳細スケジュール | Reference | [event_schedule_details.md](tables/global/event_schedule_details.md) |
| last_process_times | 最終処理実行日時 | History | [last_process_times.md](tables/global/last_process_times.md) |
| schedules | 通知スケジュール | Transaction | [schedules.md](tables/global/schedules.md) |
| battle_recruitment_schedules | マルチ募集スケジュール | Transaction | [battle_recruitment_schedules.md](tables/global/battle_recruitment_schedules.md) |

### ギルドテーブル（Guild Scope）

ギルド単位でカスタマイズ可能なデータ。グローバルテーブルの対応テーブルが存在する場合、ギルド固有データが優先される。

| テーブル物理名 | テーブル日本語名 | テーブルタイプ | 設計書 |
|-------------|--------------|------------|-------|
| guild_environments | ギルド環境変数 | Reference | [guild_environments.md](tables/guild/guild_environments.md) |
| guild_messages | ギルドメッセージ定義 | Reference | [guild_messages.md](tables/guild/guild_messages.md) |
| guild_channels | ギルドチャンネル | Reference | [guild_channels.md](tables/guild/guild_channels.md) |
| guild_event_schedules | ギルドイベントスケジュール | Reference | [guild_event_schedules.md](tables/guild/guild_event_schedules.md) |
| guild_event_schedule_details | ギルドイベント詳細スケジュール | Reference | [guild_event_schedule_details.md](tables/guild/guild_event_schedule_details.md) |
| guild_last_process_times | ギルド最終処理実行日時 | History | [guild_last_process_times.md](tables/guild/guild_last_process_times.md) |
| guild_spreadsheet_imports | ギルドスプレッドシート取込設定 | Reference | [guild_spreadsheet_imports.md](tables/guild/guild_spreadsheet_imports.md) |
| guild_spreadsheet_exports | ギルドスプレッドシート出力設定 | Reference | [guild_spreadsheet_exports.md](tables/guild/guild_spreadsheet_exports.md) |

### コミュニティテーブル（Community Scope）

ギルド内のユーザー活動データ。ギルド固有だがグローバルテーブルとの対応関係はない。

| テーブル物理名 | テーブル日本語名 | テーブルタイプ | 設計書 |
|-------------|--------------|------------|-------|
| battle_recruitments | マルチバトル募集情報 | Transaction | [battle_recruitments.md](tables/community/battle_recruitments.md) |

## テーブル関連図（概念）

```
[グローバル参照データ]
battle_types ──→ quests ──→ quest_aliases
                   │
                   ├──→ battle_recruitments (Community)
                   │
elements ──→ event_schedules ──→ event_schedule_details ──→ schedules ──→ battle_recruitment_schedules
                                                                │
                                                                └──→ notifications (予定)

[ギルド固有データ]
guild_environments
guild_spreadsheet_imports
guild_spreadsheet_exports
guild_messages
guild_channels ──→ channel_types (Global)
guild_event_schedules ──→ guild_event_schedule_details
guild_last_process_times

[データ参照フロー]
Application Layer
    ↓
1. guild_* テーブルを検索
    ↓（存在しない場合）
2. グローバルテーブルを検索
    ↓（存在しない場合）
3. デフォルト値 or エラー
```

## Rustとの対応

現在のRust実装（SeaORM）では、以下のエンティティが存在：

### 実装済みエンティティ（src/models/entities/）

- battle_types
- quests
- quest_aliases
- battle_recruitments
- event_schedules
- event_schedule_details
- message_texts（messagesに相当）
- environments
- guilds
- notifications（schedulesから移行）
- notification_rel_battle_recruitments
- notification_rel_event_schedules
- last_process_times

### 未実装エンティティ（Pythonには存在、Rustで未実装）

- elements
- channel_types
- guild_channels
- guild_environments
- guild_event_schedules
- guild_event_schedule_details
- guild_last_process_times
- guild_messages
- battle_recruitment_schedules（notificationsに統合済みの可能性）

## データ移行とスプレッドシート連携

### スプレッドシート構成

#### 「テーブル名」シート
- テーブルのメタ情報を定義
- **1行目**: マッピングキー（`sheet_name`, `table_name`, `table_scope`, `table_io`, `table_type` など）
- **2行目**: 日本語説明（任意）
- **3行目以降**: データ行
- `table_io`: `in`（読み込み）、`out`（書き込み）、`in,out`（双方向）
- `table_type`: `reference`, `transaction`, `history`
- 未定義のキー名は無視される

#### 各テーブルシート
- シート名 = テーブル日本語名
- 1行目: カラム物理名（PostgreSQLと同一）
- 2行目: カラム日本語名（人間用）
- 3行目以降: データ行

### グローバル・ギルド別スプレッドシート

| スプレッドシート種別 | 環境変数 | 対象テーブル | コマンド実行権限 |
|------------------|---------|------------|---------------|
| **グローバル用** | `GSPREAD_GLOBAL_URL` | All Scopeテーブル | BOT_ADMIN_SERVER限定 |
| **ギルド読み込み用** | `guild_spreadsheet_imports`テーブル | Guild Scopeテーブル（`/gspread_load`） | gbf_bot_controlロール |
| **ギルド書き込み用** | `guild_spreadsheet_exports`テーブル | Guild Scopeテーブル（`/gspread_push`） | gbf_bot_controlロール |

## セキュリティとアクセス制御

### 環境変数

```bash
# Bot管理者専用サーバーID（グローバルスプレッドシート操作権限）
BOT_ADMIN_SERVER_ID=123456789012345678

# Googleサービスアカウント認証
GOOGLE_SERVICE_ACCOUNT_KEY_FILE=/path/to/service-account-key.json
```

### 権限レベル

1. **Bot管理者**: グローバルスプレッドシート読み書き（全ギルド影響）
2. **ギルド管理者（gbf_bot_controlロール）**: ギルド固有スプレッドシート読み書き
3. **一般ユーザー**: 読み取りのみ（Bot動作経由）

## パフォーマンス考慮事項

### インデックス設計

- **複合UNIQUE制約**: 自然キーとして機能（guild_id, channel_id, message_id等）
- **外部キー**: 参照整合性を保証
- **UUIDプライマリキー**: 分散環境での一意性保証

### データ量見積もり

| テーブル | 想定レコード数 | 増加頻度 |
|---------|-------------|---------|
| quests | 数百〜数千 | 低（新クエスト追加時） |
| battle_recruitments | ギルドあたり数千〜数万 | 高（ユーザー活動） |
| event_schedules | 数百 | 低（イベント開催時） |
| schedules | 数万〜数十万 | 中（イベントごとに生成） |

## 将来の拡張性

### 追加予定テーブル

- **users**: ユーザー情報管理
- **user_settings**: ユーザー個別設定
- **audit_logs**: 操作履歴ログ

### マイグレーション戦略

- SeaORM Migrationを使用
- バージョン管理されたマイグレーションファイル（migration/src/m*.rs）
- ロールバック対応

## 参考情報

- Python実装: `.tmp/gbf_bot/src/gbf/models/`
- Rust実装: `src/models/entities/`
- マイグレーション: `migration/src/`
- タイムスタンプ自動化: [sea_orm_timestamp_automation.md](sea_orm_timestamp_automation.md)
