package contracts

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"os"
	"testing"

	"wetee/test/contracts/cloud"
	"wetee/test/contracts/proxy"
	"wetee/test/contracts/subnet"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	chain "github.com/wetee-dao/ink.go"
	"github.com/wetee-dao/ink.go/pallet/revive"
	"github.com/wetee-dao/ink.go/util"
	"github.com/wetee-dao/tee-dsecret/pkg/model"
)

func TestContractInit(t *testing.T) {
	client, err := chain.InitClient([]string{TestChainUrl}, true)
	if err != nil {
		panic(err)
	}

	pk, err := chain.Sr25519PairFromSecret("//Alice", 42)
	if err != nil {
		util.LogWithPurple("Sr25519PairFromSecret", err)
		panic(err)
	}

	podData, err := os.ReadFile("../../target/pod.release.polkavm")
	if err != nil {
		util.LogWithPurple("read file error", err)
		panic(err)
	}
	podCodeHash, err := client.UploadInkCode(podData, &pk)
	if err != nil {
		util.LogWithPurple("UploadInkCode", err)
		panic(err)
	}

	subnetImplAddress, _ := DeploySubnetContract(client, pk)
	cloudProxyAddress, _ := DeployCloudContract(client, *subnetImplAddress, *podCodeHash, pk)

	InitSubnet(client, pk, subnetImplAddress.Hex())
	InitWorker(client, pk, subnetImplAddress.Hex())

	fmt.Println("subnet address (proxy2) => ", subnetImplAddress.Hex())
	fmt.Println("cloud  address (proxy1) => ", cloudProxyAddress.Hex())
}

func TestInitWorker(t *testing.T) {
	client, err := chain.InitClient([]string{TestChainUrl}, true)
	if err != nil {
		panic(err)
	}

	pk, err := chain.Sr25519PairFromSecret("//Alice", 42)
	if err != nil {
		util.LogWithPurple("Sr25519PairFromSecret", err)
		panic(err)
	}

	InitWorker(client, pk, SubnetAddress)
}

func TestCloudUpdate(t *testing.T) {
	client, err := chain.InitClient([]string{TestChainUrl}, true)
	if err != nil {
		panic(err)
	}

	pk, err := chain.Sr25519PairFromSecret("//Alice", 42)
	if err != nil {
		util.LogWithPurple("Sr25519PairFromSecret", err)
		panic(err)
	}

	/// init pod
	cloudData, err := os.ReadFile("../../target/cloud.release.polkavm")
	if err != nil {
		util.LogWithPurple("read file error", err)
		panic(err)
	}

	salt := genSalt()
	res, err := cloud.DeployCloudWithNew(chain.DeployParams{
		Client: client,
		Signer: &pk,
		Code:   util.InkCode{Upload: &cloudData},
		Salt:   util.NewSome(salt),
	})

	cloudIns, err := proxy.InitProxyContract(client, CloudAddress)
	if err != nil {
		util.LogWithPurple("InitCloudContract", err)
		panic(err)
	}

	err = cloudIns.ExecUpgrade(*res, chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		util.LogWithPurple("ExecUpgrade", err)
		panic(err)
	}

	fmt.Println("new cloud address: ", res.Hex())
	fmt.Println("proxy address: ", cloudIns.ContractAddress().Hex())
}

func TestSubnetUpdate(t *testing.T) {
	client, err := chain.InitClient([]string{TestChainUrl}, true)
	if err != nil {
		panic(err)
	}

	pk, err := chain.Sr25519PairFromSecret("//Alice", 42)
	if err != nil {
		util.LogWithPurple("Sr25519PairFromSecret", err)
		panic(err)
	}

	netData, err := os.ReadFile("../../target/subnet.release.polkavm")
	if err != nil {
		util.LogWithPurple("read file error", err)
		panic(err)
	}

	salt := genSalt()
	res, err := subnet.DeploySubnetWithNew(chain.DeployParams{
		Client: client,
		Signer: &pk,
		Code:   util.InkCode{Upload: &netData},
		Salt:   util.NewSome(salt),
	})
	if err != nil {
		util.LogWithPurple("DeploySubnetWithNew", err)
		panic(err)
	}

	subnetIns, err := proxy.InitProxyContract(client, SubnetAddress)
	if err != nil {
		util.LogWithPurple("InitSubnetContract", err)
		panic(err)
	}

	err = subnetIns.ExecUpgrade(*res, chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		util.LogWithPurple("ExecUpgrade", err)
		panic(err)
	}

	fmt.Println("new subnet address: ", res.Hex())
	fmt.Println("proxy address: ", subnetIns.ContractAddress().Hex())
}

func TestMapAccount(t *testing.T) {
	client, err := chain.InitClient([]string{TestChainUrl}, true)
	if err != nil {
		panic(err)
	}

	pk, err := chain.Sr25519PairFromSecret("//Alice", 42)
	if err != nil {
		util.LogWithPurple("Sr25519PairFromSecret", err)
		panic(err)
	}

	h160 := pk.H160Address()

	_, isSome, err := revive.GetOriginalAccountLatest(client.Api().RPC.State, h160)
	if err != nil {
		util.LogWithPurple("GetOriginalAccountLatest", err)
		panic(err)
	}
	if !isSome {
		runtimeCall := revive.MakeMapAccountCall()
		call, err := (runtimeCall).AsCall()
		if err != nil {
			panic(err)
		}

		err = client.SignAndSubmit(&pk, call, true, 0)
		if err != nil {
			panic(err)
		}
	}
}

func TestSetPrice(t *testing.T) {
	client, err := chain.InitClient([]string{TestChainUrl}, true)
	if err != nil {
		panic(err)
	}

	pk, err := chain.Sr25519PairFromSecret("//Alice", 42)
	if err != nil {
		util.LogWithPurple("Sr25519PairFromSecret", err)
		panic(err)
	}

	subnetIns, err := subnet.InitSubnetContract(client, SubnetAddress)
	if err != nil {
		util.LogWithPurple("InitCloudContract", err)
		panic(err)
	}

	err = subnetIns.ExecSetLevelPrice(1, subnet.RunPrice{
		CpuPer:       1,
		CvmCpuPer:    1,
		MemoryPer:    1,
		CvmMemoryPer: 1,
		DiskPer:      1,
		GpuPer:       1,
	}, chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		util.LogWithPurple("ExecSetLevelPrice", err)
		panic(err)
	}
}

func TestSetAssetPrice(t *testing.T) {
	client, err := chain.InitClient([]string{TestChainUrl}, true)
	if err != nil {
		panic(err)
	}

	pk, err := chain.Sr25519PairFromSecret("//Alice", 42)
	if err != nil {
		util.LogWithPurple("Sr25519PairFromSecret", err)
		panic(err)
	}

	subnetIns, err := subnet.InitSubnetContract(client, SubnetAddress)
	if err != nil {
		util.LogWithPurple("InitCloudContract", err)
		panic(err)
	}

	name := []byte("T")
	err = subnetIns.ExecSetAsset(subnet.AssetInfo{
		Native: &name,
	}, types.NewU256(*big.NewInt(1000)), chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})

	if err != nil {
		util.LogWithPurple("ExecSetAsset", err)
		panic(err)
	}
}

func DeploySubnetContract(client *chain.ChainClient, pk chain.Signer) (*types.H160, *subnet.Subnet) {
	data, err := os.ReadFile("../../target/subnet.release.polkavm")
	if err != nil {
		util.LogWithPurple("read file error", err)
		panic(err)
	}

	res, err := subnet.DeploySubnetWithNew(chain.DeployParams{
		Client: client,
		Signer: &pk,
		Code:   util.InkCode{Upload: &data},
		Salt:   util.NewSome(genSalt()),
	})
	if err != nil {
		util.LogWithPurple("DeployContract", err)
		panic(err)
	}
	fmt.Println("subnet address: ", res.Hex())

	proxyCode, err := os.ReadFile("../../target/proxy.release.polkavm")
	if err != nil {
		util.LogWithPurple("read proxy file error", err)
		panic(err)
	}
	subnetProxyAddress, err := proxy.DeployProxyWithNew(*res, util.NewSome(pk.H160Address()), chain.DeployParams{
		Client: client,
		Signer: &pk,
		Code:   util.InkCode{Upload: &proxyCode},
		Salt:   util.NewSome(genSalt()),
	})
	if err != nil {
		util.LogWithPurple("DeployProxyWithNew", err)
		panic(err)
	}
	fmt.Println("subnet proxy address: ", subnetProxyAddress.Hex())

	subnetContract, err := subnet.InitSubnetContract(client, subnetProxyAddress.Hex())
	if err != nil {
		panic(err)
	}

	err = subnetContract.ExecInit(chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		panic(err)
	}

	return subnetProxyAddress, subnetContract
}

func DeployCloudContract(client *chain.ChainClient, subnetAddress types.H160, podCodeHash types.H256, pk chain.Signer) (*types.H160, *cloud.Cloud) {
	data, err := os.ReadFile("../../target/cloud.release.polkavm")
	if err != nil {
		util.LogWithPurple("read file error", err)
		panic(err)
	}

	res, err := cloud.DeployCloudWithNew(chain.DeployParams{
		Client: client,
		Signer: &pk,
		Code:   util.InkCode{Upload: &data},
		Salt:   util.NewSome(genSalt()),
	})
	if err != nil {
		util.LogWithPurple("DeployContract", err)
		panic(err)
	}
	fmt.Println("cloud address: ", res.Hex())

	proxyCode, err := os.ReadFile("../../target/proxy.release.polkavm")
	if err != nil {
		util.LogWithPurple("read proxy file error", err)
		panic(err)
	}
	cloudProxyAddress, err := proxy.DeployProxyWithNew(*res, util.NewSome(pk.H160Address()), chain.DeployParams{
		Client: client,
		Signer: &pk,
		Code:   util.InkCode{Upload: &proxyCode},
		Salt:   util.NewSome(genSalt()),
	})
	if err != nil {
		util.LogWithPurple("DeployProxyWithNew", err)
		panic(err)
	}
	fmt.Println("cloud proxy address: ", cloudProxyAddress.Hex())

	cloudContract, err := cloud.InitCloudContract(client, cloudProxyAddress.Hex())
	if err != nil {
		panic(err)
	}
	err = cloudContract.ExecInit(subnetAddress, podCodeHash, chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		panic(err)
	}

	return cloudProxyAddress, cloudContract
}

func InitSubnet(client *chain.ChainClient, pk chain.Signer, subnetAddress string) {
	_call := chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	}
	subnetContract, err := subnet.InitSubnetContract(client, subnetAddress)
	if err != nil {
		panic(err)
	}

	v1, _ := model.PubKeyFromSS58("5G9jCxcTzRyQAwHKtcbXodVDXfeYTyeMdQxaP7hf1sNQhZ31")
	p1, _ := model.PubKeyFromSS58("5FKgVaCgpVKD8uC6z2bLFW23JUbQdYbFrX48po9NsZQmze5t")
	err = subnetContract.ExecSecretRegister(
		[]byte("node0"),
		v1.AccountID(),
		p1.AccountID(),
		subnet.Ip{
			Ipv4:   util.NewSome[uint32](3232263885),
			Ipv6:   util.NewNone[types.U128](),
			Domain: util.NewNone[[]byte](),
		},
		30110,
		_call,
	)
	fmt.Println("node0 register result:", err)

	v2, _ := model.PubKeyFromSS58("5GnjKDE6ArHPqaAwETR4TF7XZbjHU4pytXomt57jJrEBjP75")
	p2, _ := model.PubKeyFromSS58("5EwWfJzsZFs3coDWKjNSWJRsTgXGfYfwoDr6ZH1HufUMzWMs")
	err = subnetContract.ExecSecretRegister(
		[]byte("node1"),
		v2.AccountID(),
		p2.AccountID(),
		subnet.Ip{
			Ipv4:   util.NewSome[uint32](3232263885),
			Ipv6:   util.NewNone[types.U128](),
			Domain: util.NewNone[[]byte](),
		},
		30120,
		_call,
	)
	fmt.Println("node1 register result:", err)

	v3, _ := model.PubKeyFromSS58("5CQXegBto71RP1duknM8JWPkDZrPrqgxHutPjnvnpaz2qaRx")
	p3, _ := model.PubKeyFromSS58("5C8ynzqMj1D6a3vUbxds62Vp7iHFCr2Wpbffw6r2HbnWTN6D")
	err = subnetContract.ExecSecretRegister(
		[]byte("node2"),
		v3.AccountID(),
		p3.AccountID(),
		subnet.Ip{
			Ipv4:   util.NewSome[uint32](3232263885),
			Ipv6:   util.NewNone[types.U128](),
			Domain: util.NewNone[[]byte](),
		},
		30130,
		_call,
	)
	fmt.Println("node2 register result:", err)

	err = subnetContract.ExecSetBootNodes([]uint64{0, 1, 2}, _call)
	if err != nil {
		panic(err)
	}

	err = subnetContract.ExecValidatorJoin(1, _call)
	if err != nil {
		panic(err)
	}

	err = subnetContract.ExecValidatorJoin(2, _call)
	if err != nil {
		panic(err)
	}
}

func InitWorker(client *chain.ChainClient, pk chain.Signer, subnetAddress string) {
	_call := chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	}

	subnetContract, err := subnet.InitSubnetContract(client, subnetAddress)
	if err != nil {
		panic(err)
	}

	err = subnetContract.ExecSetRegion([]byte("defalut"), _call)
	if err != nil {
		panic(err)
	}

	err = subnetContract.ExecSetLevelPrice(0, subnet.RunPrice{
		CpuPer:       1,
		CvmCpuPer:    1,
		MemoryPer:    1,
		CvmMemoryPer: 1,
		DiskPer:      1,
		GpuPer:       1,
	}, _call)
	if err != nil {
		panic(err)
	}

	name := []byte("T")
	err = subnetContract.ExecSetAsset(subnet.AssetInfo{
		Native: &name,
	}, types.NewU256(*big.NewInt(1000)), chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		panic(err)
	}

	pubkey, _ := model.PubKeyFromSS58("5GSBfdb3PxME3XM4JrkFKAgHH77ADDWXUx6o8KGVmavLnZ44")
	err = subnetContract.ExecWorkerRegister(
		[]byte("worker0"),
		pubkey.AccountID(),
		subnet.Ip{
			Ipv4:   util.NewNone[uint32](),
			Ipv6:   util.NewNone[types.U128](),
			Domain: util.NewSome([]byte("xiaobai.asyou.me")),
		},
		10000,
		1,
		0,
		_call,
	)
	if err != nil {
		panic(err)
	}

	err = subnetContract.ExecWorkerMortgage(
		0,
		10000, 10000,
		0, 0,
		1000000,
		0,
		types.NewU256(*big.NewInt(10000000)),
		_call,
	)
	if err != nil {
		panic(err)
	}
}

func genSalt() [32]byte {
	bytes := make([]byte, 32)
	_, err := rand.Read(bytes)
	if err != nil {
		panic(err)
	}
	randomBytes := [32]byte{}
	copy(randomBytes[:], bytes)

	return randomBytes
}

func TestCloudQuerySecret(t *testing.T) {
	client, err := chain.InitClient([]string{TestChainUrl}, true)
	if err != nil {
		panic(err)
	}

	pk, err := chain.Sr25519PairFromSecret("//Alice", 42)
	if err != nil {
		util.LogWithPurple("Sr25519PairFromSecret", err)
		panic(err)
	}

	cloudContract, err := cloud.InitCloudContract(client, CloudAddress)
	if err != nil {
		panic(err)
	}

	fmt.Println(pk.SS58Address(42))
	fmt.Println(pk.H160Address().Hex())

	_call := chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	}
	err = cloudContract.ExecDelDisk(0, _call)
	if err != nil {
		panic(err)
	}
	fmt.Println("del disk:", err)

	// err = cloudContract.ExecInitDisk([]byte("node0"), 10, _call)
	// if err != nil {
	// 	panic(err)
	// }

	disk, _, err := cloudContract.QueryDisk(pk.H160Address(), 0, chain.DryRunParams{
		Origin:    pk.AccountID(),
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		panic(err)
	}
	fmt.Println("disk", disk)

	disks, _, err := cloudContract.QueryUserDisks(pk.H160Address(), util.NewNone[uint64](), 1000, chain.DryRunParams{
		Origin:    pk.AccountID(),
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		panic(err)
	}
	fmt.Println("disks:", disks)
}

func TestCloudUpdatePodContract(t *testing.T) {
	client, err := chain.InitClient([]string{TestChainUrl}, true)
	if err != nil {
		panic(err)
	}

	pk, err := chain.Sr25519PairFromSecret("//Alice", 42)
	if err != nil {
		util.LogWithPurple("Sr25519PairFromSecret", err)
		panic(err)
	}

	/// init pod
	podData, err := os.ReadFile("../../target/pod.release.polkavm")
	if err != nil {
		util.LogWithPurple("read file error", err)
		panic(err)
	}

	/// upload pod code
	podCode, err := client.UploadInkCode(podData, &pk)
	if err != nil {
		util.LogWithPurple("UploadInkCode", err)
		panic(err)
	}

	cloudContract, err := cloud.InitCloudContract(client, CloudAddress)
	if err != nil {
		panic(err)
	}

	err = cloudContract.ExecSetPodContract(*podCode, chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		panic(err)
	}

	err = cloudContract.ExecUpdatePodContract(0, chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		panic(err)
	}
}

func TestPodCharge(t *testing.T) {

}

func TestCloudTransfer(t *testing.T) {
	client, err := chain.InitClient([]string{TestChainUrl}, true)
	if err != nil {
		panic(err)
	}

	pk, err := chain.Sr25519PairFromSecret("//Alice", 42)
	if err != nil {
		util.LogWithPurple("Sr25519PairFromSecret", err)
		panic(err)
	}

	cloudContract, err := cloud.InitCloudContract(client, CloudAddress)
	if err != nil {
		panic(err)
	}

	tName := []byte("T")
	err = cloudContract.ExecTransfer(cloud.AssetInfo{
		Native: &tName,
	}, pk.H160Address(), types.NewU256(*big.NewInt(10001)), chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		panic(err)
	}
}
