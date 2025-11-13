// use super::subnet::*;
// use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn test_error<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    // let mut constructor = SubnetRef::new();
    // let contract = client
    //     .instantiate("DAO", &ink_e2e::alice(), &mut constructor)
    //     .submit()
    //     .await
    //     .expect("instantiate failed");
    
    // let mut call_builder = contract.call_builder::<Subnet>();

    // // when
    // let result = client
    //     .call(
    //         &ink_e2e::alice(),
    //         &call_builder.set_boot_nodes(),
    //     )
    //     .submit()
    //     .await;

    // println!("{:?}", result);
    // println!("xxxxxxxxxxxxxxxx {:?}", result.err());
    // // assert!(result.is_err());

    Ok(())
}
