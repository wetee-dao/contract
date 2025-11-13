use ink_e2e::ContractsBackend;
use subnet::{Subnet, SubnetRef};
use ink::primitives::AccountIdMapper;

use super::cloud::*;
use crate::datas::{Command, Container, PodType, TEEType};

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn test_create_user_pod<Client: E2EBackend>() -> E2EResult<()> {
    let pod_code = client
        .upload("pod", &ink_e2e::alice())
        .submit()
        .await
        .expect("pod upload failed");

    let subnet_contract = client
        .instantiate("subnet", &ink_e2e::alice(), &mut SubnetRef::new())
        .submit()
        .await
        .expect("subnet upload failed");

    let mut subnet_call_builder = subnet_contract.call_builder::<Subnet>();
    let _ = client
        .call(
            &ink_e2e::alice(),
            &subnet_call_builder.set_region("defalut".as_bytes().to_vec()),
        )
        .submit()
        .await
        .expect("Calling `set_region` failed")
        .return_value();

    let _ = client
        .call(
            &ink_e2e::alice(),
            &subnet_call_builder.worker_register(
                "worker0".as_bytes().to_vec(),
                ink::primitives::AccountId::from([0x01; 32]),
                subnet::datas::Ip {
                    ipv4: None,
                    ipv6: None,
                    domain: None,
                },
                0,
                1,
                1,
            ),
        )
        .submit()
        .await
        .expect("Calling `worker_register` failed")
        .return_value();

    let cloud_contract = client
        .instantiate(
            "cloud",
            &ink_e2e::alice(),
            &mut CloudRef::new(subnet_contract.addr, pod_code.code_hash),
        )
        .submit()
        .await
        .expect("cloud init failed");

    let mut cloud_call_builder = cloud_contract.call_builder::<Cloud>();

    let _ = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.create_pod(
                "pod1".as_bytes().to_vec(),
                PodType::CPU,
                TEEType::SGX,
                vec![Container {
                    image: "nginx".as_bytes().to_vec(),
                    command: Command::NONE,
                    port: Vec::new(),
                    cpu: 1,
                    mem: 1024,
                    disk: Vec::new(),
                    gpu: 0,
                    env: Vec::new(),
                }],
                1,
                1,
                0,
            ),
        )
        .submit()
        .await
        .expect("Calling `create_user_pod` failed")
        .return_value();

    let _ = client
        .call(
            &ink_e2e::alice(),
            &cloud_call_builder.create_pod(
                "pod2".as_bytes().to_vec(),
                PodType::CPU,
                TEEType::SGX,
                vec![Container {
                    image: "nginx".as_bytes().to_vec(),
                    command: Command::NONE,
                    port: Vec::new(),
                    cpu: 1,
                    mem: 1024,
                    disk: Vec::new(),
                    gpu: 0,
                    env: Vec::new(),
                }],
                1,
                1,
                0,
            ),
        )
        .submit()
        .await
        .expect("Calling `create_user_pod` failed")
        .return_value();

    let list = client
        .call(&ink_e2e::alice(), &cloud_call_builder.pods(None, 100))
        .dry_run()
        .await?
        .return_value();
    println!("list: {:?}", list);

    Ok(())
}

#[ink_e2e::test]
async fn test_delete_user_disk<Client: E2EBackend>() -> E2EResult<()> {
    let pod_code = client
        .upload("pod", &ink_e2e::alice())
        .submit()
        .await
        .expect("pod upload failed");

    let subnet_contract = client
        .instantiate("subnet", &ink_e2e::alice(), &mut SubnetRef::new())
        .submit()
        .await
        .expect("subnet upload failed");

    let cloud_contract = client
        .instantiate(
            "cloud",
            &ink_e2e::alice(),
            &mut CloudRef::new(subnet_contract.addr, pod_code.code_hash),
        )
        .submit()
        .await
        .expect("cloud init failed");

    let mut cloud_call_builder = cloud_contract.call_builder::<Cloud>();

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
