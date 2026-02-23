# master.channel_types テーブル設計

## 概要

- スキーマ: `master`
- テーブル: `channel_types`
- 実装ソース: `src/models/entities/master/channel_types.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `i32` | NO | 主キー |
| `name` | `String` | NO |  |
| `memo` | `Option<String>` | YES |  |

## マスターデータ値

| id | name | memo |
| --- | --- | --- |
| 1 | イベントスケジュール通知 | イベントスケジュール通知の送信先チャンネル |
| 2 | マルチ募集 | マルチ募集メッセージの送信先チャンネル |
| 3 | 団連絡用 | 団連絡の通知先（団員のみ閲覧可能なチャンネルの場合、Botにも権限を与えてください） |
| 4 | マルチ募集チャンネル（他サーバー共用） | 外部のguildで募集した時用の通知先。通常のマルチ募集チャンネルと同じでも良いし、未定義も可能 |
| 5 | 管理者通知 | bot実行中のエラーや設定不足を管理者（gbf_bot_controlロール保持者）に通知するチャンネル |

## Rust enum

このテーブルのIDは `src/models/entities/master/channel_types.rs` に定義された `GuildChannelType` enumで表現されます。
固定マスターIDをenumで表現するルールについてはコーディング規約を参照してください。

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
