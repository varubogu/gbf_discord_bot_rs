# db不足テーブル

## ギルド版のみ存在するテーブルについて

guildsテーブルを基本とし、guild系テーブルはすべて外部制約で

### (guild)サーバー覧	guilds

列
- guild_id（i64,PK）

### (guild)サーバーチャンネル一覧	guild_channels
- guild_id（i64,PK）
- channel_type(i32,PK,channel_types.idと外部制約)

## グローバル版とギルド版の両方が存在するテーブルについて

下記テーブルはグローバルのテーブルを参考にし、guild_idを複合主キーとしてグローバル版のテーブル構造からギルド版のテーブルとする
動作としては主キーが同一のglobalとguild両方が存在する場合、そのテーブルはguild側のデータで上書きする

** (guild)環境変数	guild_environments**
** (guild)イベント期間内詳細スケジュール	guild_event_schedule_details**
** (guild)イベントスケジュール	guild_event_schedules**
** (guild)メッセージ	guild_message_texts**
** (guild)最終実行日時	guild_last_process_times**

