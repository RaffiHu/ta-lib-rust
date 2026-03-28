use crate::settings::{
    CANDLE_SETTINGS_COUNT, CandleSetting, CandleSettingType, Compatibility,
    DEFAULT_CANDLE_SETTINGS, FUNC_UNST_ID_COUNT, FuncUnstId, RetCode,
};

/// Stateful TA-Lib execution context.
#[derive(Debug, Clone)]
pub struct Context {
    unstable_periods: [u32; FUNC_UNST_ID_COUNT],
    compatibility: Compatibility,
    candle_settings: [CandleSetting; CANDLE_SETTINGS_COUNT],
}

impl Default for Context {
    fn default() -> Self {
        Self {
            unstable_periods: [0; FUNC_UNST_ID_COUNT],
            compatibility: Compatibility::Default,
            candle_settings: DEFAULT_CANDLE_SETTINGS,
        }
    }
}

impl Context {
    /// Creates a fresh context with TA-Lib default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets the context to TA-Lib defaults.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Sets an unstable period.
    pub fn set_unstable_period(&mut self, id: FuncUnstId, unstable_period: u32) -> RetCode {
        match id {
            FuncUnstId::FuncUnstNone => RetCode::BadParam,
            FuncUnstId::FuncUnstAll => {
                self.unstable_periods.fill(unstable_period);
                RetCode::Success
            }
            _ => match id.index() {
                Some(index) => {
                    self.unstable_periods[index] = unstable_period;
                    RetCode::Success
                }
                None => RetCode::BadParam,
            },
        }
    }

    /// Gets an unstable period.
    #[must_use]
    pub fn get_unstable_period(&self, id: FuncUnstId) -> u32 {
        id.index()
            .map(|index| self.unstable_periods[index])
            .unwrap_or(0)
    }

    /// Sets the active compatibility mode.
    pub fn set_compatibility(&mut self, value: Compatibility) -> RetCode {
        self.compatibility = value;
        RetCode::Success
    }

    /// Gets the active compatibility mode.
    #[must_use]
    pub fn get_compatibility(&self) -> Compatibility {
        self.compatibility
    }

    /// Sets one candle setting.
    pub fn set_candle_settings(
        &mut self,
        setting_type: CandleSettingType,
        range_type: crate::RangeType,
        avg_period: i32,
        factor: f64,
    ) -> RetCode {
        let Some(index) = setting_type.index() else {
            return RetCode::BadParam;
        };

        self.candle_settings[index] = CandleSetting {
            setting_type,
            range_type,
            avg_period,
            factor,
        };

        RetCode::Success
    }

    /// Restores the TA-Lib default candle settings.
    pub fn restore_candle_default_settings(&mut self, setting_type: CandleSettingType) -> RetCode {
        match setting_type {
            CandleSettingType::AllCandleSettings => {
                self.candle_settings = DEFAULT_CANDLE_SETTINGS;
                RetCode::Success
            }
            _ => {
                let Some(index) = setting_type.index() else {
                    return RetCode::BadParam;
                };
                self.candle_settings[index] = DEFAULT_CANDLE_SETTINGS[index];
                RetCode::Success
            }
        }
    }

    /// Gets one candle setting.
    #[must_use]
    pub fn candle_setting(&self, setting_type: CandleSettingType) -> Option<CandleSetting> {
        setting_type
            .index()
            .map(|index| self.candle_settings[index])
    }
}
