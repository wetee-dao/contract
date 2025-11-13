package contracts

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"os"
	"testing"

	"wetee/test/contracts/cloud"
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

	/// init pod
	podData, err := os.ReadFile("../../target/ink/pod/pod.polkavm")
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

	/// init subnet
	subnetAddress := DeploySubnetContract(client, pk)

	/// init cloud
	cloudCode, err := os.ReadFile("../../target/ink/cloud/cloud.polkavm")
	if err != nil {
		util.LogWithPurple("read file error", err)
		panic(err)
	}

	salt := genSalt()
	cloudAddress, err := cloud.DeployCloudWithNew(*subnetAddress, *podCode, chain.DeployParams{
		Client: client,
		Signer: &pk,
		Code:   util.InkCode{Upload: &cloudCode},
		Salt:   util.NewSome(salt),
	})

	if err != nil {
		util.LogWithPurple("DeployContract", err)
		panic(err)
	}

	InitSubnet(client, pk, subnetAddress.Hex())
	InitWorker(client, pk, subnetAddress.Hex())
	fmt.Println("subnet address ======> ", subnetAddress.Hex())
	fmt.Println("cloud  address ======> ", cloudAddress.Hex())
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
	cloudData, err := os.ReadFile("../../target/ink/cloud/cloud.polkavm")
	if err != nil {
		util.LogWithPurple("read file error", err)
		panic(err)
	}

	code, err := client.UploadInkCode(cloudData, &pk)
	if err != nil {
		util.LogWithPurple("UploadInkCode", err)
		panic(err)
	}

	cloudIns, err := cloud.InitCloudContract(client, CloudAddress)
	if err != nil {
		util.LogWithPurple("InitCloudContract", err)
		panic(err)
	}

	err = cloudIns.ExecSetCode(*code, chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		util.LogWithPurple("ExecSetCode", err)
	}
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

	/// init pod
	netData, err := os.ReadFile("../../target/ink/subnet/subnet.polkavm")
	if err != nil {
		util.LogWithPurple("read file error", err)
		panic(err)
	}

	netCode, err := client.UploadInkCode(netData, &pk)
	if err != nil {
		util.LogWithPurple("UploadInkCode", err)
		panic(err)
	}

	fmt.Println("cloudAddress: ", CloudAddress)

	subnetIns, err := subnet.InitSubnetContract(client, SubnetAddress)
	if err != nil {
		util.LogWithPurple("InitCloudContract", err)
		panic(err)
	}

	err = subnetIns.ExecSetCode(*netCode, chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})

	if err != nil {
		util.LogWithPurple("subnet ExecSetCode", err)
	}
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

func DeploySubnetContract(client *chain.ChainClient, pk chain.Signer) *types.H160 {
	data, err := os.ReadFile("../../target/ink/subnet/subnet.polkavm")
	if err != nil {
		util.LogWithPurple("read file error", err)
		panic(err)
	}

	salt := genSalt()
	res, err := subnet.DeploySubnetWithNew(chain.DeployParams{
		Client: client,
		Signer: &pk,
		Code:   util.InkCode{Upload: &data},
		Salt:   util.NewSome(salt),
	})

	if err != nil {
		util.LogWithPurple("DeployContract", err)
		panic(err)
	}

	return res
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

	v1, _ := model.PubKeyFromSS58("5CdERUzLMFh5D8RB82bd6t4nuqKJLdNr6ZQ9NAsoQqVMyz5B")
	p1, _ := model.PubKeyFromSS58("5CAG6XhZY5Q3seRa4BwDhSQGFHqoA4H2m3GJKew7xArJwcNJ")
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

	v2, _ := model.PubKeyFromSS58("5Fk6tyXKk9HmATcSvtcEjMHsyfn2e49H76qP72yFXzUU4ws6")
	p2, _ := model.PubKeyFromSS58("5GuRb3N6Qraej2S3kQNX33UMnk47saYTAH4EBGzPiuqG8kni")
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

	v3, _ := model.PubKeyFromSS58("5CK7kDvy6svMswxifABZAu8GFrcAvEw1z9nt7Wuuvh8YMzx1")
	p3, _ := model.PubKeyFromSS58("5FgmV7fM5yAyZK5DfbAv3x9CrSBcnNt3Zykbxs9S9HHrvbeG")
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

	subnetContract.ExecSetBootNodes([]uint64{0, 1, 2}, _call)
	subnetContract.ExecValidatorJoin(1, _call)
	subnetContract.ExecValidatorJoin(2, _call)
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
