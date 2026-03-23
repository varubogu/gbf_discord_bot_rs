# Start and Update (Docker Compose)

This page describes how a bot operator (infra/deploy) can “start the bot and update it without downtime”.

## Prerequisites (minimum)

- An environment where Docker works
- This repository checkout
- Service account key (JSON) placed under `.local/`
- `.env.app` / `.env.db` / `.env.maintenance` configured

## Start (production baseline)

```bash
./mng.sh prod up
```

## Which image is used?

This project typically starts by pulling prebuilt images from a registry (GHCR).
The source is determined by environment variables.

- `GITHUB_REPOSITORY`: in `owner/repo` format (e.g., `varubogu/gbf_discord_bot_rs`)

Example:

```bash
export GITHUB_REPOSITORY=owner/repo
```

## Stop

```bash
./mng.sh prod down
```

## Update only the bot (minimize downtime)

```bash
./mng.sh prod update_app
```

## Run maintenance (cleanup)

```bash
docker compose run --rm maintenance
```

## Common pitfalls

- `GITHUB_REPOSITORY` is not set, so the pull source is not what you intended
- Missing `.env.app` / `.env.db` / `.env.maintenance` files or empty values (especially tokens and DB passwords)
- `.local/` does not exist, or the key JSON is not present
