/// Return codes mirrored from upstream TA-Lib.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum RetCode {
    Success = 0,
    LibNotInitialize = 1,
    BadParam = 2,
    AllocErr = 3,
    GroupNotFound = 4,
    FuncNotFound = 5,
    InvalidHandle = 6,
    InvalidParamHolder = 7,
    InvalidParamHolderType = 8,
    InvalidParamFunction = 9,
    InputNotAllInitialize = 10,
    OutputNotAllInitialize = 11,
    OutOfRangeStartIndex = 12,
    OutOfRangeEndIndex = 13,
    InvalidListType = 14,
    BadObject = 15,
    NotSupported = 16,
    InternalError = 5000,
    UnknownErr = 0xFFFF,
}

/// Compatibility mode mirrored from upstream TA-Lib.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum Compatibility {
    #[default]
    Default = 0,
    Metastock = 1,
}

/// Moving-average type metadata mirrored from upstream TA-Lib.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MAType {
    Sma = 0,
    Ema = 1,
    Wma = 2,
    Dema = 3,
    Tema = 4,
    Trima = 5,
    Kama = 6,
    Mama = 7,
    T3 = 8,
}

/// Function unstable-period identifiers mirrored from upstream TA-Lib.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum FuncUnstId {
    Adx = 0,
    Adxr = 1,
    Atr = 2,
    Cmo = 3,
    Dx = 4,
    Ema = 5,
    HtDcPeriod = 6,
    HtDcPhase = 7,
    HtPhasor = 8,
    HtSine = 9,
    HtTrendline = 10,
    HtTrendMode = 11,
    Imi = 12,
    Kama = 13,
    Mama = 14,
    Mfi = 15,
    MinusDI = 16,
    MinusDM = 17,
    Natr = 18,
    PlusDI = 19,
    PlusDM = 20,
    Rsi = 21,
    StochRsi = 22,
    T3 = 23,
    FuncUnstAll = 24,
    FuncUnstNone = -1,
}

pub(crate) const FUNC_UNST_ID_COUNT: usize = 24;

impl FuncUnstId {
    pub(crate) fn index(self) -> Option<usize> {
        let value = self as i32;
        if value < 0 || value >= FUNC_UNST_ID_COUNT as i32 {
            return None;
        }

        Some(value as usize)
    }
}

/// Candle range type mirrored from upstream TA-Lib.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum RangeType {
    RealBody = 0,
    HighLow = 1,
    Shadows = 2,
}

/// Candle setting kind mirrored from upstream TA-Lib.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum CandleSettingType {
    BodyLong = 0,
    BodyVeryLong = 1,
    BodyShort = 2,
    BodyDoji = 3,
    ShadowLong = 4,
    ShadowVeryLong = 5,
    ShadowShort = 6,
    ShadowVeryShort = 7,
    Near = 8,
    Far = 9,
    Equal = 10,
    AllCandleSettings = 11,
}

pub(crate) const CANDLE_SETTINGS_COUNT: usize = 11;

impl CandleSettingType {
    pub(crate) fn index(self) -> Option<usize> {
        let value = self as i32;
        if value < 0 || value >= CANDLE_SETTINGS_COUNT as i32 {
            return None;
        }

        Some(value as usize)
    }
}

/// Candle setting payload mirrored from upstream TA-Lib.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CandleSetting {
    /// Setting identifier.
    pub setting_type: CandleSettingType,
    /// Range basis used by the setting.
    pub range_type: RangeType,
    /// Averaging period.
    pub avg_period: i32,
    /// Multiplier factor.
    pub factor: f64,
}

pub(crate) const DEFAULT_CANDLE_SETTINGS: [CandleSetting; CANDLE_SETTINGS_COUNT] = [
    CandleSetting {
        setting_type: CandleSettingType::BodyLong,
        range_type: RangeType::RealBody,
        avg_period: 10,
        factor: 1.0,
    },
    CandleSetting {
        setting_type: CandleSettingType::BodyVeryLong,
        range_type: RangeType::RealBody,
        avg_period: 10,
        factor: 3.0,
    },
    CandleSetting {
        setting_type: CandleSettingType::BodyShort,
        range_type: RangeType::RealBody,
        avg_period: 10,
        factor: 1.0,
    },
    CandleSetting {
        setting_type: CandleSettingType::BodyDoji,
        range_type: RangeType::HighLow,
        avg_period: 10,
        factor: 0.1,
    },
    CandleSetting {
        setting_type: CandleSettingType::ShadowLong,
        range_type: RangeType::RealBody,
        avg_period: 0,
        factor: 1.0,
    },
    CandleSetting {
        setting_type: CandleSettingType::ShadowVeryLong,
        range_type: RangeType::RealBody,
        avg_period: 0,
        factor: 2.0,
    },
    CandleSetting {
        setting_type: CandleSettingType::ShadowShort,
        range_type: RangeType::Shadows,
        avg_period: 10,
        factor: 1.0,
    },
    CandleSetting {
        setting_type: CandleSettingType::ShadowVeryShort,
        range_type: RangeType::HighLow,
        avg_period: 10,
        factor: 0.1,
    },
    CandleSetting {
        setting_type: CandleSettingType::Near,
        range_type: RangeType::HighLow,
        avg_period: 5,
        factor: 0.2,
    },
    CandleSetting {
        setting_type: CandleSettingType::Far,
        range_type: RangeType::HighLow,
        avg_period: 5,
        factor: 0.6,
    },
    CandleSetting {
        setting_type: CandleSettingType::Equal,
        range_type: RangeType::HighLow,
        avg_period: 5,
        factor: 0.05,
    },
];
