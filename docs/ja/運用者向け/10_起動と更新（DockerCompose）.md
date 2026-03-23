# 起動と更新（Docker Compose）

この章は、Bot運用者（インフラ/デプロイ担当）が「Botを起動し、止めずに更新する」ための手順です。

## 前提（最小）

- Docker が使える環境
- このリポジトリ一式
- `.local/` にサービスアカウント鍵（JSON）が置かれている
- `.env.app` / `.env.db` / `.env.maintenance` が設定されている

## 起動（本番の基本）

```bash
./mng.sh prod up
```

## どのイメージを使う？

このプロジェクトは、基本的にビルド済みイメージをレジストリ（GHCR）から取得して起動します。
取得先は環境変数で決まります。

- `GITHUB_REPOSITORY`: `owner/repo` 形式（例: `varubogu/gbf_discord_bot_rs`）

設定例:

```bash
export GITHUB_REPOSITORY=owner/repo
```

## 停止

```bash
./mng.sh prod down
```

## Botだけ更新（停止時間を短くしたいとき）

```bash
./mng.sh prod update_app
```

## メンテナンス実行（クリーンアップ）

```bash
docker compose run --rm maintenance
```

## よくあるつまずき

- `GITHUB_REPOSITORY` が未設定で、pull先が意図したものにならない
- `.env.app` / `.env.db` / `.env.maintenance` の不足/空欄（特にトークン、DBパスワード）
- `.local/` が存在しない、または鍵JSONが入っていない
