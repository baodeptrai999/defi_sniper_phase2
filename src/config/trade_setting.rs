use crate::*;
use once_cell::sync::Lazy;

pub static STOP_LOSS: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.stop_loss / 100.0);
pub static REAL_TP_MULTIPLY: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.real_tp_multiply / 100.0);

pub static TS_1: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.trailing_1 / 100.0);
pub static TS_2: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.trailing_2 / 100.0);
pub static TS_3: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.trailing_3 / 100.0);
pub static TS_4: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.trailing_4 / 100.0);
pub static TS_5: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.trailing_5 / 100.0);

pub static TS_1_STOP: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.trailing_1_stop / 100.0);
pub static TS_2_STOP: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.trailing_2_stop / 100.0);
pub static TS_3_STOP: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.trailing_3_stop / 100.0);
pub static TS_4_STOP: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.trailing_4_stop / 100.0);
pub static TS_5_STOP: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.trailing_5_stop / 100.0);

pub static TS_1_SELL_PCNT: Lazy<f64> =
    Lazy::new(|| CONFIG.sell_setting.trailing_1_sell_percentage / 100.0);
pub static TS_2_SELL_PCNT: Lazy<f64> =
    Lazy::new(|| CONFIG.sell_setting.trailing_2_sell_percentage / 100.0);
pub static TS_3_SELL_PCNT: Lazy<f64> =
    Lazy::new(|| CONFIG.sell_setting.trailing_3_sell_percentage / 100.0);
pub static TS_4_SELL_PCNT: Lazy<f64> =
    Lazy::new(|| CONFIG.sell_setting.trailing_4_sell_percentage / 100.0);
pub static TS_5_SELL_PCNT: Lazy<f64> =
    Lazy::new(|| CONFIG.sell_setting.trailing_5_sell_percentage / 100.0);

pub fn init_validator() {
    let _ = *VALID_TS_STOP;
    let _ = *VALID_TS;
}

pub static VALID_TS_STOP: Lazy<bool> = Lazy::new(|| {
    let ts1_val = *TS_1 * (1.0 - *TS_1_STOP) * 100.0;
    let ts2_val = *TS_2 * (1.0 - *TS_2_STOP) * 100.0;
    let ts3_val = *TS_3 * (1.0 - *TS_3_STOP) * 100.0;
    let ts4_val = *TS_4 * (1.0 - *TS_4_STOP) * 100.0;
    let ts5_val = *TS_5 * (1.0 - *TS_5_STOP) * 100.0;

    if !(ts1_val < ts2_val && ts2_val < ts3_val && ts3_val < ts4_val && ts4_val < ts5_val) {
        error!(
            "[ERROR] => Invalid TS_STOP\n\t* TS STOP Point should be in order\n\t* TS_1_TS_STOP : {:5.3}\n\t* TS_2_TS_STOP : {:5.3}\n\t* TS_3_TS_STOP : {:5.3}\n\t* TS_4_TS_STOP : {:5.3}\n\t* TS_5_TS_STOP : {:5.3}",
            *TS_1 * (1.0 - *TS_1_STOP) * 100.0,
            *TS_2 * (1.0 - *TS_2_STOP) * 100.0,
            *TS_3 * (1.0 - *TS_3_STOP) * 100.0,
            *TS_4 * (1.0 - *TS_4_STOP) * 100.0,
            *TS_5 * (1.0 - *TS_5_STOP) * 100.0,
        );
        panic!("INVALID CONFIG");
    };

    true
});

pub static VALID_TS: Lazy<bool> = Lazy::new(|| {
    if !(*STOP_LOSS < *TS_1 && *TS_1 < *TS_2 && *TS_2 < *TS_3 && *TS_3 < *TS_4 && *TS_4 < *TS_5) {
        error!(
            "[ERROR] => Invalid Order\n\tSTOP_LOSS : {:5.3}\n\t* TS_1 : {:5.3}\n\t* TS_2 : {:5.3}\n\t* TS_3 : {:5.3}\n\t* TS_4 : {:5.3}\n\t* TS_5 : {:5.3}",
            *STOP_LOSS, *TS_1, *TS_2, *TS_3, *TS_4, *TS_5
        );
        panic!("INVALID CONFIG: TS Range");
    };

    true
});
