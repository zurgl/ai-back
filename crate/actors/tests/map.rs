use dashmap::DashMap;

#[test]
fn run() {
    let words = DashMap::new();
    words.insert(0_u32, "world".to_string());
    words.insert(1_u32, "new".to_string());
    words.insert(2_u32, "DashMap".to_string());

    let v: Vec<u32> = words.iter().map(|value| *value.key()).collect();

    println!("{v:?}");
}
