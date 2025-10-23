# 属性定義（elements）

## 概要

**テーブル物理名**: `elements`
**テーブルタイプ**: Reference
**テーブルスコープ**: All

## 用途

グラブルのゲーム内属性（火、水、土、風、光、闇）を定義します。イベントスケジュールの有利属性表示や、ゲーム要素の分類に使用されます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| element_id | Integer | PK, NOT NULL | 属性ID |
| stamp | String | NULLABLE | 属性スタンプ（絵文字、例: 🔥） |
| name_jp | String | NULLABLE | 属性名（日本語、例: 火） |
| name_en | String | NULLABLE | 属性名（英語、例: Fire） |

## 制約

### プライマリキー
- `element_id`

### 外部キー
なし

### UNIQUE制約
なし

## インデックス

- PK: `element_id`（自動作成）

## データサンプル

| element_id | stamp | name_jp | name_en |
|-----------|-------|---------|---------|
| 1 | 🔥 | 火 | Fire |
| 2 | 💧 | 水 | Water |
| 3 | 🌍 | 土 | Earth |
| 4 | 💨 | 風 | Wind |
| 5 | ⭐ | 光 | Light |
| 6 | 🌙 | 闇 | Dark |

## 関連テーブル

- **参照元**: `event_schedules`（weak_attributeで参照）
- **参照元**: `guild_event_schedules`（weak_attributeで参照）

## 備考

- グラブルの6属性を定義
- stampは Discord上での表示用絵文字
- 多言語対応のため日本語と英語の両方を保持

## Rust実装

- **エンティティ**: `src/models/entities/elements.rs`
- **実装状況**: 未実装
