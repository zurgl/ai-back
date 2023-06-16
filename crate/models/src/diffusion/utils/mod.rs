use std::path::Path;
use std::time::Instant;

pub mod bar;
pub use bar::Bar;

pub fn timeit<F: FnMut() -> T, T>(mut f: F) -> T {
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    println!("it took {} seconds", duration.as_secs());
    result
}

pub fn file_open<P: AsRef<Path>>(path: P) -> anyhow::Result<std::fs::File> {
    std::fs::File::open(path.as_ref()).map_err(|e| {
        let context = format!("error opening {:?}", path.as_ref().to_string_lossy());
        anyhow::Error::new(e).context(context)
    })
}

pub fn rename(prompt: &str, scheduler: &str) -> anyhow::Result<String> {
    let name = prompt
        .split_whitespace()
        .map(|world| world.to_lowercase())
        .collect::<Vec<String>>()
        .join("_");
    let with_scheduler = format!("{scheduler}-{name}");

    Ok(with_scheduler)
}

pub fn wait_for_input() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

use tch::Tensor;

pub fn from<'a, T>(t: &'a Tensor) -> T
where
    <T as TryFrom<&'a tch::Tensor>>::Error: std::fmt::Debug,
    T: TryFrom<&'a Tensor>,
{
    T::try_from(t).unwrap()
}

pub fn f64_from(t: &Tensor) -> f64 {
    from::<f64>(t)
}

pub fn slerp(t: f64, x: &tch::Tensor, y: &tch::Tensor) -> anyhow::Result<tch::Tensor> {
    let norm = f64_from(&x.norm()) * f64_from(&y.norm());
    let dot: f64 = f64_from(&((x * y) / norm).sum(tch::Kind::Float));

    Ok(if dot.abs() > 0.9995 {
        (1. - t) * x + t * y
    } else {
        let theta_0 = dot.acos();
        let sin_theta_0 = theta_0.sin();
        let theta_t = theta_0 * t;
        let sin_theta_t = theta_t.sin();
        let s0 = (theta_0 - theta_t).sin() / sin_theta_0;
        let s1 = sin_theta_t / sin_theta_0;
        s0 * x + s1 * y
    })
}

pub fn linspace(start: i64, end: i64, steps: i64) -> Vec<(u64, f64)> {
    let mut v: Vec<(u64, f64)> = Vec::new();
    let c = ((end - start) as f64) / (steps as f64 - 1f64);
    for k in 0..steps {
        v.push((k as u64, c * (k as f64)));
    }
    v
}
