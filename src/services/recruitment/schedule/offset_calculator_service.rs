use chrono::NaiveTime;

/// 募集開始日オフセット計算サービス
///
/// 定期募集スケジュール作成時の募集開始日オフセットのデフォルト値を決定します。
pub struct OffsetCalculatorService;

impl OffsetCalculatorService {
    pub fn new() -> Self {
        Self
    }

    /// 募集開始日オフセットのデフォルト値を決定
    ///
    /// # 引数
    /// - `recruit_start_time`: 募集開始時刻
    /// - `quest_start_time`: クエスト開始時刻
    ///
    /// # 戻り値
    /// - 0: 当日募集（募集開始時刻 < クエスト開始時刻）
    /// - 1: 前日募集（募集開始時刻 ≧ クエスト開始時刻）
    ///
    /// # ロジック
    /// 時刻部分のみを比較してオフセット値を決定します。
    /// - 募集開始時刻がクエスト開始時刻より早い場合、当日の早い時刻に募集を開始し、
    ///   遅い時刻にクエストを開始する自然な流れが成立するため、オフセット0（当日募集）とします。
    /// - 募集開始時刻がクエスト開始時刻と同じかそれより遅い場合、当日だと募集開始が
    ///   クエスト開始と同時または後になってしまうため、オフセット1（前日募集）とします。
    pub fn determine_default_offset(
        recruit_start_time: NaiveTime,
        quest_start_time: NaiveTime,
    ) -> i32 {
        if recruit_start_time < quest_start_time {
            0 // 当日募集
        } else {
            1 // 前日募集
        }
    }
}

impl Default for OffsetCalculatorService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_default_offset_same_day() {
        // 募集開始時刻 < クエスト開始時刻 → オフセット0（当日募集）
        let recruit_time = NaiveTime::from_hms_opt(20, 0, 0).unwrap();
        let quest_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        assert_eq!(
            OffsetCalculatorService::determine_default_offset(recruit_time, quest_time),
            0
        );
    }

    #[test]
    fn test_determine_default_offset_previous_day() {
        // 募集開始時刻 > クエスト開始時刻 → オフセット1（前日募集）
        let recruit_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        let quest_time = NaiveTime::from_hms_opt(20, 0, 0).unwrap();
        assert_eq!(
            OffsetCalculatorService::determine_default_offset(recruit_time, quest_time),
            1
        );
    }

    #[test]
    fn test_determine_default_offset_equal_time() {
        // 募集開始時刻 == クエスト開始時刻 → オフセット1（前日募集）
        let recruit_time = NaiveTime::from_hms_opt(21, 0, 0).unwrap();
        let quest_time = NaiveTime::from_hms_opt(21, 0, 0).unwrap();
        assert_eq!(
            OffsetCalculatorService::determine_default_offset(recruit_time, quest_time),
            1
        );
    }

    #[test]
    fn test_determine_default_offset_edge_case_midnight() {
        // 募集開始時刻 23:59、クエスト開始時刻 00:00 → オフセット1（前日募集）
        let recruit_time = NaiveTime::from_hms_opt(23, 59, 0).unwrap();
        let quest_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        assert_eq!(
            OffsetCalculatorService::determine_default_offset(recruit_time, quest_time),
            1
        );
    }

    #[test]
    fn test_determine_default_offset_edge_case_early_morning() {
        // 募集開始時刻 00:00、クエスト開始時刻 05:00 → オフセット0（当日募集）
        let recruit_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let quest_time = NaiveTime::from_hms_opt(5, 0, 0).unwrap();
        assert_eq!(
            OffsetCalculatorService::determine_default_offset(recruit_time, quest_time),
            0
        );
    }
}
