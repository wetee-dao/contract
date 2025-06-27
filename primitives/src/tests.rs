use crate::{combine_u32_to_u64, split_u64_to_u32};

#[test]
fn test_add() {
    let k1: u32 = 123456789;
    let k2: u32 = 987654321;

    println!("init value: k1 = {}, k2 = {}", k1, k2);
    let combined = combine_u32_to_u64(k1, k2);

    println!("Combined u64: {}", combined);

    let (a, b) = split_u64_to_u32(combined);

    println!("Split back: k1 = {}, k2 = {}", a, b);

    assert_eq!(k1, a);
    assert_eq!(k2, b);
}
