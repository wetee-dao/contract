use super::test::*;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn test_error<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    let mut constructor = TestCaseRef::new();
    let contract = client
        .instantiate("test", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    
    let call_builder = contract.call_builder::<TestCase>();

    // when
    let result = client
        .call(
            &ink_e2e::alice(),
            &call_builder.get_transaction(2),
        )
        .submit()
        .await;

    println!("{:?}", result);

    assert!(result.is_err());

    Ok(())
}

#[ink_e2e::test]
async fn test_set<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    let mut constructor = TestCaseRef::new();
    let contract = client
        .instantiate("test", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");

    let mut call_builder = contract.call_builder::<TestCase>();
    let result = client
        .call(
            &ink_e2e::alice(),
            &call_builder.set(),
        )
        .submit()
        .await;

    assert!(result.is_ok());

    println!("{:?}", result);

    let call_builder2 = contract.call_builder::<TestCase>();
    let result2 = client
        .call(
            &ink_e2e::alice(),
            &call_builder2.get(),
        )
        .submit()
        .await
        .expect("Calling `insert_balance` failed")
        .return_value();

    println!("{:?}", result2);

    assert!(result2 == 2);

    // assert!(result.is_ok());

    Ok(())
}