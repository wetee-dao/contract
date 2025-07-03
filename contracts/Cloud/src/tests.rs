use super::cloud::*;

fn init() -> Cloud {
    Cloud::new()
}

#[ink::test]
fn user_pod() {
    let mut c = init();

    let p = c.create_user_pod();

    assert!(p.is_ok());
    _ = c.create_user_pod();

    let list = c.user_pods();
    println!("list: {:?}", list);

    let list2 = c.user_desc_pods();
    println!("list2: {:?}", list2);
}


#[ink::test]
fn pods() {
    let mut c = init();

    let p = c.create_user_pod();

    assert!(p.is_ok());
    _ = c.create_user_pod();

    let list = c.pods();
    println!("list: {:?}", list);

    let list2 = c.desc_pods();
    println!("list2: {:?}", list2);
}
