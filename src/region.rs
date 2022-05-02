use std::str::FromStr;
use std::convert::Infallible;
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

            _ => None
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
        match x {
            "jp_ja" => Ok(Japanese),
            "us_en" => Ok(UsEnglish),
            "us_fr" => Ok(UsFrench),
            "us_es" => Ok(UsSpanish),
            "eu_en" => Ok(EuEnglish),
            "eu_fr" => Ok(EuFrench),
            "eu_es" => Ok(EuSpanish),
            "eu_de" => Ok(EuGerman),
            "eu_nl" => Ok(EuDutch),
            "eu_it" => Ok(EuItalian),
            "eu_ru" => Ok(EuRussian),
            "kr_ko" => Ok(Korean),
            "zh_cn" => Ok(ChinaChinese),
            "zh_tw" => Ok(TaiwanChinese),

            _ => Ok(None)
        }
    }
}