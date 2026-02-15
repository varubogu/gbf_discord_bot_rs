# Quick Start (Shortest Path)

This page summarizes the minimum setup to “make the bot usable in your Discord server”.

## Steps

1. `gbf_bot_control` ロールを作る（Bot設定を触れる人にだけ付ける）
2. サーバー用スプレッドシートを用意し、サービスアカウントに共有する
3. スプレッドシートを登録する
4. チャンネルを登録する
5. スプレッドシートを読み込む（反映）

## 1) Create the `gbf_bot_control` role

- Use the role name `gbf_bot_control` (recommended for clarity).
- Grant it to as few people as possible to prevent mistakes.

## 2) Prepare and share the server spreadsheet

- To allow the bot to access the spreadsheet, share it with the bot’s **service account**.
- If you don’t know the share target (email address), ask the bot operator.

### Recommended sharing settings

- Permission: **Editor** (in most cases).
- Share at the spreadsheet (file) level.

## 3) Register the spreadsheet

- Tell the bot “this server uses this spreadsheet”.
- You can input either a URL or an ID (choose what fits your operation).

## 4) Register channels

- Register the channels where the bot should send messages (notifications, recruitment posts, etc.).
- Ensure the bot has permission to view and post in those channels.

## 5) Load the spreadsheet (apply)

- Apply the registered spreadsheet contents to the DB.
- Run this during initial setup and after changing settings.

## Notes (avoid accidents)

- `gbf_bot_control` is a powerful permission; keep the assignee set minimal.
- After changing settings, don’t forget to “load the spreadsheet” (changes won’t apply otherwise).
