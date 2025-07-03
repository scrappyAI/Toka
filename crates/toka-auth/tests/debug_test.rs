fn main() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    println!("now: {}", now);
    
    for delta_secs in [-1, -10, -3600] {
        let exp = (now as i64 + delta_secs) as u64;
        println!("delta_secs: {}, exp: {}, exp > now: {}", 
                 delta_secs, exp, exp > now);
    }
}
