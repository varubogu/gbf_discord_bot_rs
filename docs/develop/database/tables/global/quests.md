# クエスト情報（quests）

## 概要

**テーブル物理名**: `quests`
**テーブルタイプ**: Reference
**テーブルスコープ**: All

## 用途

グラブルのクエスト定義を管理します。クエスト名、募集人数、使用可能な戦術、デフォルト戦術などを定義し、マルチバトル募集の基準となる情報を提供します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| target_id | Integer | PK, NOT NULL | クエストID |
| recruit_count | Integer | NULLABLE | 募集人数（マルチバトルの参加可能人数） |
| quest_name | String | NOT NULL | クエスト名（例: プロトバハムート、アルバハHL） |
| use_battle_type | String | NULLABLE | 使用可能な戦術ID（カンマ区切り、例: "1,2,3"） |
| default_battle_type | String | NULLABLE | デフォルトの戦術ID |

## 制約

### プライマリキー
- `target_id`

### 外部キー
なし（use_battle_type、default_battle_typeは文字列形式でbattle_typesを参照）

### UNIQUE制約
なし

## インデックス

- PK: `target_id`（自動作成）

## データサンプル

| target_id | recruit_count | quest_name | use_battle_type | default_battle_type |
|-----------|--------------|-----------|-----------------|---------------------|
| 1 | 30 | プロトバハムート | 1,2,3 | 1 |
| 2 | 18 | アルティメットバハムートHL | 1,2 | 1 |
| 3 | 6 | ルシファーHL | 1 | 1 |

## 関連テーブル

- **参照元**: `quest_aliases`（target_idで参照）
- **参照元**: `battle_recruitments`（target_idで参照）
- **参照先**: `battle_types`（use_battle_type、default_battle_typeで論理的に参照）

## 備考

- マルチバトル募集の基準となるクエスト情報を定義
- use_battle_typeとdefault_battle_typeは文字列としてbattle_typesのIDを保持
- recruit_countはマルチバトルの最大参加人数を示す

## Rust実装

- **エンティティ**: `src/models/entities/quests.rs`
- **実装状況**: 実装済み
