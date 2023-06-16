use std::{
    path::Path,
    time::{Duration, Instant},
};
use tokio::time;

pub fn time() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .ok()
        .unwrap()
        .as_micros()
}

pub async fn wait(duration: u64) {
    time::sleep(Duration::from_millis(duration)).await;
}

pub fn timeit<F: FnMut() -> T, T>(mut f: F) -> T {
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    println!("it took {} seconds", duration.as_secs());
    result
}

pub fn file_open<P: AsRef<Path>>(path: P) -> Result<std::fs::File, String> {
    std::fs::File::open(path.as_ref()).map_err(|_| {
        let context = format!("error opening {:?}", path.as_ref().to_string_lossy());
        context
    })
}

pub fn rename(prompt: &str, scheduler: &str) -> Result<String, String> {
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

pub fn root() -> String {
    crate::constants::role::ROOT.to_owned()
}
