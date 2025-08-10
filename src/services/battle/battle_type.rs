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