# Cloud 合约：ink vs polkadot（wrevive）差异对照

## 1. 构造函数

| 项目 | ink | polkadot |
|------|-----|----------|
| 参数 | `subnet_addr: Address`, `pod_contract_code_hash: H256` | `subnet_addr: Address`, `code_hash: [u8; 32]` |
| 返回 | `Self`（无 Result） | `Result<(), Error>` |

- **差异**：polkadot 的 code_hash 用 `[u8; 32]`，与 `H256` 内存布局一致，调用方需按需转换。

---

## 2. create_pod：Pod 实例化与 salt

| 项目 | ink | polkadot |
|------|-----|----------|
| 实例化 | `PodRef::new(...).endowment(pay_value).code_hash(...).salt_bytes(Some(u64_to_u8_32(pod_id))).instantiate()` | `env().instantiate(..., input_data, ...)`，无 salt |
| 充值 | endowment = `transferred_value` | value = `transferred`（通过 instantiate 的 value 参数） |

- **差异**：ink 用 **salt = u64_to_u8_32(pod_id)** 做确定性地址；当前 wrevive `instantiate` 未传 salt，同一 pod_id 在不同部署/链上得到的 Pod 地址可能不同。若需要与 ink 一致的可预测地址，需 wrevive 支持 instantiate 的 salt 参数。

---

## 3. 列表/分页类型

| 接口 | ink | polkadot |
|------|-----|----------|
| `user_pod_len` | 返回 `u32` | 返回 `u64` |
| `user_pods` | `start: Option<u32>`, `size: u32` | `start: Option<u64>`, `size: u64` |
| `worker_pods` | `start: Option<u64>`, `size: u64` | 同左 |
| `worker_pod_len` | `pods_of_worker.next_id(worker_id)`（等价长度） | `PODS_OF_WORKER.len(...)`，语义一致 |

- **差异**：ink 的 `pod_of_user` 为 double_u32_map，故 len/start/size 用 u32；polkadot 的 List2D 内层索引为 u64，故统一用 u64。若前端/调用方约定用 u32，需在调用侧做截断或校验。

---

## 4. pods() 分页语义

| 项目 | ink | polkadot |
|------|-----|----------|
| 实现 | `pods.desc_list(start, size)`，按**已存在的 pod_id** 降序 | 从 `total = NEXT_POD_ID` 起自 `start.unwrap_or(total-1)` 递减，对每个 `cur` 做 `PODS.get(cur)` |

- **差异**：ink 的 desc_list 通常只遍历“有数据的 key”；polkadot 按 pod_id 区间递减扫描，遇到未创建的 pod_id 会跳过，因此**有空洞时返回条数可能少于 size**。逻辑正确，但可能多扫若干次存储。

---

## 5. stop_pod：从 worker 列表移除

| 项目 | ink | polkadot |
|------|-----|----------|
| 实现 | `pods_of_worker.delete_by_key(worker_id, index.0)` | `PODS_OF_WORKER.clear(env(), &worker_id, k2)` |

- **差异**：ink 的 `delete_by_key` 可能真正删键并收缩结构；polkadot 的 List2D 使用 `clear`，只清空该 (k1, k2) 的 value，**不回收 k2 下标**，因此会留下“空洞”。后续 `list_all`/`desc_list` 若实现为“只遍历有值的槽”则行为接近；若按 k2 连续遍历可能看到空位，需依赖 List2D 的 list 实现是否过滤空值。

---

## 6. balance / transfer：ERC20

| 项目 | ink | polkadot |
|------|-----|----------|
| balance(AssetInfo::ERC20) | 使用 precompile `erc20(...).balanceOf(address)` | 固定返回 `U256::ZERO`（未接 ERC20） |
| transfer(AssetInfo::ERC20) | precompile `asset.transfer(to, amount)` | 直接返回 `Err(Error::PayFailed)` |

- **差异**：PolkaVM 当前无 ERC20 预编译，polkadot 侧**故意不实现** ERC20 分支，与“仅支持 Native”的假设一致。

---

## 7. set_code

| 项目 | ink | polkadot |
|------|-----|----------|
| 实现 | `self.env().set_code_hash(&code_hash)`，真正更新合约代码 | 固定 `Err(Error::SetCodeFailed)` |

- **差异**：pallet-revive / wrevive 若未暴露 `set_code_hash` 类 host API，polkadot 无法实现与 ink 相同的“治理升级合约代码”行为。

---

## 8. create_secret 返回值

| 项目 | ink | polkadot |
|------|-----|----------|
| 实现 | `Ok(self.user_secrets.insert(caller, &secret))`，insert 返回 id | `USER_SECRETS.insert(...).ok_or(Error::NotFound)` |

- **差异**：ink 的 insert 一般总返回 id；polkadot 的 List2D `insert` 返回 `Option<Ix>`，在溢出等情况下可能为 None，此时返回 `Error::NotFound`。语义上 polkadot 多了一种“创建失败”路径。

---

## 9. 其他一致或等价部分

- **update_pod_contract**：ink 调 `pod.contract.set_code(...)`，polkadot 用 `call_contract_raw(&pod.pod_address, set_code_selector, ...)`，语义一致。
- **start_pod**：`pod_key` 在 ink 为 `AccountId`（32 字节），polkadot 为 `[u8; 32]`，一致。
- **mint_pod**：费用用 U256 累加、`amount = pay_value * 1000 / price`，与 ink 一致（已改用 wrevive_api::U256 运算）。
- **restart_pod / pod_report / edit_container / pod / pod_ext_info / pods_by_ids**：逻辑与 ink 对齐。
- **user_secrets / secret / mint_secret / del_secret**：行为一致；del 在 polkadot 用 `clear` 代替 `delete_by_key`，见上文 stop_pod。
- **create_disk / update_disk_key / disk / user_disks / del_disk**：与 ink 一致；del_disk 同样用 `clear`。
- **charge**：仅接收 value，无逻辑差异。
- **ensure_from_gov / ensure_from_side_chain**：与 ink 一致。

---

## 10. 小结表

| 类别 | 差异项 | 影响 |
|------|--------|------|
| ABI/类型 | 构造函数返回 Result；code_hash 为 [u8;32]；user_pod_len 为 u64，user_pods 用 u64 分页 | 调用方需适配类型与错误处理 |
| 行为 | create_pod 无 salt，Pod 地址非按 pod_id 确定性 | 若需与 ink 同链同 pod_id 同地址，需 runtime 支持 salt |
| 行为 | pods() 有 pod_id 空洞时可能返回少于 size 条 | 可接受；或后续优化为“仅遍历已存在 pod” |
| 行为 | stop_pod / del_secret / del_disk 用 clear 留空洞 | 与 List2D 设计一致；若需紧凑列表需另做设计 |
| 能力 | balance/transfer 的 ERC20、set_code 未实现 | 受限于 PolkaVM/wrevive 当前能力，有意不做 |
| 错误 | create_secret 在 insert 返回 None 时返回 NotFound | 多一种失败情况，调用方需处理 |

如需与 ink 完全一致，可优先考虑：  
1）为 instantiate 增加 salt（若 runtime 支持）；  
2）将 user_pod_len / user_pods 的 u32 与 ink 对齐（或在 ABI 层做转换）；  
3）在文档中明确 ERC20、set_code、create_secret 的差异与限制。
