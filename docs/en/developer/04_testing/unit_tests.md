# Unit Tests

## Scope

- 1つの関数/サービス/変換処理など、「その場で完結できる」ロジック
- 例: 入力検証、日時解釈、メッセージ生成、条件分岐

## How to write

- AAA（Arrange–Act–Assert）
- 成功/失敗の両方を最低限入れる（境界値や例外系）

## External I/O

- Discord/DB/スプレッドシートなどの外部I/Oはモック化する
- 依存はTraitで抽象化し、`mockall` を使って差し替える

## Location

- 原則は対象実装ファイル内の `#[cfg(test)] mod tests { ... }`
