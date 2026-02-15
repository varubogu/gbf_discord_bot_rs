# Development Rules (Must Read)

This section is where you “return to the rules when you’re unsure”.

## Top priorities (must follow)

- Comments/docs/error messages are written in Japanese (code identifiers are in English)
- Do not break the dependency direction: `events → facades → services → repository`
- Only facades define transaction boundaries
- Do not use `unwrap()` in production code

## Index

- [ワークフロー](workflow.md)
- [コーディング規約](coding_standards.md)
- [エラーハンドリング](error_handling.md)
- [ロギング](logging.md)
- [セキュリティ](security.md)
- [パフォーマンス](performance.md)
- [タイムゾーン](time_zones.md)
- [アンチパターン](anti_patterns.md)
