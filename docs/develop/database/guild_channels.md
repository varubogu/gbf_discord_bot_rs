# guild_channels テーブル設計書

## 概要

ギルドごとの通知チャンネル設定を管理するテーブル。
イベントスケジュール通知など、用途別にDiscordチャンネルを設定するために使用する。

## テーブル定義

### テーブル名
`guild_channels`

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| guild_id | BIGINT | NOT NULL, PRIMARY KEY | DiscordギルドID |
| channel_type | INTEGER | NOT NULL, PRIMARY KEY | チャンネル種別ID（channel_typesテーブルのIDを参照） |
| channel_id | BIGINT | NOT NULL | DiscordチャンネルID |
| created_at | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT CURRENT_TIMESTAMP | 作成日時 |
| updated_at | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT CURRENT_TIMESTAMP | 更新日時 |

### 主キー
- 複合主キー: `(guild_id, channel_type)`

### 外部キー
- `channel_type` → `channel_types(id)`
  - 参照整合性制約あり
  - ON DELETE RESTRICT（channel_typeが使用中の場合は削除不可）

### インデックス
- 主キーインデックス: `(guild_id, channel_type)` (自動作成)
- 検索用インデックス: `idx_guild_channels_guild_id` ON `guild_id`
  - ギルドIDでの検索を高速化

## データ制約

### 一意性制約
- `(guild_id, channel_type)` の組み合わせは一意
- 同じギルドで同じチャンネル種別は1つのチャンネルのみ設定可能

### NULL制約
- すべてのカラムがNOT NULL
- channel_idは必ず有効なDiscordチャンネルIDを設定

## 使用例

### データ例

```sql
-- ギルド123456789の募集チャンネル設定
INSERT INTO guild_channels (guild_id, channel_type, channel_id)
VALUES (123456789, 1, 987654321);

-- ギルド123456789のイベント通知チャンネル設定
INSERT INTO guild_channels (guild_id, channel_type, channel_id)
VALUES (123456789, 2, 987654322);
```

### 想定されるchannel_types

| ID | name | memo |
|----|------|------|
| 1 | recruit | マルチ募集チャンネル |
| 2 | event | イベント通知チャンネル |
| 3 | announcement | お知らせチャンネル |

※ channel_typesテーブルのデータは別途定義

## 関連テーブル

### guilds
- `guild_id` で関連
- 1つのギルドに対して複数のチャンネル設定が可能（1対多）

### channel_types
- `channel_type` で関連
- チャンネル種別のマスターデータ

## 運用

### データ登録
- `/channel_register` コマンド（後日実装予定）で登録
- 管理者権限が必要

### データ更新
- 同じ `(guild_id, channel_type)` で再登録すると上書き
- `ON CONFLICT (guild_id, channel_type) DO UPDATE` を使用

### データ削除
- ギルドからBotが削除された場合、該当ギルドのデータを削除
- チャンネルが削除された場合、該当レコードを削除

## スケジュール通知での使用

### 処理フロー

1. `event_schedule_details.notification_channel_type` を取得
2. 各ギルドの `guild_channels` テーブルから該当する `channel_type` のレコードを検索
3. 取得した `channel_id` を通知先として使用

### クエリ例

```sql
-- 通知先チャンネル一覧を取得
SELECT
    gc.guild_id,
    gc.channel_id,
    ct.name as channel_type_name
FROM guild_channels gc
INNER JOIN channel_types ct ON gc.channel_type = ct.id
WHERE gc.channel_type = 2  -- イベント通知チャンネル
ORDER BY gc.guild_id;
```

### スケジュール生成時の処理

```rust
// event_schedule_details.notification_channel_type に基づいて
// 該当するギルド・チャンネルのペアを取得
let guild_channels = guild_channels::Entity::find()
    .filter(guild_channels::Column::ChannelType.eq(detail.notification_channel_type))
    .all(&db)
    .await?;

// 各ギルド・チャンネルに対して通知スケジュールを生成
for gc in guild_channels {
    let schedule = calculate_schedule(&event_schedule, &detail, gc.guild_id, gc.channel_id);
    schedules.push(schedule);
}
```

## セキュリティ考慮事項

### 権限チェック
- チャンネル登録時にBotが該当チャンネルへの書き込み権限を持つか確認
- 管理者権限を持つユーザーのみが設定変更可能

### データ検証
- `channel_id` が有効なDiscordチャンネルIDであることを確認
- `channel_type` が `channel_types` テーブルに存在することを確認（外部キー制約）
- `guild_id` がBotが参加しているギルドであることを確認

## パフォーマンス考慮事項

### インデックス戦略
- `guild_id` でのフィルタリングが頻繁に行われるため、インデックスを作成
- 複合主キーによる `(guild_id, channel_type)` での検索も高速

### クエリ最適化
- スケジュール生成時は `channel_type` でフィルタして一括取得
- ギルド数が多い場合でもインデックスにより高速な検索が可能

## 将来の拡張性

### 想定される拡張
1. **チャンネル設定の優先順位**
   - 複数のチャンネルを登録可能にして優先順位を設定
   - `priority` カラムを追加

2. **通知の有効/無効フラグ**
   - チャンネルは登録するが一時的に通知を無効化
   - `is_enabled` カラムを追加

3. **通知時間帯の制限**
   - 特定の時間帯のみ通知を送信
   - `notify_start_time`, `notify_end_time` カラムを追加

4. **メンション設定**
   - チャンネル種別ごとにメンション対象ロールを設定
   - 別テーブル `guild_channel_mentions` を作成

## マイグレーション

### 作成
- マイグレーションファイル: `m20251124_create_guild_channels.rs`
- 実行順序: channel_types テーブルの後

### ロールバック
- 外部キー制約により、使用中のチャンネル設定がある場合は削除不可
- 依存する通知データを先に削除する必要がある

## テスト戦略

### 単体テスト
- 複合主キーの一意性制約テスト
- 外部キー制約テスト
- NOT NULL制約テスト

### 統合テスト
- チャンネル登録処理のテスト
- スケジュール生成時のチャンネル取得テスト
- チャンネル更新・削除処理のテスト

## 備考

### 既存のrecruit_channel_idとの関係
- `guilds.recruit_channel_id` は後方互換性のため残す
- 新しい実装では `guild_channels` テーブルを使用
- マイグレーション時に既存の `recruit_channel_id` を `guild_channels` に移行することを推奨

### データ移行例

```sql
-- 既存のrecruit_channel_idをguild_channelsに移行
INSERT INTO guild_channels (guild_id, channel_type, channel_id)
SELECT
    guild_id,
    1 as channel_type,  -- 1 = recruit
    recruit_channel_id
FROM guilds
WHERE recruit_channel_id IS NOT NULL
ON CONFLICT (guild_id, channel_type) DO NOTHING;
```
