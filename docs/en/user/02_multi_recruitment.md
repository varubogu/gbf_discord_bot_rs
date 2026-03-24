# Co-op Recruitment (User)

This feature helps you gather participants for a co-op (multi) battle.

![Co-op recruitment command](image-4.png)

## What you can do

- Create a recruitment post by specifying a quest and time
- Participants can join via buttons or reactions (appearance depends on server settings)

## Try it (example)

1. Type `/` in the chat box
2. Choose `/recruit_new` (reaction style) or `/recruit_new_v2` (button style)
3. Enter the quest name and time, then send

<a id="quest-datetime-input"></a>

## Tips for entering quest date/time

- Date
  - ISO format like `2024-12-31`
  - Slash format like `12/31`
  - Relative words like `today` / `tomorrow`
- Time
  - 24-hour time with colon like `21:00`
  - 4-digit number interpreted as time (e.g., `2200` → `22:00`)
  - 12-hour format with AM/PM like `9 PM`, `9:30 PM`

### Examples

- `2024-12-31 21:00`
- `12/31 21:00`
- `today 21:00`
- `tomorrow 21:30`
- `tomorrow 2200`

<a id="what-should-i-enter"></a>

## What should I enter?

### `/recruit_new` (reaction version) and `/recruit_new_v2` (button version)

| Field | Required | Description |
|---|---:|---|
| Quest | ✅ | The quest to recruit for (select from suggestions) |
| Departure date/time | ✅ | When you will depart |
| Strategy | Optional | If omitted, the quest default is used |
| Dismissal time(s) | Optional | Up to 3 values, comma-separated (e.g., `1h before, 21:00`) |

## Cancel a recruitment

1. Right-click the recruitment message (long-press on mobile)
2. Select “Apps” → “Recruit Cancel”

Or:

1. Right-click the recruitment message (long-press on mobile)
2. Select “Copy Message Link”
3. Run `/recruit_cancel` and paste the message link

## Change a recruitment (edit details)

1. Right-click the recruitment message (long-press on mobile)
2. Select “Apps” → “Recruit Change”
3. In the menu, choose/input only what you want to change
  - Quest: choose from the select menu
  - Strategy: choose from the select menu
  - Departure date/time: use the “Enter Departure Datetime” button
4. Press `Apply` to apply the changes

Or:

1. Right-click the recruitment message (long-press on mobile)
2. Select “Copy Message Link”
3. Run `/recruit_change` and paste the message link
