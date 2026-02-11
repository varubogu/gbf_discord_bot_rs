# 結合テスト個別設計書

## 目的

このディレクトリでは、機能別の結合テスト仕様を管理する。
共通ルールは `docs/develop/rules/testing/integration_test.md` を参照する。

## ファイル命名

- `{feature}.md`
- 例: `auto_recruitment.md`, `quest_recruitment.md`

## 設計書テンプレート

```markdown
# {機能名} 結合テスト設計

## 対象

- 対象Facade
- 関連Service/Repository

## ユースケース

- UC-1: ...
- UC-2: ...

## 前提データ

- 必須レコード
- 一意ID方針（guild_id/user_id など）

## テストケース

1. 正常系
2. 業務異常系
3. 前提欠如異常系
4. 冪等性/重複系

## クリーンアップ

- 前処理
- 後処理

## 実行方法

- ローカル実行コマンド
- CI実行条件
```

## 運用

- 新機能追加時は、対応する個別設計書を同時に追加する。
- テストケース追加・削除時は、個別設計書を同時更新する。
