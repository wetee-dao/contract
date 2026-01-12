use ink::primitives::AccountIdMapper;
use ink_e2e::ContractsBackend;
use cloud_test_macros::setup_contracts;
use ink::H256;
use primitives::AssetInfo;

use super::cloud::*;
use crate::datas::{Container, ContainerInput, EditType};

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn test_delete_user_disk<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    let alice = AccountIdMapper::to_address(&ink_e2e::alice().public_key().as_ref());
    
    // 创建第一个磁盘
    let disk_id1 = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.create_disk("d1".as_bytes().to_vec(), 10),
        )
        .submit()
        .await
        .expect("Calling `create_disk` failed")
        .return_value()
        .expect("Failed to get disk_id1");

    // 创建第二个磁盘
    let disk_id2 = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.create_disk("d2".as_bytes().to_vec(), 10),
        )
        .submit()
        .await
        .expect("Calling `create_disk` failed")
        .return_value()
        .expect("Failed to get disk_id2");

    // 查询磁盘列表（应该有2个）
    let list = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.user_disks(alice, None, 100),
        )
        .dry_run()
        .await?
        .return_value();
    println!("list: {:?}", list);
    assert_eq!(list.len(), 2, "应该有两个磁盘");

    // 删除第一个磁盘
    let _ = client
        .call(&ink_e2e::alice(), &cloud_call_builder.del_disk(disk_id1))
        .submit()
        .await
        .expect("Calling `del_disk` failed")
        .return_value();

    // 再次查询磁盘列表（应该只剩1个）
    let list2 = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.user_disks(alice, None, 100),
        )
        .dry_run()
        .await?
        .return_value();
    println!("list2: {:?}", list2);
    assert_eq!(list2.len(), 1, "删除后应该只剩一个磁盘");
    
    // 验证删除的是正确的磁盘（剩余的应该是 disk_id2）
    assert_eq!(list2[0].0, disk_id2, "剩余的磁盘ID应该是 disk_id2");

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

#[ink_e2e::test]
async fn test_set_pod_contract<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    let new_hash = H256::from([2u8; 32]);
    
    // 设置新的 Pod 合约代码哈希（alice 是创建者，即 gov_contract）
    let _ = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.set_pod_contract(new_hash),
        )
        .submit()
        .await
        .expect("Calling `set_pod_contract` failed")
        .return_value();

    // 查询更新后的代码哈希
    let pod_contract = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.pod_contract(),
        )
        .dry_run()
        .await?
        .return_value();
    println!("pod_contract after set: {:?}", pod_contract);

    Ok(())
}

#[ink_e2e::test]
async fn test_set_mint_interval<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 查询初始挖矿间隔
    let initial_interval = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.mint_interval(),
        )
        .dry_run()
        .await?
        .return_value();
    println!("initial_interval: {:?}", initial_interval);

    // 设置新的挖矿间隔（alice 是创建者，即 gov_contract）
    let new_interval = 20000u32.into();
    let _ = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.set_mint_interval(new_interval),
        )
        .submit()
        .await
        .expect("Calling `set_mint_interval` failed")
        .return_value();

    // 查询更新后的挖矿间隔
    let updated_interval = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.mint_interval(),
        )
        .dry_run()
        .await?
        .return_value();
    println!("updated_interval: {:?}", updated_interval);

    Ok(())
}

#[ink_e2e::test]
async fn test_charge<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 测试充值功能（payable）
    let _ = client
        .call(&ink_e2e::alice(), &cloud_call_builder.charge())
        .value(1000)
        .submit()
        .await
        .expect("Calling `charge` failed")
        .return_value();

    Ok(())
}

#[ink_e2e::test]
async fn test_balance<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 先充值
    let _ = client
        .call(&ink_e2e::alice(), &cloud_call_builder.charge())
        .value(1000)
        .submit()
        .await
        .expect("Calling `charge` failed")
        .return_value();

    // 查询原生代币余额
    let balance = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.balance(AssetInfo::Native(vec![0u8; 20])),
        )
        .dry_run()
        .await?
        .return_value();
    println!("native balance: {:?}", balance);

    Ok(())
}

#[ink_e2e::test]
async fn test_pod_report<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 查询不存在的 Pod 报告
    let report = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.pod_report(0),
        )
        .dry_run()
        .await?
        .return_value();
    println!("pod_report for non-existent pod: {:?}", report);

    Ok(())
}

#[ink_e2e::test]
async fn test_pod<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 查询不存在的 Pod
    let pod_info = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.pod(0),
        )
        .dry_run()
        .await?
        .return_value();
    println!("pod_info for non-existent pod: {:?}", pod_info);

    Ok(())
}

#[ink_e2e::test]
async fn test_pod_ext_info<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 查询不存在的 Pod 扩展信息
    let ext_info = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.pod_ext_info(0),
        )
        .dry_run()
        .await?
        .return_value();
    println!("pod_ext_info for non-existent pod: {:?}", ext_info);

    Ok(())
}

#[ink_e2e::test]
async fn test_pods_by_ids<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 查询多个 Pod ID（包括不存在的）
    let pods = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.pods_by_ids(vec![0, 1, 2]),
        )
        .dry_run()
        .await?
        .return_value();
    println!("pods_by_ids: {:?}", pods);

    Ok(())
}

#[ink_e2e::test]
async fn test_worker_pod_len<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 查询工作节点的 Pod 数量
    let worker_pod_len = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.worker_pod_len(0),
        )
        .dry_run()
        .await?
        .return_value();
    println!("worker_pod_len: {:?}", worker_pod_len);

    Ok(())
}

#[ink_e2e::test]
async fn test_worker_pods<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 查询工作节点的 Pod 列表
    let worker_pods = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.worker_pods(0, None, 100),
        )
        .dry_run()
        .await?
        .return_value();
    println!("worker_pods: {:?}", worker_pods);

    Ok(())
}

#[ink_e2e::test]
async fn test_worker_pods_version<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 查询工作节点的 Pod 版本信息
    let worker_pods_version = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.worker_pods_version(0),
        )
        .dry_run()
        .await?
        .return_value();
    println!("worker_pods_version: {:?}", worker_pods_version);

    Ok(())
}

#[ink_e2e::test]
async fn test_stop_pod<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 尝试停止不存在的 Pod（应该返回错误）
    let result = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.stop_pod(0),
        )
        .submit()
        .await;
    
    // 预期会失败，因为没有 Pod
    println!("stop_pod result: {:?}", result.is_err());

    Ok(())
}

#[ink_e2e::test]
async fn test_restart_pod<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 尝试重启不存在的 Pod（应该返回错误）
    let result = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.restart_pod(0),
        )
        .submit()
        .await;
    
    // 预期会失败，因为没有 Pod
    println!("restart_pod result: {:?}", result.is_err());

    Ok(())
}

#[ink_e2e::test]
async fn test_update_pod_contract<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 尝试更新不存在的 Pod 合约（应该返回错误）
    let result = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.update_pod_contract(0),
        )
        .submit()
        .await;
    
    // 预期会失败，因为没有 Pod
    println!("update_pod_contract result: {:?}", result.is_err());

    Ok(())
}

#[ink_e2e::test]
async fn test_edit_container<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 首先需要创建一个 pod 才能编辑容器
    // 注意：create_pod 需要 worker，但在测试环境中可能无法完全设置
    // 这里测试编辑容器的逻辑，先测试编辑不存在的 pod（应该返回错误）
    let container_input: Vec<ContainerInput> = vec![];
    
    let result = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.edit_container(0, container_input.clone()),
        )
        .submit()
        .await;
    
    // 预期会失败，因为没有 Pod
    assert!(result.is_err(), "编辑不存在的 pod 应该返回错误");
    println!("edit_container result (expected error): {:?}", result.is_err());

    // 如果有 pod，测试添加容器的逻辑
    // 注意：实际测试中，需要先创建 region、worker、然后创建 pod
    // 这里只是演示编辑容器的基本逻辑
    let new_container = Container::default();
    let container_input_insert: Vec<ContainerInput> = vec![ContainerInput {
        etype: EditType::INSERT,
        container: new_container,
    }];
    
    // 由于没有 pod，这个调用也会失败，但我们可以验证逻辑
    let result2 = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.edit_container(0, container_input_insert),
        )
        .submit()
        .await;
    
    assert!(result2.is_err(), "编辑不存在的 pod 应该返回错误");

    Ok(())
}

#[ink_e2e::test]
async fn test_user_secrets_query<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    let alice = AccountIdMapper::to_address(&ink_e2e::alice().public_key().as_ref());
    
    // 查询空的密钥列表
    let secrets = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.user_secrets(alice, None, 100),
        )
        .dry_run()
        .await?
        .return_value();
    println!("user_secrets (empty): {:?}", secrets);

    // 创建密钥后查询
    let secret_hash = H256::from([3u8; 32]);
    let _secret_id = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.create_secret("test_secret".as_bytes().to_vec(), secret_hash),
        )
        .submit()
        .await
        .expect("Calling `create_secret` failed")
        .return_value()
        .expect("Failed to get secret_id");

    let secrets_after = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.user_secrets(alice, None, 100),
        )
        .dry_run()
        .await?
        .return_value();
    println!("user_secrets (after create): {:?}", secrets_after);

    Ok(())
}

#[ink_e2e::test]
async fn test_user_disks_pagination<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    let alice = AccountIdMapper::to_address(&ink_e2e::alice().public_key().as_ref());
    
    // 创建多个磁盘
    for i in 0..5 {
        let _ = client
            .call(
                &ink_e2e::alice(),
                &cloud_call_builder.create_disk(format!("disk{}", i).as_bytes().to_vec(), 10 + i as u32),
            )
            .submit()
            .await
            .expect("Calling `create_disk` failed")
            .return_value();
    }

    // 使用分页查询
    let disks_page1 = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.user_disks(alice, None, 2),
        )
        .dry_run()
        .await?
        .return_value();
    println!("user_disks page 1 (size=2): {:?}", disks_page1);

    Ok(())
}

#[ink_e2e::test]
async fn test_pods_pagination<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 使用分页查询 Pod 列表
    let pods_page1 = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.pods(Some(0), 10),
        )
        .dry_run()
        .await?
        .return_value();
    println!("pods page 1: {:?}", pods_page1);

    Ok(())
}

#[ink_e2e::test]
async fn test_user_pods_pagination<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    setup_contracts!(client);
    
    // 使用分页查询用户 Pod 列表
    let user_pods_page1 = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.user_pods(Some(0), 10),
        )
        .dry_run()
        .await?
        .return_value();
    println!("user_pods page 1: {:?}", user_pods_page1);

    Ok(())
}
