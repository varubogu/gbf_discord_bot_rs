# Co-op Recruitment (User)

This feature helps you gather participants for a co-op (multi) battle.

![マルチバトル募集コマンド](image-4.png)

## What you can do

- Create a recruitment post by specifying a quest and time
- Participants can join via buttons or reactions (appearance depends on server settings)

## Try it (example)

1. Type `/` in the chat box
2. Choose `/マルチバトル募集` (reaction style) or `/マルチバトル募集2` (button style)
3. Enter the quest name and time, then send

<a id="quest-datetime-input"></a>

## Tips for entering quest date/time

- Date
  - ISO format like `2024-12-31`
  - Slash format like `12/31`
  - Japanese formats like `12月31日`
  - Relative words in Japanese like `今日` (today) / `明日` (tomorrow)
  - Relative words in English like `today` / `tomorrow`
- Time
  - 24-hour time with colon like `21:00`
  - 4-digit number interpreted as time (e.g., `2200` → `22:00`)
  - Japanese formats like `21時`, `21時30分`, `21時半`

### Examples

- `2024-12-31 21:00`
- `12/31 21:00`
- `12月31日 21時`
- `今日 21:00`
- `明日 21時半`
- `tomorrow 2200`

<a id="what-should-i-enter"></a>

## What should I enter?

### `/マルチバトル募集`（リアクション版） `/マルチバトル募集2`（ボタン版）

| Field | Required | Description |
|---|---:|---|
| Quest | ✅ | The quest to recruit for (select from suggestions) |
| Departure date/time | ✅ | When you will depart |
| Strategy | Optional | If omitted, the quest default is used |
| Dismissal time(s) | Optional | Up to 3 values, comma-separated (e.g., `1時間前, 21:00`) |

## Cancel a recruitment

1. Right-click the recruitment message (long-press on mobile)
2. Select “Apps” → “募集キャンセル”

Or:

1. Right-click the recruitment message (long-press on mobile)
2. Select “Copy Message Link”
3. Run `/recruit_cancel` or `/募集キャンセル` and paste the message link

## Change a recruitment (edit details)

1. Right-click the recruitment message (long-press on mobile)
2. Select “Apps” → “募集内容変更”
3. In the form, fill only the fields you want to change and submit
  - Quest name (optional)
  - Departure date/time (optional)
  - Strategy (optional: ID or display name)
4. Fields left empty are not changed

Or:

1. Right-click the recruitment message (long-press on mobile)
2. Select “Copy Message Link”
3. Run `/recruit_change` or `/マルチバトル募集内容変更` and paste the message link
