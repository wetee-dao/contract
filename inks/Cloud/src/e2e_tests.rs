use ink::primitives::AccountIdMapper;
use ink_e2e::ContractsBackend;
use subnet::SubnetRef;
use cloud_test_macros::setup_contracts;
use ink::H256;

use super::cloud::*;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn test_delete_user_disk<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    let _ = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.create_disk("d1".as_bytes().to_vec(), 10),
        )
        .submit()
        .await
        .expect("Calling `init_disk` failed")
        .return_value();

    let _ = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.create_disk("d2".as_bytes().to_vec(), 10),
        )
        .submit()
        .await
        .expect("Calling `init_disk` failed")
        .return_value();

    let alice = AccountIdMapper::to_address(&ink_e2e::alice().public_key().as_ref());
    let list = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.user_disks(alice, None, 100),
        )
        .dry_run()
        .await?
        .return_value();
    println!("list: {:?}", list);

    let _ = client
        .call(&ink_e2e::alice(), &cloud_call_builder.del_disk(0))
        .submit()
        .await
        .expect("Calling `delete_disk` failed")
        .return_value();

    let list2 = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.user_disks(alice, None, 100),
        )
        .dry_run()
        .await?
        .return_value();
    println!("list2: {:?}", list2);

    Ok(())
}

#[ink_e2e::test]
async fn test_create_and_list_disks<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    let alice = AccountIdMapper::to_address(&ink_e2e::alice().public_key().as_ref());
    
    // 创建多个磁盘
    let disk_id1 = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.create_disk("disk1".as_bytes().to_vec(), 20),
        )
        .submit()
        .await
        .expect("Calling `create_disk` failed")
        .return_value()
        .expect("Failed to get disk_id1");

    let disk_id2 = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.create_disk("disk2".as_bytes().to_vec(), 30),
        )
        .submit()
        .await
        .expect("Calling `create_disk` failed")
        .return_value()
        .expect("Failed to get disk_id2");

    println!("disk_id1: {:?}, disk_id2: {:?}", disk_id1, disk_id2);

    // 查询磁盘列表
    let disks = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.user_disks(alice, None, 100),
        )
        .dry_run()
        .await?
        .return_value();
    println!("disks: {:?}", disks);

    // 查询单个磁盘
    let disk = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.disk(alice, disk_id1),
        )
        .dry_run()
        .await?
        .return_value();
    println!("disk: {:?}", disk);

    Ok(())
}

#[ink_e2e::test]
async fn test_create_and_delete_secret<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    let alice = AccountIdMapper::to_address(&ink_e2e::alice().public_key().as_ref());
    let secret_hash = H256::from([1u8; 32]);
    
    // 创建密钥
    let secret_id = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.create_secret("secret_key".as_bytes().to_vec(), secret_hash),
        )
        .submit()
        .await
        .expect("Calling `create_secret` failed")
        .return_value()
        .expect("Failed to get secret_id");

    println!("secret_id: {:?}", secret_id);

    // 查询密钥列表
    let secrets = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.user_secrets(alice, None, 100),
        )
        .dry_run()
        .await?
        .return_value();
    println!("secrets: {:?}", secrets);

    // 查询单个密钥
    let secret = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.secret(alice, secret_id),
        )
        .dry_run()
        .await?
        .return_value();
    println!("secret: {:?}", secret);

    // 删除密钥
    let _ = client
        .call(&ink_e2e::alice(), &cloud_call_builder.del_secret(secret_id))
        .submit()
        .await
        .expect("Calling `del_secret` failed")
        .return_value();

    // 再次查询密钥列表
    let secrets_after = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.user_secrets(alice, None, 100),
        )
        .dry_run()
        .await?
        .return_value();
    println!("secrets_after: {:?}", secrets_after);

    Ok(())
}

#[ink_e2e::test]
async fn test_basic_info<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 查询子网地址
    let subnet_addr = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.subnet_address(),
        )
        .dry_run()
        .await?
        .return_value();
    println!("subnet_addr: {:?}", subnet_addr);

    // 查询 Pod 合约代码哈希
    let pod_contract = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.pod_contract(),
        )
        .dry_run()
        .await?
        .return_value();
    println!("pod_contract: {:?}", pod_contract);

    // 查询挖矿间隔
    let mint_interval = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.mint_interval(),
        )
        .dry_run()
        .await?
        .return_value();
    println!("mint_interval: {:?}", mint_interval);

    Ok(())
}

#[ink_e2e::test]
async fn test_pod_queries<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 查询 Pod 总数
    let pod_len = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.pod_len(),
        )
        .dry_run()
        .await?
        .return_value();
    println!("pod_len: {:?}", pod_len);

    // 查询用户的 Pod 数量
    let user_pod_len = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.user_pod_len(),
        )
        .dry_run()
        .await?
        .return_value();
    println!("user_pod_len: {:?}", user_pod_len);

    // 查询用户 Pod 列表
    let user_pods = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.user_pods(None, 100),
        )
        .dry_run()
        .await?
        .return_value();
    println!("user_pods: {:?}", user_pods);

    // 查询所有 Pod 列表
    let pods = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.pods(None, 100),
        )
        .dry_run()
        .await?
        .return_value();
    println!("pods: {:?}", pods);

    Ok(())
}
