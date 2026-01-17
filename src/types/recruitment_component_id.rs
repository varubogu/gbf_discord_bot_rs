use crate::types::AppError;

/// 募集ボタンのCustom ID
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecruitmentComponentId {
    /// シンプル参加ボタン（`recruit_join`）
    Join,
    /// 属性参加ボタン（`recruit_join_{element_id}`）
    /// element_id: 1=火, 2=水, 3=土, 4=風, 5=光, 6=闇
    JoinElement(i32),
    /// 全属性可能ボタン（`recruit_join_0`）
    JoinAllElements,
    /// すべて取り消しボタン（`recruit_leave_all`）
    LeaveAll,
    /// 属性セレクトメニュー（`recruit_select_elements`）
    SelectElements,
    /// セレクトメニューからの参加ボタン（`recruit_join_selected`）
    JoinSelected,
}

impl RecruitmentComponentId {
    /// Custom IDをパースしてRecruitmentComponentIdに変換
    ///
    /// # 引数
    /// * `custom_id` - パース対象のCustom ID文字列
    ///
    /// # 戻り値
    /// * `Ok(RecruitmentComponentId)` - パース成功
    /// * `Err(AppError)` - パース失敗
    ///
    /// # 例
    /// ```
    /// use gbf_discord_bot_rs::types::RecruitmentComponentId;
    ///
    /// let id = RecruitmentComponentId::parse("recruit_join").unwrap();
    /// assert_eq!(id, RecruitmentComponentId::Join);
    ///
    /// let id = RecruitmentComponentId::parse("recruit_join_1").unwrap();
    /// assert_eq!(id, RecruitmentComponentId::JoinElement(1));
    ///
    /// let id = RecruitmentComponentId::parse("recruit_join_0").unwrap();
    /// assert_eq!(id, RecruitmentComponentId::JoinAllElements);
    ///
    /// let id = RecruitmentComponentId::parse("recruit_leave_all").unwrap();
    /// assert_eq!(id, RecruitmentComponentId::LeaveAll);
    /// ```
    pub fn parse(custom_id: &str) -> Result<Self, AppError> {
        match custom_id {
            "recruit_join" => Ok(Self::Join),
            "recruit_leave_all" => Ok(Self::LeaveAll),
            "recruit_select_elements" => Ok(Self::SelectElements),
            "recruit_join_selected" => Ok(Self::JoinSelected),
            s if s.starts_with("recruit_join_") => {
                let element_id_str = s
                    .strip_prefix("recruit_join_")
                    .ok_or_else(|| AppError::Generic(format!("無効なCustom ID形式: {s}")))?;

                let element_id: i32 = element_id_str.parse().map_err(|_| {
                    AppError::Generic(format!("属性IDが数値ではありません: {element_id_str}"))
                })?;

                // element_id = 0 は全属性可能
                if element_id == 0 {
                    Ok(Self::JoinAllElements)
                } else if (1..=6).contains(&element_id) {
                    Ok(Self::JoinElement(element_id))
                } else {
                    Err(AppError::Generic(format!(
                        "無効な属性ID: {element_id}（1-6の範囲で指定してください）"
                    )))
                }
            }
            _ => Err(AppError::Generic(format!("未知のCustom ID: {custom_id}"))),
        }
    }

    /// RecruitmentComponentIdをCustom ID文字列に変換
    ///
    /// # 戻り値
    /// Custom ID文字列
    ///
    /// # 例
    /// ```
    /// use gbf_discord_bot_rs::types::RecruitmentComponentId;
    ///
    /// assert_eq!(RecruitmentComponentId::Join.to_custom_id(), "recruit_join");
    /// assert_eq!(RecruitmentComponentId::JoinElement(1).to_custom_id(), "recruit_join_1");
    /// assert_eq!(RecruitmentComponentId::JoinAllElements.to_custom_id(), "recruit_join_0");
    /// assert_eq!(RecruitmentComponentId::LeaveAll.to_custom_id(), "recruit_leave_all");
    /// ```
    pub fn to_custom_id(&self) -> String {
        match self {
            Self::Join => "recruit_join".to_string(),
            Self::JoinElement(element_id) => format!("recruit_join_{element_id}"),
            Self::JoinAllElements => "recruit_join_0".to_string(),
            Self::LeaveAll => "recruit_leave_all".to_string(),
            Self::SelectElements => "recruit_select_elements".to_string(),
            Self::JoinSelected => "recruit_join_selected".to_string(),
        }
    }

    /// 募集参加かどうかを判定
    ///
    /// # 戻り値
    /// * `true` - 参加ボタン
    /// * `false` - 退出ボタン
    pub fn is_join(&self) -> bool {
        matches!(
            self,
            Self::Join | Self::JoinElement(_) | Self::JoinAllElements | Self::JoinSelected
        )
    }

    /// 属性IDを取得（参加ボタンの場合）
    ///
    /// # 戻り値
    /// * `Some(element_id)` - 属性参加ボタンまたは全属性可能ボタンの場合
    /// * `None` - シンプル参加ボタンまたは退出ボタンの場合
    pub fn element_id(&self) -> Option<i32> {
        match self {
            Self::JoinElement(id) => Some(*id),
            Self::JoinAllElements => Some(0),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_join() {
        let id = RecruitmentComponentId::parse("recruit_join").unwrap();
        assert_eq!(id, RecruitmentComponentId::Join);
        assert!(id.is_join());
        assert_eq!(id.element_id(), None);
    }

    #[test]
    fn test_parse_join_element() {
        for element_id in 1..=6 {
            let custom_id = format!("recruit_join_{element_id}");
            let id = RecruitmentComponentId::parse(&custom_id).unwrap();
            assert_eq!(id, RecruitmentComponentId::JoinElement(element_id));
            assert!(id.is_join());
            assert_eq!(id.element_id(), Some(element_id));
        }
    }

    #[test]
    fn test_parse_join_all_elements() {
        let id = RecruitmentComponentId::parse("recruit_join_0").unwrap();
        assert_eq!(id, RecruitmentComponentId::JoinAllElements);
        assert!(id.is_join());
        assert_eq!(id.element_id(), Some(0));
    }

    #[test]
    fn test_parse_leave_all() {
        let id = RecruitmentComponentId::parse("recruit_leave_all").unwrap();
        assert_eq!(id, RecruitmentComponentId::LeaveAll);
        assert!(!id.is_join());
        assert_eq!(id.element_id(), None);
    }

    #[test]
    fn test_parse_invalid_element_id() {
        let result = RecruitmentComponentId::parse("recruit_join_7");
        assert!(result.is_err());

        let result = RecruitmentComponentId::parse("recruit_join_-1");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unknown_custom_id() {
        let result = RecruitmentComponentId::parse("unknown_id");
        assert!(result.is_err());
    }

    #[test]
    fn test_to_custom_id() {
        assert_eq!(RecruitmentComponentId::Join.to_custom_id(), "recruit_join");
        assert_eq!(
            RecruitmentComponentId::JoinElement(1).to_custom_id(),
            "recruit_join_1"
        );
        assert_eq!(
            RecruitmentComponentId::JoinAllElements.to_custom_id(),
            "recruit_join_0"
        );
        assert_eq!(
            RecruitmentComponentId::LeaveAll.to_custom_id(),
            "recruit_leave_all"
        );
    }

    #[test]
    fn test_round_trip() {
        let test_cases = vec![
            "recruit_join",
            "recruit_join_1",
            "recruit_join_2",
            "recruit_join_3",
            "recruit_join_4",
            "recruit_join_5",
            "recruit_join_6",
            "recruit_join_0",
            "recruit_leave_all",
        ];

        for custom_id in test_cases {
            let parsed = RecruitmentComponentId::parse(custom_id).unwrap();
            let reconstructed = parsed.to_custom_id();
            assert_eq!(custom_id, reconstructed);
        }
    }
}
