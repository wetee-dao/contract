// use super::cloud::*;
// use crate::datas::*;
// use ink::env::test::default_accounts;
// use ink::Hash;

// fn init() -> Cloud {
//     let accounts = default_accounts();
//     let subnet_addr = accounts.alice;
//     let pod_code_hash = Hash::from([0u8; 32]);
    
//     Cloud::new(subnet_addr, pod_code_hash)
// }

// #[ink::test]
// fn test_new() {
//     let cloud = init();
//     let accounts = default_accounts();
    
//     assert_eq!(cloud.gov_contract(), accounts.alice);
// }

// #[ink::test]
// fn test_subnet_address() {
//     let cloud = init();
//     let accounts = default_accounts();
    
//     assert_eq!(cloud.subnet_address(), accounts.alice);
// }

// #[ink::test]
// fn test_pod_contract() {
//     let cloud = init();
//     let pod_hash = cloud.pod_contract();
//     assert_eq!(pod_hash, Hash::from([0u8; 32]));
// }

// #[ink::test]
// fn test_mint_interval() {
//     let cloud = init();
//     // 默认 mint_interval 是 14400
//     assert_eq!(cloud.mint_interval(), 14400u32.into());
// }

// #[ink::test]
// fn test_user_pods() {
//     let cloud = init();
//     let accounts = default_accounts();
    
//     let pods = cloud.user_pods(accounts.alice, None, 10);
//     // 初始状态应该没有 Pod
//     assert_eq!(pods.len(), 0);
// }

// #[ink::test]
// fn test_pod_status() {
//     let cloud = init();
    
//     // 测试不存在的 Pod
//     let status = cloud.pod_status(0);
//     assert_eq!(status, 0); // 默认状态
// }

// #[ink::test]
// fn test_pod_info() {
//     let cloud = init();
    
//     // 测试不存在的 Pod
//     let info = cloud.pod_info(0);
//     assert!(info.is_none());
// }

