# ink! Subnet vs revive Subnet 合约功能对比

本文档对比 `contract/inks/Subnet`（ink! 版）与 `contract/revives/Subnet`（wrevive/PolkaVM 版）的接口与能力差异。

## 一、总体结论

| 维度 | ink! Subnet | revive Subnet |
|------|-------------|----------------|
| 定位 | 完整子网管理（Worker/Secret 注册、抵押、启停、Epoch 推进） | **已对齐**：完整实现与 ink 等价的 Worker/Secret/Validator/Epoch 逻辑 |
| 存储 | 完整（workers、secrets、regions、epoch、抵押、boot_nodes 等） | 与 ink 对齐的存储语义；runing/pending 使用 `Mapping<u32,(u64,u32)>`+长度 |
| 调用方 | 链上用户、Gov、侧链、其他合约 | 同上 |

revive 版已实现 ink 版的**全部 message**（除 `set_code` 因 pallet-revive 未暴露 set_code_hash 而恒返回 `Err(SetCodeFailed)`）。

---

## 二、构造函数

| 接口 | ink! | revive | 说明 |
|------|------|--------|------|
| `new()` | ✅ | ✅ | 行为一致：caller 为 gov，epoch_solt=72000，其余初始化为 0/空 |

---

## 三、治理与配置（仅 Gov 可写）

| 接口 | ink! | revive | 说明 |
|------|------|--------|------|
| `set_epoch_solt(epoch_solt)` | ✅ | ✅ | 一致 |
| `set_region(name)` | ✅ | ✅ | 一致（revive 用 `Bytes`） |
| `region(id)` | ✅ | ✅ | 一致 |
| `regions()` | ✅ | ✅ | 一致（revive 逆序列出，最多 1000 条） |
| `set_level_price(level, price)` | ✅ | ✅ | 一致 |
| `level_price(level)` | ✅ | ✅ | 一致 |
| `set_asset(info, price)` | ✅ | ✅ | 一致 |
| `asset(id)` | ✅ | ✅ | 一致 |
| `set_boot_nodes(nodes)` | ✅ | ✅ | 语义一致；ink 会 sort+dedup，revive 未做 |
| `boot_nodes()` | ✅ | ✅ | 一致（返回 `Vec<SecretNode>`） |
| `set_code(code_hash)` | ✅ 真正换码 | ⚠️ 固定返回 `Err(SetCodeFailed)` | pallet-revive 当前未暴露 set_code_hash，revive 仅占位 |
| `side_chain_key()` | ✅ | ✅ | 一致 |

---

## 四、Epoch 与验证者

| 接口 | ink! | revive | 说明 |
|------|------|--------|------|
| `epoch_info()` | ✅ | ✅ | 一致 |
| `set_next_epoch(_node_id)` | ✅ | ✅ | 一致：侧链推进 epoch、calc_new_validators |
| `next_epoch_validators()` | ✅ | ✅ | 一致 |
| `get_pending_secrets()` | ✅ | ✅ | 一致（`Vec<(u64, u32)>`） |
| `validators()` | ✅ | ✅ | 一致 |
| `validator_join(id)` | ✅ | ✅ | 一致 |
| `validator_delete(id)` | ✅ | ✅ | 一致 |

---

## 五、Worker 节点

| 接口 | ink! | revive | 说明 |
|------|------|--------|------|
| `worker(id)` | ✅ | ✅ | 一致 |
| `workers(start, size)` | ✅ | ✅ | 一致（降序分页） |
| `user_worker(user)` | ✅ | ✅ | 一致 |
| `mint_worker(account_id)` | ✅ | ✅ | 一致 |
| `worker_register(...)` | ✅ | ✅ | 一致 |
| `worker_update(id, name, ip, port)` | ✅ | ✅ | 一致 |
| `worker_mortgage(id, cpu, mem, ...)` | ✅ | ✅ | 一致 |
| `worker_unmortgage(worker_id, mortgage_id)` | ✅ | ✅ | 一致（通过 env().call 转账） |
| `worker_start(id)` | ✅ | ✅ | 一致 |
| `worker_stop(id)` | ✅ | ✅ | 一致 |

---

## 六、Secret 节点

| 接口 | ink! | revive | 说明 |
|------|------|--------|------|
| `secrets()` | ✅ | ✅ | 一致 |
| `secret_register(...)` | ✅ | ✅ | 一致（id==0 时加入 runing_secrets） |
| `secret_update(id, ...)` | ✅ | ✅ | 一致 |
| `secret_deposit(id, deposit)` | ✅ | ✅ | 一致 |
| `secret_delete(id)` | ✅ | ✅ | 一致 |

---

## 七、存储结构对比（概念）

ink 与 revive 在「键与类型」上对齐（如 gov_contract、epoch、regions、workers、secrets、boot_nodes、level_prices、asset_* 等），便于迁移与跨合约读取。  
revive 未实现的 message 所对应的**写入路径**（worker_register、secret_register、set_next_epoch 等）在 revive 版中不会被调用，因此相关存储项在 revive 场景下仅由「尚未实现的逻辑」或后续补全的 message 使用。

---

## 八、差异小结

1. **revive 已与 ink 对齐**  
   - 治理与配置、Region/Worker/Secret/Validator/Epoch 的读写与侧链推进均已实现。  
   - `set_boot_nodes` 与 ink 一致：sort + dedup。  
   - `worker_unmortgage` 通过 `env().call(..., value)` 实现转账。

2. **唯一差异**  
   - **set_code**：ink 可真正换码；revive 因 pallet-revive 未暴露 set_code_hash，恒返回 `Err(SetCodeFailed)`。

3. **存储细节**  
   - runing_secrets / pending_secrets 在 revive 中为 `Mapping<u32, (u64, u32)>` + `*_LEN`，便于整表替换与 Gov 维护验证者列表。
