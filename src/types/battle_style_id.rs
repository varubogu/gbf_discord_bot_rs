/// マルチバトル戦術ID
///
/// `master.battle_styles`テーブルのIDに対応する列挙型。
/// クエストの`default_battle_style_id`で使用する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum BattleStyleId {
    /// 6属性討伐クエスト（火/水/土/風/光/闇の各属性1人ずつ必要）
    SixElements = 1,
}

impl BattleStyleId {
    /// i32からBattleStyleIdに変換
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::SixElements),
            _ => None,
        }
    }

    /// BattleStyleIdをi32に変換
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// 6属性クエストかどうかを判定
    pub fn is_six_elements(battle_style_id: i32) -> bool {
        battle_style_id == Self::SixElements.as_i32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_six_elements_id() {
        assert_eq!(BattleStyleId::SixElements.as_i32(), 1);
    }

    #[test]
    fn test_from_i32() {
        assert_eq!(BattleStyleId::from_i32(1), Some(BattleStyleId::SixElements));
        assert_eq!(BattleStyleId::from_i32(0), None);
        assert_eq!(BattleStyleId::from_i32(999), None);
    }

    #[test]
    fn test_is_six_elements() {
        assert!(BattleStyleId::is_six_elements(1));
        assert!(!BattleStyleId::is_six_elements(0));
        assert!(!BattleStyleId::is_six_elements(2));
    }
}
