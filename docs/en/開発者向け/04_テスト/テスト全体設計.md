# Overall Test Design

## Goals

- 変更が壊していないことを素早く確認する
- バグを「再現手順ごと」固定化して、再発を防ぐ
- レイヤー責務（特にトランザクション境界）を崩さない

## Test types

### Unit tests

- 速い、失敗箇所が分かりやすい
- 外部I/O（Discord/DB/スプレッドシート）は原則モック化する

### Integration tests

- Facade起点で、複数レイヤーの整合性を確認する
- 外部I/O（Discord/DB/スプレッドシート）は本番と同様のものをテスト環境として作成し、そのまま利用する

### System/E2E tests

- 自動テストの対象外とする（Discordは操作の自動化が規約上難しいため）

## Common rules

- AAA（Arrange–Act–Assert）の順で書く
- テスト間で状態を共有しない（順序に依存しない）
- 「業務上意味のある結果」を確認する（実装の詳細に寄せすぎない）
