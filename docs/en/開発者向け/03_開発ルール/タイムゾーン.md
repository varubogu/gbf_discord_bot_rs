# Time Zones

Date/time inputs and notifications easily become complex due to time zones.

## Minimum rules

- For time zones: use UTC for computation and DB storage, and use the Discord server (guild) time zone for user input and display (convert in the events layer)
- Never leave the time zone of “user-entered date/time” ambiguous
- Logic depending on `now()` is hard to test; consider passing a reference time as an argument
