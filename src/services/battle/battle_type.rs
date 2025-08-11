use serenity::all::ReactionType;

// Battle types enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleType {
    Default = 0,
    AllElement = 1,
    Fire = 2,
    Water = 3,
    Earth = 4,
    Wind = 5,
    Light = 6,
    Dark = 7,
}

impl BattleType {
    pub fn from_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Default),
            1 => Some(Self::AllElement),
            2 => Some(Self::Fire),
            3 => Some(Self::Water),
            4 => Some(Self::Earth),
            5 => Some(Self::Wind),
            6 => Some(Self::Light),
            7 => Some(Self::Dark),
            _ => None,
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Self::Default => "デフォルト",
            Self::AllElement => "全属性",
            Self::Fire => "火属性",
            Self::Water => "水属性",
            Self::Earth => "土属性",
            Self::Wind => "風属性",
            Self::Light => "光属性",
            Self::Dark => "闇属性",
        }
    }
    
    pub fn reactions(&self) -> Vec<ReactionType> {
        match self {
            Self::Default | Self::AllElement => vec![
                ReactionType::Unicode("🔥".to_string()),
                ReactionType::Unicode("💧".to_string()),
                ReactionType::Unicode("🌱".to_string()),
                ReactionType::Unicode("🌪️".to_string()),
                ReactionType::Unicode("✨".to_string()),
                ReactionType::Unicode("🌑".to_string()),
            ],
            Self::Fire => vec![ReactionType::Unicode("🔥".to_string())],
            Self::Water => vec![ReactionType::Unicode("💧".to_string())],
            Self::Earth => vec![ReactionType::Unicode("🌱".to_string())],
            Self::Wind => vec![ReactionType::Unicode("🌪️".to_string())],
            Self::Light => vec![ReactionType::Unicode("✨".to_string())],
            Self::Dark => vec![ReactionType::Unicode("🌑".to_string())],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battle_type_from_value() {
        // Test all valid values
        assert_eq!(BattleType::from_value(0), Some(BattleType::Default));
        assert_eq!(BattleType::from_value(1), Some(BattleType::AllElement));
        assert_eq!(BattleType::from_value(2), Some(BattleType::Fire));
        assert_eq!(BattleType::from_value(3), Some(BattleType::Water));
        assert_eq!(BattleType::from_value(4), Some(BattleType::Earth));
        assert_eq!(BattleType::from_value(5), Some(BattleType::Wind));
        assert_eq!(BattleType::from_value(6), Some(BattleType::Light));
        assert_eq!(BattleType::from_value(7), Some(BattleType::Dark));
        
        // Test invalid values
        assert_eq!(BattleType::from_value(-1), None);
        assert_eq!(BattleType::from_value(8), None);
        assert_eq!(BattleType::from_value(100), None);
    }

    #[test]
    fn test_battle_type_name() {
        assert_eq!(BattleType::Default.name(), "デフォルト");
        assert_eq!(BattleType::AllElement.name(), "全属性");
        assert_eq!(BattleType::Fire.name(), "火属性");
        assert_eq!(BattleType::Water.name(), "水属性");
        assert_eq!(BattleType::Earth.name(), "土属性");
        assert_eq!(BattleType::Wind.name(), "風属性");
        assert_eq!(BattleType::Light.name(), "光属性");
        assert_eq!(BattleType::Dark.name(), "闇属性");
    }

    #[test]
    fn test_battle_type_reactions() {
        // Test Default and AllElement have all 6 reactions
        let default_reactions = BattleType::Default.reactions();
        let all_element_reactions = BattleType::AllElement.reactions();
        assert_eq!(default_reactions.len(), 6);
        assert_eq!(all_element_reactions.len(), 6);
        
        // Verify they contain the expected emoji
        assert!(default_reactions.iter().any(|r| r.to_string().contains("🔥")));
        assert!(default_reactions.iter().any(|r| r.to_string().contains("💧")));
        assert!(default_reactions.iter().any(|r| r.to_string().contains("🌱")));
        assert!(default_reactions.iter().any(|r| r.to_string().contains("🌪️")));
        assert!(default_reactions.iter().any(|r| r.to_string().contains("✨")));
        assert!(default_reactions.iter().any(|r| r.to_string().contains("🌑")));
        
        // Test specific element types have single reactions
        let fire_reactions = BattleType::Fire.reactions();
        assert_eq!(fire_reactions.len(), 1);
        assert!(fire_reactions[0].to_string().contains("🔥"));
        
        let water_reactions = BattleType::Water.reactions();
        assert_eq!(water_reactions.len(), 1);
        assert!(water_reactions[0].to_string().contains("💧"));
        
        let earth_reactions = BattleType::Earth.reactions();
        assert_eq!(earth_reactions.len(), 1);
        assert!(earth_reactions[0].to_string().contains("🌱"));
        
        let wind_reactions = BattleType::Wind.reactions();
        assert_eq!(wind_reactions.len(), 1);
        assert!(wind_reactions[0].to_string().contains("🌪️"));
        
        let light_reactions = BattleType::Light.reactions();
        assert_eq!(light_reactions.len(), 1);
        assert!(light_reactions[0].to_string().contains("✨"));
        
        let dark_reactions = BattleType::Dark.reactions();
        assert_eq!(dark_reactions.len(), 1);
        assert!(dark_reactions[0].to_string().contains("🌑"));
    }

    #[test]
    fn test_battle_type_equality() {
        assert_eq!(BattleType::Default, BattleType::Default);
        assert_eq!(BattleType::Fire, BattleType::Fire);
        assert_ne!(BattleType::Fire, BattleType::Water);
        assert_ne!(BattleType::Default, BattleType::AllElement);
    }

    #[test]
    fn test_battle_type_as_i32() {
        assert_eq!(BattleType::Default as i32, 0);
        assert_eq!(BattleType::AllElement as i32, 1);
        assert_eq!(BattleType::Fire as i32, 2);
        assert_eq!(BattleType::Water as i32, 3);
        assert_eq!(BattleType::Earth as i32, 4);
        assert_eq!(BattleType::Wind as i32, 5);
        assert_eq!(BattleType::Light as i32, 6);
        assert_eq!(BattleType::Dark as i32, 7);
    }
}