use std::convert::Infallible;
use std::str::FromStr;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    None = 0,
    Japan = 1,
    UnitedStates = 2,
    Europe = 3,
    Korea = 4,
    China = 5,
}

impl From<usize> for Locale {
    fn from(r: usize) -> Locale {
        use Locale::*;
        match r {
            1 => Japan,
            2 => UnitedStates,
            3 => Europe,
            4 => Korea,
            5 => China,

            _ => None,
        }
    }
}

impl From<u32> for Locale {
    fn from(x: u32) -> Self {
        Locale::from(x as usize)
    }
}

impl From<u16> for Locale {
    fn from(x: u16) -> Self {
        Locale::from(x as usize)
    }
}

impl From<u8> for Locale {
    fn from(x: u8) -> Self {
        Locale::from(x as usize)
    }
}

impl FromStr for Locale {
    type Err = Infallible;
    fn from_str(x: &str) -> Result<Self, Self::Err> {
        use Locale::*;
        Ok(match x {
            "jp" => Japan,
            "us" => UnitedStates,
            "eu" => Europe,
            "kr" => Korea,
            "zh" => China,

            _ => None,
        })
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    None = 0,
    Japanese = 1,
    UsEnglish = 2,
    UsFrench = 3,
    UsSpanish = 4,
    EuEnglish = 5,
    EuFrench = 6,
    EuSpanish = 7,
    EuGerman = 8,
    EuDutch = 9,
    EuItalian = 10,
    EuRussian = 11,
    Korean = 12,
    ChinaChinese = 13,
    TaiwanChinese = 14,
}

impl Region {
    pub fn get_locale(&self) -> Option<Locale> {
        match self {
            Region::Japanese => Some(Locale::Japan),
            Region::UsEnglish => Some(Locale::UnitedStates),
            Region::UsFrench => Some(Locale::UnitedStates),
            Region::UsSpanish => Some(Locale::UnitedStates),
            Region::EuEnglish => Some(Locale::Europe),
            Region::EuFrench => Some(Locale::Europe),
            Region::EuSpanish => Some(Locale::Europe),
            Region::EuGerman => Some(Locale::Europe),
            Region::EuDutch => Some(Locale::Europe),
            Region::EuItalian => Some(Locale::Europe),
            Region::EuRussian => Some(Locale::Europe),
            Region::Korean => Some(Locale::Korea),
            Region::ChinaChinese => Some(Locale::China),
            Region::TaiwanChinese => Some(Locale::China),

            _ => None,
        }
    }
}

impl From<usize> for Region {
    fn from(r: usize) -> Region {
        use Region::*;
        match r {
            1 => Japanese,
            2 => UsEnglish,
            3 => UsFrench,
            4 => UsSpanish,
            5 => EuEnglish,
            6 => EuFrench,
            7 => EuSpanish,
            8 => EuGerman,
            9 => EuDutch,
            10 => EuItalian,
            11 => EuRussian,
            12 => Korean,
            13 => ChinaChinese,
            14 => TaiwanChinese,

            _ => None,
        }
    }
}

impl From<u32> for Region {
    fn from(x: u32) -> Self {
        Region::from(x as usize)
    }
}

impl From<u16> for Region {
    fn from(x: u16) -> Self {
        Region::from(x as usize)
    }
}

impl From<u8> for Region {
    fn from(x: u8) -> Self {
        Region::from(x as usize)
    }
}

impl FromStr for Region {
    type Err = Infallible;
    fn from_str(x: &str) -> Result<Self, Self::Err> {
        use Region::*;
        Ok(match x {
            "jp_ja" => Japanese,
            "us_en" => UsEnglish,
            "us_fr" => UsFrench,
            "us_es" => UsSpanish,
            "eu_en" => EuEnglish,
            "eu_fr" => EuFrench,
            "eu_es" => EuSpanish,
            "eu_de" => EuGerman,
            "eu_nl" => EuDutch,
            "eu_it" => EuItalian,
            "eu_ru" => EuRussian,
            "kr_ko" => Korean,
            "zh_cn" => ChinaChinese,
            "zh_tw" => TaiwanChinese,

            _ => None,
        })
    }
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Region::None => write!(f, ""),
            Region::Japanese => write!(f, "jp_ja"),
            Region::UsEnglish => write!(f, "us_en"),
            Region::UsFrench => write!(f, "us_fr"),
            Region::UsSpanish => write!(f, "us_es"),
            Region::EuEnglish => write!(f, "eu_en"),
            Region::EuFrench => write!(f, "eu_fr"),
            Region::EuSpanish => write!(f, "eu_es"),
            Region::EuGerman => write!(f, "eu_de"),
            Region::EuDutch => write!(f, "eu_nl"),
            Region::EuItalian => write!(f, "eu_it"),
            Region::EuRussian => write!(f, "eu_ru"),
            Region::Korean => write!(f, "kr_ko"),
            Region::ChinaChinese => write!(f, "zh_cn"),
            Region::TaiwanChinese => write!(f, "zh_tw"),
        }
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Locale::None => write!(f, ""),
            Locale::Japan => write!(f, "jp"),
            Locale::UnitedStates => write!(f, "us"),
            Locale::Europe => write!(f, "eu"),
            Locale::Korea => write!(f, "kr"),
            Locale::China => write!(f, "zh"),
        }
    }
}
