# Discord Constraints (Developer)

Discord is convenient, but it has constraints that affect bot implementation.
This page is a memo that focuses on the issues most likely to cause real-world problems.

## Important: respond to interactions quickly

Interactions (slash commands, etc.) will fail if you don’t respond quickly.

- If the work is heavy, first acknowledge it (defer)
- Then return the result (edit / follow-up)

## Rate limits

The Discord API has rate limits.
In many cases, poise should handle waiting/retrying automatically.

- Don’t mass-send or mass-edit in tight loops
- Don’t repeatedly update the same content
- Don’t spam immediate retries on failure (add backoff)

## Designing message updates

- Prefer editing existing messages over posting new ones (keeps channels cleaner)
- But frequent edits are more likely to hit rate limits

## Components (buttons, etc.)

- Buttons/select menus are useful, but they can become unmanageable without proper state design
- Prefer an idempotent design that stores “what should happen on press” in the DB

## Security

- Always validate target channel, executor permissions, target guild ID, etc.
- Do not trust external input (user input); validate before use
