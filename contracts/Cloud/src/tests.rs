use super::cloud::*;

fn init() -> Cloud {
    Cloud::new()
}

#[ink::test]
fn create_user_pod() {
    let mut c = init();

    let p = c.create_user_pod();

    assert!(p.is_ok());
    _ = c.create_user_pod();

    let list = c.user_pods();
    println!("list: {:?}", list)
}
