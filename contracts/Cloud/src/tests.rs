use ink::H256;

use super::cloud::*;
use crate::datas::{Command, Container, PodType, TEEType, CR};

fn init() -> Cloud {
    Cloud::new(H256::default(), H256::default())
}

#[ink::test]
fn user_pod() {
    let mut c = init();

    let p = c.create_user_pod(
        "pod1".as_bytes().to_vec(),
        PodType::CpuService,
        TEEType::SGX,
        vec![Container {
            name: "t1".as_bytes().to_vec(),
            image: "nginx".as_bytes().to_vec(),
            command: Command::NONE,
            port: Vec::new(),
            cr: CR {
                cpu: 1,
                mem: 1024,
                disk: Vec::new(),
                gpu: 0,
            },
        }],
        1,
        1,
        1,
    );
    assert!(p.is_ok());

    _ = c.create_user_pod(
        "pod2".as_bytes().to_vec(),
        PodType::CpuService,
        TEEType::SGX,
        Vec::new(),
        1,
        1,
        1,
    );

    let list = c.user_pods(1, 500);
    println!("list: {:?}", list);
}
