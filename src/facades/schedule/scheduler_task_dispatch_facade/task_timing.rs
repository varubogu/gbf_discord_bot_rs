use chrono::Utc;

/// タスクの実行時刻に達しているか判定する
pub(super) fn is_task_due(
    task_schedule_datetime: chrono::DateTime<Utc>,
    now: chrono::DateTime<Utc>,
) -> bool {
    task_schedule_datetime <= now
}

/// 出発時刻を過ぎているため募集作成をスキップすべきか判定する
pub(super) fn should_skip_recruitment_creation(
    quest_start_at: chrono::DateTime<Utc>,
    now: chrono::DateTime<Utc>,
) -> bool {
    quest_start_at <= now
}

#[cfg(test)]
mod tests {
    use super::{is_task_due, should_skip_recruitment_creation};
    use chrono::{Duration, TimeZone, Utc};

    #[test]
    fn is_due_when_schedule_is_past_or_now() {
        let now = Utc::now();
        assert!(is_task_due(now - Duration::seconds(1), now));
        assert!(is_task_due(now, now));
    }

    #[test]
    fn is_not_due_when_schedule_is_future() {
        let now = Utc::now();
        assert!(!is_task_due(now + Duration::seconds(1), now));
    }

    #[test]
    fn should_skip_recruitment_creation_when_quest_start_equals_now() {
        let now = Utc.with_ymd_and_hms(2026, 3, 3, 12, 0, 0).single().unwrap();
        assert!(should_skip_recruitment_creation(now, now));
    }

    #[test]
    fn should_skip_recruitment_creation_when_quest_start_is_past() {
        let now = Utc.with_ymd_and_hms(2026, 3, 3, 12, 0, 0).single().unwrap();
        let quest_start_at = now - Duration::seconds(1);
        assert!(should_skip_recruitment_creation(quest_start_at, now));
    }

    #[test]
    fn should_not_skip_recruitment_creation_when_quest_start_is_future() {
        let now = Utc.with_ymd_and_hms(2026, 3, 3, 12, 0, 0).single().unwrap();
        let quest_start_at = now + Duration::seconds(1);
        assert!(!should_skip_recruitment_creation(quest_start_at, now));
    }
}
