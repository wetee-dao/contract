package subnet

import (
	"errors"
	"fmt"
	"math/big"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	chain "github.com/wetee-dao/ink.go"
	"github.com/wetee-dao/ink.go/util"
)

func DeploySubnetWithNew(__ink_params chain.DeployParams) (*types.H160, error) {
	return __ink_params.Client.DeployContract(
		__ink_params.Code, __ink_params.Signer, types.NewU128(*big.NewInt(0)),
		util.InkContractInput{
			Selector: "0x9bae9d5e",
			Args:     []any{},
		},
		__ink_params.Salt,
	)
}

func InitSubnetContract(client *chain.ChainClient, address string) (*Subnet, error) {
	contractAddress, err := util.HexToH160(address)
	if err != nil {
		return nil, err
	}
	return &Subnet{
		ChainClient: client,
		Address:     contractAddress,
	}, nil
}

type Subnet struct {
	ChainClient *chain.ChainClient
	Address     types.H160
}

func (c *Subnet) Client() *chain.ChainClient {
	return c.ChainClient
}

func (c *Subnet) ContractAddress() types.H160 {
	return c.Address
}

func (c *Subnet) QueryBootNodes(
	__ink_params chain.DryRunParams,
) (*util.Result[[]SecretNode, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "boot_nodes")
	}
	v, gas, err := chain.DryRunInk[util.Result[[]SecretNode, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x3fd8cc61",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) DryRunSetBootNodes(
	nodes []uint64, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "set_boot_nodes")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xe6b90091",
			Args:     []any{nodes},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecSetBootNodes(
	nodes []uint64, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSetBootNodes(nodes, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xe6b90091",
			Args:     []any{nodes},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfSetBootNodes(
	nodes []uint64, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSetBootNodes(nodes, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xe6b90091",
			Args:     []any{nodes},
		},
	)
}

func (c *Subnet) DryRunSetRegion(
	name []byte, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "set_region")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xb6993f90",
			Args:     []any{name},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecSetRegion(
	name []byte, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSetRegion(name, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xb6993f90",
			Args:     []any{name},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfSetRegion(
	name []byte, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSetRegion(name, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xb6993f90",
			Args:     []any{name},
		},
	)
}

func (c *Subnet) QueryRegion(
	id uint32, __ink_params chain.DryRunParams,
) (*util.Option[[]byte], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "region")
	}
	v, gas, err := chain.DryRunInk[util.Option[[]byte]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x69197ade",
			Args:     []any{id},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *Subnet) QueryWorker(
	id uint64, __ink_params chain.DryRunParams,
) (*util.Option[K8sCluster], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "worker")
	}
	v, gas, err := chain.DryRunInk[util.Option[K8sCluster]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xdfcf3455",
			Args:     []any{id},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *Subnet) QueryWorkers(
	start util.Option[uint64], size uint64, __ink_params chain.DryRunParams,
) (*[]Tuple_92, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "workers")
	}
	v, gas, err := chain.DryRunInk[[]Tuple_92](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xc9dfba3b",
			Args:     []any{start, size},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *Subnet) QueryUserWorker(
	user types.H160, __ink_params chain.DryRunParams,
) (*util.Option[Tuple_92], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "user_worker")
	}
	v, gas, err := chain.DryRunInk[util.Option[Tuple_92]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x936d50f4",
			Args:     []any{user},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *Subnet) QueryMintWorker(
	id util.AccountId, __ink_params chain.DryRunParams,
) (*util.Option[Tuple_92], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "mint_worker")
	}
	v, gas, err := chain.DryRunInk[util.Option[Tuple_92]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x59a206f7",
			Args:     []any{id},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *Subnet) DryRunWorkerRegister(
	name []byte, p2p_id util.AccountId, ip Ip, port uint32, level byte, region_id uint32, __ink_params chain.DryRunParams,
) (*util.Result[uint64, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "worker_register")
	}
	v, gas, err := chain.DryRunInk[util.Result[uint64, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xb90fc981",
			Args:     []any{name, p2p_id, ip, port, level, region_id},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecWorkerRegister(
	name []byte, p2p_id util.AccountId, ip Ip, port uint32, level byte, region_id uint32, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunWorkerRegister(name, p2p_id, ip, port, level, region_id, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xb90fc981",
			Args:     []any{name, p2p_id, ip, port, level, region_id},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfWorkerRegister(
	name []byte, p2p_id util.AccountId, ip Ip, port uint32, level byte, region_id uint32, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunWorkerRegister(name, p2p_id, ip, port, level, region_id, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xb90fc981",
			Args:     []any{name, p2p_id, ip, port, level, region_id},
		},
	)
}

func (c *Subnet) DryRunWorkerUpdate(
	id uint64, name []byte, ip Ip, port uint32, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "worker_update")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x79bdb96f",
			Args:     []any{id, name, ip, port},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecWorkerUpdate(
	id uint64, name []byte, ip Ip, port uint32, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunWorkerUpdate(id, name, ip, port, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x79bdb96f",
			Args:     []any{id, name, ip, port},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfWorkerUpdate(
	id uint64, name []byte, ip Ip, port uint32, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunWorkerUpdate(id, name, ip, port, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x79bdb96f",
			Args:     []any{id, name, ip, port},
		},
	)
}

func (c *Subnet) DryRunWorkerMortgage(
	id uint64, cpu uint32, mem uint32, cvm_cpu uint32, cvm_mem uint32, disk uint32, gpu uint32, deposit types.U256, __ink_params chain.DryRunParams,
) (*util.Result[uint32, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "worker_mortgage")
	}
	v, gas, err := chain.DryRunInk[util.Result[uint32, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xf70c3369",
			Args:     []any{id, cpu, mem, cvm_cpu, cvm_mem, disk, gpu, deposit},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecWorkerMortgage(
	id uint64, cpu uint32, mem uint32, cvm_cpu uint32, cvm_mem uint32, disk uint32, gpu uint32, deposit types.U256, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunWorkerMortgage(id, cpu, mem, cvm_cpu, cvm_mem, disk, gpu, deposit, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xf70c3369",
			Args:     []any{id, cpu, mem, cvm_cpu, cvm_mem, disk, gpu, deposit},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfWorkerMortgage(
	id uint64, cpu uint32, mem uint32, cvm_cpu uint32, cvm_mem uint32, disk uint32, gpu uint32, deposit types.U256, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunWorkerMortgage(id, cpu, mem, cvm_cpu, cvm_mem, disk, gpu, deposit, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xf70c3369",
			Args:     []any{id, cpu, mem, cvm_cpu, cvm_mem, disk, gpu, deposit},
		},
	)
}

func (c *Subnet) DryRunWorkerUnmortgage(
	worker_id uint64, mortgage_id uint32, __ink_params chain.DryRunParams,
) (*util.Result[uint32, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "worker_unmortgage")
	}
	v, gas, err := chain.DryRunInk[util.Result[uint32, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x6d25dbe9",
			Args:     []any{worker_id, mortgage_id},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecWorkerUnmortgage(
	worker_id uint64, mortgage_id uint32, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunWorkerUnmortgage(worker_id, mortgage_id, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x6d25dbe9",
			Args:     []any{worker_id, mortgage_id},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfWorkerUnmortgage(
	worker_id uint64, mortgage_id uint32, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunWorkerUnmortgage(worker_id, mortgage_id, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x6d25dbe9",
			Args:     []any{worker_id, mortgage_id},
		},
	)
}

func (c *Subnet) DryRunWorkerStart(
	id uint64, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "worker_start")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x6efe8ecf",
			Args:     []any{id},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecWorkerStart(
	id uint64, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunWorkerStart(id, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x6efe8ecf",
			Args:     []any{id},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfWorkerStart(
	id uint64, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunWorkerStart(id, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x6efe8ecf",
			Args:     []any{id},
		},
	)
}

func (c *Subnet) DryRunWorkerStop(
	id uint64, __ink_params chain.DryRunParams,
) (*util.Result[uint64, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "worker_stop")
	}
	v, gas, err := chain.DryRunInk[util.Result[uint64, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xeab3ba14",
			Args:     []any{id},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecWorkerStop(
	id uint64, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunWorkerStop(id, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xeab3ba14",
			Args:     []any{id},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfWorkerStop(
	id uint64, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunWorkerStop(id, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xeab3ba14",
			Args:     []any{id},
		},
	)
}

func (c *Subnet) QuerySecrets(
	__ink_params chain.DryRunParams,
) (*[]Tuple_101, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "secrets")
	}
	v, gas, err := chain.DryRunInk[[]Tuple_101](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xd91d379f",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *Subnet) DryRunSecretRegister(
	name []byte, validator_id util.AccountId, p2p_id util.AccountId, ip Ip, port uint32, __ink_params chain.DryRunParams,
) (*util.Result[uint64, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "secret_register")
	}
	v, gas, err := chain.DryRunInk[util.Result[uint64, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x55719146",
			Args:     []any{name, validator_id, p2p_id, ip, port},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecSecretRegister(
	name []byte, validator_id util.AccountId, p2p_id util.AccountId, ip Ip, port uint32, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSecretRegister(name, validator_id, p2p_id, ip, port, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x55719146",
			Args:     []any{name, validator_id, p2p_id, ip, port},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfSecretRegister(
	name []byte, validator_id util.AccountId, p2p_id util.AccountId, ip Ip, port uint32, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSecretRegister(name, validator_id, p2p_id, ip, port, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x55719146",
			Args:     []any{name, validator_id, p2p_id, ip, port},
		},
	)
}

func (c *Subnet) DryRunSecretUpdate(
	id uint64, name []byte, ip Ip, port uint32, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "secret_update")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x032511ea",
			Args:     []any{id, name, ip, port},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecSecretUpdate(
	id uint64, name []byte, ip Ip, port uint32, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSecretUpdate(id, name, ip, port, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x032511ea",
			Args:     []any{id, name, ip, port},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfSecretUpdate(
	id uint64, name []byte, ip Ip, port uint32, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSecretUpdate(id, name, ip, port, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x032511ea",
			Args:     []any{id, name, ip, port},
		},
	)
}

func (c *Subnet) DryRunSecretDeposit(
	id uint64, deposit types.U256, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "secret_deposit")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x4d815cac",
			Args:     []any{id, deposit},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecSecretDeposit(
	id uint64, deposit types.U256, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSecretDeposit(id, deposit, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x4d815cac",
			Args:     []any{id, deposit},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfSecretDeposit(
	id uint64, deposit types.U256, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSecretDeposit(id, deposit, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x4d815cac",
			Args:     []any{id, deposit},
		},
	)
}

func (c *Subnet) DryRunSecretDelete(
	id uint64, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "secret_delete")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x016716ab",
			Args:     []any{id},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecSecretDelete(
	id uint64, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSecretDelete(id, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x016716ab",
			Args:     []any{id},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfSecretDelete(
	id uint64, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSecretDelete(id, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x016716ab",
			Args:     []any{id},
		},
	)
}

func (c *Subnet) QueryValidators(
	__ink_params chain.DryRunParams,
) (*[]Tuple_104, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "validators")
	}
	v, gas, err := chain.DryRunInk[[]Tuple_104](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xcc64f718",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *Subnet) QueryGetPendingSecrets(
	__ink_params chain.DryRunParams,
) (*[]Tuple_63, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "get_pending_secrets")
	}
	v, gas, err := chain.DryRunInk[[]Tuple_63](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x8c922079",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *Subnet) DryRunValidatorJoin(
	id uint64, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "validator_join")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x3c10643c",
			Args:     []any{id},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecValidatorJoin(
	id uint64, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunValidatorJoin(id, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x3c10643c",
			Args:     []any{id},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfValidatorJoin(
	id uint64, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunValidatorJoin(id, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x3c10643c",
			Args:     []any{id},
		},
	)
}

func (c *Subnet) DryRunValidatorDelete(
	id uint64, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "validator_delete")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xdbac71a8",
			Args:     []any{id},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecValidatorDelete(
	id uint64, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunValidatorDelete(id, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xdbac71a8",
			Args:     []any{id},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfValidatorDelete(
	id uint64, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunValidatorDelete(id, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xdbac71a8",
			Args:     []any{id},
		},
	)
}

func (c *Subnet) QueryEpochInfo(
	__ink_params chain.DryRunParams,
) (*EpochInfo, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "epoch_info")
	}
	v, gas, err := chain.DryRunInk[EpochInfo](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xfd83a947",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *Subnet) DryRunSetEpochSolt(
	epoch_solt uint32, __ink_params chain.DryRunParams,
) (*util.NullTuple, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "set_epoch_solt")
	}
	v, gas, err := chain.DryRunInk[util.NullTuple](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x6f527ac8",
			Args:     []any{epoch_solt},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}

func (c *Subnet) ExecSetEpochSolt(
	epoch_solt uint32, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSetEpochSolt(epoch_solt, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x6f527ac8",
			Args:     []any{epoch_solt},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfSetEpochSolt(
	epoch_solt uint32, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSetEpochSolt(epoch_solt, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x6f527ac8",
			Args:     []any{epoch_solt},
		},
	)
}

func (c *Subnet) DryRunSetNextEpoch(
	_node_id uint64, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "set_next_epoch")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0xfc3bf295",
			Args:     []any{_node_id},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecSetNextEpoch(
	_node_id uint64, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSetNextEpoch(_node_id, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xfc3bf295",
			Args:     []any{_node_id},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfSetNextEpoch(
	_node_id uint64, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSetNextEpoch(_node_id, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0xfc3bf295",
			Args:     []any{_node_id},
		},
	)
}

func (c *Subnet) QueryNextEpochValidators(
	__ink_params chain.DryRunParams,
) (*util.Result[[]Tuple_104, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "next_epoch_validators")
	}
	v, gas, err := chain.DryRunInk[util.Result[[]Tuple_104, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x9f8ccaab",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) DryRunSetCode(
	code_hash types.H256, __ink_params chain.DryRunParams,
) (*util.Result[util.NullTuple, Error], *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "set_code")
	}
	v, gas, err := chain.DryRunInk[util.Result[util.NullTuple, Error]](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x694fb50f",
			Args:     []any{code_hash},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	if v != nil && v.IsErr {
		return nil, nil, errors.New("Contract Reverted: " + v.E.Error())
	}

	return v, gas, nil
}

func (c *Subnet) ExecSetCode(
	code_hash types.H256, __ink_params chain.ExecParams,
) error {
	_param := chain.DefaultParamWithOrigin(__ink_params.Signer.AccountID())
	_param.PayAmount = __ink_params.PayAmount
	_, gas, err := c.DryRunSetCode(code_hash, _param)
	if err != nil {
		return err
	}
	return chain.CallInk(
		c,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x694fb50f",
			Args:     []any{code_hash},
		},
		__ink_params,
	)
}

func (c *Subnet) CallOfSetCode(
	code_hash types.H256, __ink_params chain.DryRunParams,
) (*types.Call, error) {
	_, gas, err := c.DryRunSetCode(code_hash, __ink_params)
	if err != nil {
		return nil, err
	}
	return chain.CallOfTransaction(
		c,
		__ink_params.PayAmount,
		gas.GasRequired,
		gas.StorageDeposit,
		util.InkContractInput{
			Selector: "0x694fb50f",
			Args:     []any{code_hash},
		},
	)
}

func (c *Subnet) QuerySideChainKey(
	__ink_params chain.DryRunParams,
) (*types.H160, *chain.DryRunReturnGas, error) {
	if c.ChainClient.Debug {
		fmt.Println()
		util.LogWithPurple("[ DryRun   method ]", "side_chain_key")
	}
	v, gas, err := chain.DryRunInk[types.H160](
		c,
		__ink_params.Origin,
		__ink_params.PayAmount,
		__ink_params.GasLimit,
		__ink_params.StorageDepositLimit,
		util.InkContractInput{
			Selector: "0x6484abad",
			Args:     []any{},
		},
	)
	if err != nil && !errors.Is(err, chain.ErrContractReverted) {
		return nil, nil, err
	}
	return v, gas, nil
}
