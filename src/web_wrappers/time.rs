use js_sys::Date;
use wasm_bindgen::JsValue;

pub struct Instant {
    start_date: f64,
}

impl Instant {
    pub fn now() -> Self {
        Self {
            start_date: Date::now(),
        }
    }
    pub fn elapsed(&self) -> Duration {
        let milis = JsValue::from_f64(Date::now() - self.start_date);
        Duration {
            milis: milis.as_f64().unwrap(),
        }
    }
}

pub struct Duration {
    milis: f64,
}
impl Duration {
    pub fn as_nanos(&self) -> u128 {
        (self.milis * 1000000.0).round() as u128
    }
    pub fn as_secs(&self) -> u64 {
        (self.milis / 1000.0).round() as u64
    }
}
