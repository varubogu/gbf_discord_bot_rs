# Scheduled Recruitment (User)

This feature automatically posts recruitments on a schedule (e.g., “every week on these days at this time”).

## What you can do

- Create, pause, resume, and delete scheduled recruitments
- View the list of scheduled recruitments

## What to remember (minimum)

- Once you set “target weekdays” and “post time”, the bot posts automatically.
- You can pause or delete scheduled recruitments that you created.

## Commands

### 1) Create: `/recruitment-schedule-create`

| Field | Required | Description |
|---|---:|---|
| Schedule name | ✅ | A readable name (e.g., `Ultimate-Baha 21:00`) |
| Quest | ✅ | The quest to recruit for |
| Quest start time | ✅ | `HH:MM` (e.g., `21:00`) |
| Target weekdays | ✅ | e.g., `Mon,Wed,Fri` / `MonTueWed` / `Daily` |
| Post start time | ✅ | e.g., `19:00` / `2h before` |
| Strategy | Optional | Override the quest’s default strategy |
| Start date offset | Optional | 0=today, 1=previous day, 2=two days before (auto if omitted) |
| Notes | Optional | Memo |
| Dismissal time(s) | Optional | Up to 3 values, comma-separated |

Note: Quest, quest start time, strategy, and dismissal time(s) use the same input formats as co-op recruitment. See:

- [Co-op recruitment: what to enter](./02_multi_recruitment.md#what-should-i-enter)
- [Co-op recruitment: quest date/time input tips](./02_multi_recruitment.md#quest-datetime-input)

### 2) List: `/recruitment-schedule-list`

- Shows the scheduled recruitments currently registered

### 3) Delete: `/recruitment-schedule-delete`

- Use this when you want to delete it permanently (assumed irreversible)

### 4) Pause/resume: `/recruitment-schedule-toggle`

If you only want to stop it temporarily, pausing is recommended instead of deleting.
You can resume it later.
