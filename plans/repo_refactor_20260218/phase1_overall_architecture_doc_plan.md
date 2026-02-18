# フェーズ1計画: 全体設計書修正

## ゴール
Repository再編の前に、設計書上で「責務境界・配置規約・命名規約」を確定し、以降のPR判断をブレさせない。

## 合意する設計原則
1. `repository` はtrait（port）と入出力型のみを保持する。
2. SeaORM実装は `infrastructure/database/repositories` に集約する。
3. `services` はtraitにのみ依存し、具象実装を知らない。
4. `facade` はトランザクション境界のみ担当し、Repository具象型を直接生成しない。
5. 互換用re-exportを導入せず、旧importは段階的に新importへ置換して削除する。

## 設計書修正対象
- `docs/en/developer/02_architecture/project_structure.md`
  - 実ディレクトリと設計意図を一致させる
  - `repository` と `infrastructure/database/repositories` の責務を明記
- `docs/en/developer/02_architecture/layered_architecture.md`
  - repository = port、infrastructure = adapter を明記
- `docs/en/developer/02_architecture/dependency_injection.md`
  - DI配線点（`di/repositories.rs`）でのみ具象型を扱うと明記
- `docs/en/developer/05_database/connections_and_transactions.md`
  - セッション変数設定（RLS）の責務を `infrastructure/database/session` に移す旨を明記
- `docs/en/developer/06_feature_specifications/*.md`
  - `src/repository/database/...` の参照パスを新構成に更新

## 成果物
- 設計書更新PR（コード変更なし）
- 旧importを残さない移行方針の明文化

## レビュー観点
- 用語統一: repository(port) / infrastructure(adapter)
- 依存方向が `events -> facades -> services -> repository` のまま維持されているか
- 既存実装との矛盾がないか（実装先行ではなく設計先行）
- 旧パス温存を前提にした記述が残っていないか

## 完了条件
- 対象ドキュメントの更新完了
- PR02以降がこの文書のみで着手可能な粒度になっている
- feature spec内の実装パス記述が旧パス依存になっていない
- 互換re-export前提の記述が計画書から除去されている
