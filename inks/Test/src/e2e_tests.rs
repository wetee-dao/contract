use ink_e2e::ContractsBackend;

use super::test::*;
use crate::datas::*;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn test_add<Client: E2EBackend>() -> E2EResult<()> {
    let contract = client
        .instantiate("test", &ink_e2e::alice(), &mut TestRef::new())
        .submit()
        .await
        .expect("subnet upload failed");

    let mut call_builder = contract.call_builder::<Test>();

    let _ = client
        .call(
            &ink_e2e::alice(),
            &call_builder.add(
                0,
                TestItem {
                    id: 1,
                    name: "1".as_bytes().to_vec(),
                },
            ),
        )
        .submit()
        .await
        .expect("Calling `add` failed")
        .return_value();

    let _ = client
        .call(
            &ink_e2e::alice(),
            &call_builder.add(
                0,
                TestItem {
                    id: 2,
                    name: "2".as_bytes().to_vec(),
                },
            ),
        )
        .submit()
        .await
        .expect("Calling `add` failed")
        .return_value();

    let _ = client
        .call(
            &ink_e2e::alice(),
            &call_builder.add(
                0,
                TestItem {
                    id: 3,
                    name: "3".as_bytes().to_vec(),
                },
            ),
        )
        .submit()
        .await
        .expect("Calling `add` failed")
        .return_value();

    let list = client
        .call(&ink_e2e::alice(), &call_builder.list(1, None, 100))
        .dry_run()
        .await?
        .return_value();
    println!("list: {:?}", list);

    let _ = client
        .call(&ink_e2e::alice(), &call_builder.del(1))
        .submit()
        .await
        .expect("Calling `add` failed")
        .return_value();

    let list = client
        .call(&ink_e2e::alice(), &call_builder.list(1, None, 100))
        .dry_run()
        .await?
        .return_value();
    println!("list: {:?}", list);


    Ok(())
}
