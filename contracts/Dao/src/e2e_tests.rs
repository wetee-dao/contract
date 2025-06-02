use super::dao::*;
use crate::traits::Member;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn test_member_list<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    let mut constructor = DAORef::new(Vec::new(), None);
    let contract = client
        .instantiate("DAO", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");

    let call_builder = contract.call_builder::<DAO>();

    // when
    let result = client
        .call(&ink_e2e::alice(), &call_builder.list())
        .dry_run()
        .await?
        .return_value();

    println!("{:?}", result);

    Ok(())
}
