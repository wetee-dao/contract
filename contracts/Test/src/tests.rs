use super::test::*;
use crate::datas::TestItem;

fn _init() -> Test {
    Test::new()
}

#[ink::test]
fn test_add() {
    let mut t = _init();
    _ = t.add(
        0,
        TestItem {
            id: 1,
            name: "test".as_bytes().to_vec(),
        },
    );
    _ = t.add(
        0,
        TestItem {
            id: 2,
            name: "test".as_bytes().to_vec(),
        },
    );
    _ = t.add(
        0,
        TestItem {
            id: 3,
            name: "test".as_bytes().to_vec(),
        },
    );

    let mut list = t.list(0);
    println!("{:?}", list);

    _ = t.del(1);

    list = t.list(0);
    println!("{:?}", list);
}
