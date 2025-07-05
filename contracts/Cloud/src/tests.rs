use super::cloud::*;
use crate::datas::{Command, Container, PodType, TEEType, CR};

fn init() -> Cloud {
    Cloud::new()
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
    );
    assert!(p.is_ok());

    _ = c.create_user_pod(
        "pod2".as_bytes().to_vec(),
        PodType::CpuService,
        TEEType::SGX,
        Vec::new(),
    );

    let list = c.user_pods(1, 500);
    println!("list: {:?}", list);
}
