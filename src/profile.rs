//! sidediff 按需启用的性能测量；嵌套区间存在重叠。
use serde::Serialize;
use std::{cell::RefCell, collections::BTreeMap, sync::OnceLock, time::Instant};

#[derive(Default, Clone, Serialize)]
pub struct Sample {
    pub calls: u64,
    pub ms: f64,
    pub max_ms: f64,
}
thread_local! {
    static SAMPLES: RefCell<BTreeMap<&'static str, Sample>> = RefCell::new(BTreeMap::new());
}
pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("SIDEDIFF_PROFILE_LOAD").is_some())
}
pub fn measure<T>(name: &'static str, f: impl FnOnce() -> T) -> T {
    if !enabled() {
        return f();
    }
    let start = Instant::now();
    let result = f();
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    SAMPLES.with(|samples| {
        let mut samples = samples.borrow_mut();
        let sample = samples.entry(name).or_default();
        sample.calls += 1;
        sample.ms += ms;
        sample.max_ms = sample.max_ms.max(ms);
    });
    result
}
pub fn snapshot() -> BTreeMap<&'static str, Sample> {
    SAMPLES.with(|samples| samples.borrow().clone())
}

pub fn merge(other: BTreeMap<&'static str, Sample>) {
    SAMPLES.with(|samples| {
        let mut samples = samples.borrow_mut();
        for (name, value) in other {
            let sample = samples.entry(name).or_default();
            sample.calls += value.calls;
            sample.ms += value.ms;
            sample.max_ms = sample.max_ms.max(value.max_ms);
        }
    });
}
