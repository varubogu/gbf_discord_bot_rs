# クエスト別名（quest_aliases）

## 概要

**テーブル物理名**: `quest_aliases`
**テーブルタイプ**: Reference
**テーブルスコープ**: All

## 用途

クエストの別名・略称を定義し、ユーザーが様々な表記でクエストを指定できるようにします。検索機能や自動認識機能で使用されます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| target_id | Integer | PK, NOT NULL | クエストID |
| target_alias_id | Integer | PK, NOT NULL | 別名連番ID |
| alias | String | UNIQUE, NOT NULL | クエスト別名（例: アルバハHL、プロバハ） |
| alias_kana_small | String | NULLABLE | クエスト別名（半角カナ）検索用 |

## 制約

### プライマリキー
- `target_id`, `target_alias_id`（複合キー）

### 外部キー
- `target_id` → `quests(target_id)`

### UNIQUE制約
- UNIQUE(`alias`) - 制約名: unique_quest_alias

## インデックス

- PK: `target_id`, `target_alias_id`（自動作成）
- UNIQUE: `alias`（自動作成）
- FK: `target_id`（外部キー制約で自動作成）

## データサンプル

| target_id | target_alias_id | alias | alias_kana_small |
|-----------|----------------|-------|------------------|
| 1 | 1 | プロトバハムート | ﾌﾟﾛﾄﾊﾞﾊﾑｰﾄ |
| 1 | 2 | プロバハ | ﾌﾟﾛﾊﾞﾊ |
| 2 | 1 | アルティメットバハムートHL | ｱﾙﾃｨﾒｯﾄﾊﾞﾊﾑｰﾄHL |
| 2 | 2 | アルバハHL | ｱﾙﾊﾞﾊHL |

## 関連テーブル

- **参照先**: `quests`（target_idで参照）

## 備考

- 1つのクエストに対して複数の別名を登録可能
- alias_kana_smallは検索時のカナ表記マッチング用
- 別名はシステム全体で一意でなければならない

## Rust実装

- **エンティティ**: `src/models/entities/quest_aliases.rs`
- **実装状況**: 実装済み
