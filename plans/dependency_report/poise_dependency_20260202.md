# poise依存除去レポート（2026-02-02）

facades層以降の層でpoiseフレームワークを使用しない方針であるため、依存を排除中です。
基本方針としては、poiseを再現したGateway抽象化を導入し、event層でpoise依存を吸収します。
これによりfacade以降は純粋な依存のない処理だけになります。


## ⚠️ facades層に残っているpoise依存
以下のファイルでPoiseContextへの依存が残っています：
具体的な修正内容は他の処理を参考にしてください。

| --- | --- |
| ファイル | 依存内容 |
| new_recruit.rs	|  PoiseContext + ctx.serenity_context() |
| role_management.rs	| PoiseContext |
| guild_load_facade.rs	| PoiseContext |
| global_push_facade.rs	| PoiseContext |
| global_load_facade.rs	| PoiseContext |
| guild_push_facade.rs	| PoiseContext |
| environment.rs	| PoiseContext（一部コメントアウト） |

## 📋 ドキュメントとの差異
remaining_refactoring_tasks.mdに記載されていた以下のファイルは、すでにpoise依存が除去されています：

~~button_handler.rs~~ ✅
~~cancel.rs~~ ✅
~~participants.rs~~ ✅
~~change.rs~~ ✅
逆に、ドキュメントに記載されていない以下のファイルにpoise依存が残っています：

spreadsheet系の4ファイル
role_management.rs
environment.rs
