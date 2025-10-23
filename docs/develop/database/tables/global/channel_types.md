# チャンネル種類（channel_types）

## 概要

**テーブル物理名**: `channel_types`
**テーブルタイプ**: Reference
**テーブルスコープ**: All

## 用途

Discordチャンネルの用途分類を定義します。募集チャンネル、通知チャンネルなど、チャンネルの役割を管理します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| channel_type | Integer | PK, NOT NULL | チャンネル種類ID |
| channel_type_name | String | NOT NULL | チャンネル種類名（例: 募集チャンネル、通知チャンネル） |
| memo | String | NULLABLE | メモ |

## 制約

### プライマリキー
- `channel_type`

### 外部キー
なし

### UNIQUE制約
なし

## インデックス

- PK: `channel_type`（自動作成）

## データサンプル

| channel_type | channel_type_name | memo |
|-------------|------------------|------|
| 1 | 募集チャンネル | マルチバトル募集用 |
| 2 | 通知チャンネル | イベント通知用 |
| 3 | 管理チャンネル | Bot管理用 |

## 関連テーブル

- **参照元**: `guild_channels`（channel_typeで参照）

## 備考

- チャンネルの役割を定義し、Botの動作を制御
- guild_channelsテーブルで具体的なチャンネルと紐づけ

## Rust実装

- **エンティティ**: `src/models/entities/channel_types.rs`
- **実装状況**: 未実装
