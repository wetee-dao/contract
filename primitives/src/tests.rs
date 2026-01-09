use crate::{combine, split};

#[test]
fn test_combine_u32_to_u64() {
    let k1: u32 = 123456789;
    let k2: u32 = 987654321;

    let combined = combine(k1, k2);

    println!("Combined u64: {}", combined);

    let (a, b) = split(combined);

    assert_eq!(k1, a);
    assert_eq!(k2, b);
}

// 测试 define_map! 宏
#[cfg(test)]
mod map_tests {
    use super::*;
    use crate::define_map;

    define_map!(TestMap, u32, u32);

    #[ink::test]
    fn test_map_len() {
        let map = TestMap::default();
        assert_eq!(map.len(), 0);
    }

    #[ink::test]
    fn test_map_next_id() {
        let mut map = TestMap::default();
        assert_eq!(map.next_id(), 0);
    }

    #[ink::test]
    fn test_map_insert() {
        let mut map = TestMap::default();
        let key1 = map.insert(&100);
        assert_eq!(key1, 0);
        assert_eq!(map.len(), 1);

        let key2 = map.insert(&200);
        assert_eq!(key2, 1);
        assert_eq!(map.len(), 2);
    }

    #[ink::test]
    fn test_map_get() {
        let mut map = TestMap::default();
        let key = map.insert(&100);
        assert_eq!(map.get(key), Some(100));
        assert_eq!(map.get(key + 1), None);
    }

    #[ink::test]
    fn test_map_contains() {
        let mut map = TestMap::default();
        let key = map.insert(&100);
        assert!(map.contains(&key));
        assert!(!map.contains(&(key + 1)));
    }

    #[ink::test]
    fn test_map_update() {
        let mut map = TestMap::default();
        let key = map.insert(&100);
        assert_eq!(map.get(key), Some(100));

        map.update(key, &200);
        assert_eq!(map.get(key), Some(200));
    }

    #[ink::test]
    fn test_map_list() {
        let mut map = TestMap::default();
        map.insert(&10);
        map.insert(&20);
        map.insert(&30);

        let list = map.list(0, 2);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], (0, 10));
        assert_eq!(list[1], (1, 20));
    }

    #[ink::test]
    fn test_map_desc_list() {
        let mut map = TestMap::default();
        map.insert(&10);
        map.insert(&20);
        map.insert(&30);

        let list = map.desc_list(None, 2);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], (2, 30));
        assert_eq!(list[1], (1, 20));
    }

    #[ink::test]
    fn test_map_desc_list_with_start_key() {
        let mut map = TestMap::default();
        map.insert(&10);
        map.insert(&20);
        map.insert(&30);

        let list = map.desc_list(Some(2), 2);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], (2, 30));
        assert_eq!(list[1], (1, 20));
    }

    #[ink::test]
    fn test_map_list_empty() {
        let map = TestMap::default();
        let list = map.list(0, 10);
        assert_eq!(list.len(), 0);
    }

    #[ink::test]
    fn test_map_desc_list_empty() {
        let map = TestMap::default();
        let list = map.desc_list(None, 10);
        assert_eq!(list.len(), 0);
    }
}

// 测试 double_u32_map! 宏
#[cfg(test)]
mod double_map_tests {
    use super::*;
    use crate::double_u32_map;

    double_u32_map!(TestDoubleMap, u32, u32);

    #[ink::test]
    fn test_double_map_next_id() {
        let map = TestDoubleMap::default();
        assert_eq!(map.next_id(1), 0);
    }

    #[ink::test]
    fn test_double_map_len() {
        let map = TestDoubleMap::default();
        assert_eq!(map.len(1), 0);
    }

    #[ink::test]
    fn test_double_map_insert() {
        let mut map = TestDoubleMap::default();
        let k2_1 = map.insert(1, &100);
        assert_eq!(k2_1, 0);
        assert_eq!(map.len(1), 1);

        let k2_2 = map.insert(1, &200);
        assert_eq!(k2_2, 1);
        assert_eq!(map.len(1), 2);

        let k2_3 = map.insert(2, &300);
        assert_eq!(k2_3, 0);
        assert_eq!(map.len(2), 1);
    }

    #[ink::test]
    fn test_double_map_get() {
        let mut map = TestDoubleMap::default();
        let k2 = map.insert(1, &100);
        assert_eq!(map.get(1, k2), Some(100));
        assert_eq!(map.get(1, k2 + 1), None);
        assert_eq!(map.get(2, k2), None);
    }

    #[ink::test]
    fn test_double_map_update() {
        let mut map = TestDoubleMap::default();
        let k2 = map.insert(1, &100);
        assert_eq!(map.get(1, k2), Some(100));

        map.update(1, k2, &200);
        assert_eq!(map.get(1, k2), Some(200));
    }

    #[ink::test]
    fn test_double_map_update_nonexistent() {
        let mut map = TestDoubleMap::default();
        let result = map.update(1, 0, &100);
        assert_eq!(result, None);
    }

    #[ink::test]
    fn test_double_map_list() {
        let mut map = TestDoubleMap::default();
        map.insert(1, &10);
        map.insert(1, &20);
        map.insert(1, &30);

        let list = map.list(1, 0, 2);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], (0, 10));
        assert_eq!(list[1], (1, 20));
    }

    #[ink::test]
    fn test_double_map_desc_list() {
        let mut map = TestDoubleMap::default();
        map.insert(1, &10);
        map.insert(1, &20);
        map.insert(1, &30);

        let list = map.desc_list(1, None, 2);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], (2, 30));
        assert_eq!(list[1], (1, 20));
    }

    #[ink::test]
    fn test_double_map_desc_list_with_start_key() {
        let mut map = TestDoubleMap::default();
        map.insert(1, &10);
        map.insert(1, &20);
        map.insert(1, &30);

        let list = map.desc_list(1, Some(2), 2);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], (2, 30));
        assert_eq!(list[1], (1, 20));
    }

    #[ink::test]
    fn test_double_map_list_all() {
        let mut map = TestDoubleMap::default();
        map.insert(1, &10);
        map.insert(1, &20);
        map.insert(1, &30);

        let list = map.list_all(1);
        assert_eq!(list.len(), 3);
        assert_eq!(list[0], (0, 10));
        assert_eq!(list[1], (1, 20));
        assert_eq!(list[2], (2, 30));
    }

    #[ink::test]
    fn test_double_map_list_empty() {
        let map = TestDoubleMap::default();
        let list = map.list(1, 0, 10);
        assert_eq!(list.len(), 0);
    }

    #[ink::test]
    fn test_double_map_list_all_empty() {
        let map = TestDoubleMap::default();
        let list = map.list_all(1);
        assert_eq!(list.len(), 0);
    }

    #[ink::test]
    fn test_double_map_delete_by_key() {
        let mut map = TestDoubleMap::default();
        let k2 = map.insert(1, &100);
        assert_eq!(map.get(1, k2), Some(100));

        let result = map.delete_by_key(1, k2);
        assert!(result);
        assert_eq!(map.get(1, k2), None);
    }

    #[ink::test]
    fn test_double_map_delete_by_key_nonexistent() {
        let mut map = TestDoubleMap::default();
        let result = map.delete_by_key(1, 0);
        assert!(!result);
    }

    #[ink::test]
    fn test_double_map_multiple_k1() {
        let mut map = TestDoubleMap::default();
        map.insert(1, &10);
        map.insert(1, &20);
        map.insert(2, &30);
        map.insert(2, &40);

        assert_eq!(map.len(1), 2);
        assert_eq!(map.len(2), 2);

        let list1 = map.list_all(1);
        assert_eq!(list1.len(), 2);
        assert_eq!(list1[0], (0, 10));
        assert_eq!(list1[1], (1, 20));

        let list2 = map.list_all(2);
        assert_eq!(list2.len(), 2);
        assert_eq!(list2[0], (0, 30));
        assert_eq!(list2[1], (1, 40));
    }
}
