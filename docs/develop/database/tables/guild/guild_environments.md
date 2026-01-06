# ギルド環境変数（guild_environments）

## 概要

**テーブル物理名**: `guild_environments`
**テーブルタイプ**: Reference
**テーブルスコープ**: Guild

## 用途

ギルド固有のBot動作設定を管理します。グローバルのenvironmentsテーブルをギルド単位で上書き可能にし、ギルドごとのカスタマイズを実現します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| guild_id | BigInteger | PK, NOT NULL | ギルドID（Discord Guild ID） |
| key | String | PK, NOT NULL | 環境変数のキー |
| value | String | NULLABLE | 環境変数の値 |
| memo | String | NULLABLE | メモ（設定の説明） |

## 制約

### プライマリキー
- `guild_id`, `key`（複合キー）

### 外部キー
なし

### UNIQUE制約
なし

## インデックス
- PK: `guild_id`, `key`（自動作成）

## データサンプル
| guild_id | key | value | memo |
|----------|-----|-------|------|
| 123456789 | RECRUITMENT_EXPIRY_HOURS | 48 | 募集の有効期限（時間） |
| 123456789 | MAX_ACTIVE_RECRUITMENTS | 10 | 同時進行可能な募集数 |
| 987654321 | LOG_LEVEL | DEBUG | ログレベル設定 |

## 関連テーブル
- **関連**: `environments`（グローバル環境変数）

## 備考
- ギルド固有の設定としてグローバル設定を上書き
- データ参照時は guild_environments → environments の順で検索
- ギルド管理者（gbf_bot_controlロール）が設定可能
- keyはenvironmentsテーブルと同じキー名を使用

## Rust実装
- **エンティティ**: `src/models/entities/guild_environments.rs`
- **実装状況**: 未実装
