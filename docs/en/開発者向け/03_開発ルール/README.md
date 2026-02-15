# Development Rules (Must Read)

This section is where you “return to the rules when you’re unsure”.

## Top priorities (must follow)

- Comments/docs/error messages are written in Japanese (code identifiers are in English)
- Do not break the dependency direction: `events → facades → services → repository`
- Only facades define transaction boundaries
- Do not use `unwrap()` in production code

## Index

- [ワークフロー](ワークフロー.md)
- [コーディング規約](コーディング規約.md)
- [エラーハンドリング](エラーハンドリング.md)
- [ロギング](ロギング.md)
- [セキュリティ](セキュリティ.md)
- [パフォーマンス](パフォーマンス.md)
- [タイムゾーン](タイムゾーン.md)
- [アンチパターン](アンチパターン.md)
