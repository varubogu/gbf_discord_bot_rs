# マルチバトル戦術（battle_types）

## 概要

**テーブル物理名**: `battle_types`
**テーブルタイプ**: Reference
**テーブルスコープ**: All

## 用途

グラブルのマルチバトルにおける戦術タイプ（青箱優先、トレハン優先など）を定義します。Discordリアクション（絵文字）とバトルタイプを紐づけ、ユーザーがリアクションで戦術を選択できるようにします。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| type_id | Integer | PK, NOT NULL | 戦術ID |
| name | String | NOT NULL | 戦術名（例: 青箱優先、トレハン優先） |
| reactions | String | NULLABLE | 戦術に応じたリアクション（絵文字） |

## 制約

### プライマリキー
- `type_id`

### 外部キー
なし

### UNIQUE制約
なし

## インデックス

- PK: `type_id`（自動作成）

## データサンプル

| type_id | name | reactions |
|---------|------|-----------|
| 1 | 青箱優先 | 🔵 |
| 2 | トレハン優先 | 💎 |
| 3 | 速攻 | ⚡ |

## 関連テーブル

- **参照元**: `quests`（use_battle_type、default_battle_typeで参照）
- **参照元**: `battle_recruitments`（battle_type_idで参照）

## 備考

- マルチバトル募集時に、ユーザーが選択可能な戦術を定義
- Discordリアクションとの紐づけにより、直感的な戦術選択が可能

## Rust実装

- **エンティティ**: `src/models/entities/battle_types.rs`
- **実装状況**: 実装済み
