# ギルドチャンネル（guild_channels）

## 概要

**テーブル物理名**: `guild_channels`
**スキーマ名**: `guild_master`
**テーブルタイプ**: Reference
**テーブルスコープ**: Guild（ギルド固有）
**実装状況**: ✅ 実装済み

## 用途

ギルドのチャンネル情報と用途の紐づけを管理します。各チャンネルの役割（募集用、通知用など）を定義し、Botの動作を制御します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| guild_id | BigInteger | PK, NOT NULL | ギルドID（Discord Guild ID） |
| channel_id | BigInteger | PK, NOT NULL | チャンネルID（Discord Channel ID） |
| channel_type | Integer | NOT NULL, FK(channel_types.channel_type) | チャンネル種類ID |

## 制約

### プライマリキー
- `guild_id`, `channel_id`（複合キー）

### 外部キー
- `channel_type` → `channel_types(channel_type)`

### UNIQUE制約
なし

## インデックス
- PK: `guild_id`, `channel_id`（自動作成）
- FK: `channel_type`（外部キー制約で自動作成）

## データサンプル
| guild_id | channel_id | channel_type |
|----------|-----------|-------------|
| 123456789 | 987654321 | 1 |
| 123456789 | 987654322 | 2 |
| 987654321 | 123456789 | 1 |

## 関連テーブル
- **参照先**: `channel_types`（channel_typeで参照）

## 備考
- チャンネルの役割を定義し、Botの動作を制御
- channel_type=1: 募集チャンネル（マルチバトル募集を投稿可能）
- channel_type=2: 通知チャンネル（イベント通知を送信）
- channel_type=3: 管理チャンネル（Bot管理コマンド実行可能）
- 1つのチャンネルに対して1つの用途を設定
- ギルド管理者が設定可能

## Rust実装
- **エンティティファイル**: `src/models/entities/guild_master/guild_channels.rs`
- **マイグレーションファイル**: `migration/src/m*_create_guild_channels.rs`
- **実装状況**: ✅ 実装済み
