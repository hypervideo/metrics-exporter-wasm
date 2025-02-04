#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BenchResult {
    pub iterations: usize,
    pub avg: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p90: f64,
    pub p99: f64,
}

impl std::fmt::Display for BenchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            iterations,
            avg,
            min: _,
            max: _,
            p50: _,
            p90: _,
            p99: _,
        } = self;
        write!(f, "avg={avg:.2}ms | it={iterations} ",)
    }
}

#[allow(dead_code)]
pub fn bench<T>(f: impl Fn() -> T) -> BenchResult {
    bench_env((), |_| f())
}

#[allow(dead_code)]
pub fn bench_env<E, T>(env: E, f: impl Fn(E) -> T) -> BenchResult
where
    E: Clone,
{
    let mut envs = vec![env.clone(); 100];
    let start = now();
    let mut i = 0;
    let mut times = vec![];

    while now() - start < 1000.0 {
        i += 1;
        let env = if let Some(env) = envs.pop() {
            env
        } else {
            envs = vec![env.clone(); 100];
            env.clone()
        };
        let bench_start = now();
        std::hint::black_box(f(env));
        times.push(now() - bench_start);
    }

    let avg = times.iter().sum::<f64>() / times.len() as f64;
    let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let p50 = percentile(&times, 0.5);
    let p90 = percentile(&times, 0.9);
    let p99 = percentile(&times, 0.99);

    BenchResult {
        iterations: i,
        avg,
        min,
        max,
        p50,
        p90,
        p99,
    }
}

fn percentile(data: &[f64], p: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut data = data.to_vec();
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let index = (data.len() as f64 * p).round() as usize;
    data[index.min(data.len() - 1)]
}

/// High-precision time in milliseconds
#[cfg(target_arch = "wasm32")]
fn now() -> f64 {
    gloo::utils::window().performance().unwrap().now()
}

#[cfg(not(target_arch = "wasm32"))]
fn now() -> f64 {
    use std::{sync::LazyLock, time::Instant};
    static START: LazyLock<Instant> = LazyLock::new(Instant::now);
    (Instant::now() - *START).as_secs_f64() * 1000.0
}
