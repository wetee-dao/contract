package main

import (
	"crypto/rand"
	"flag"
	"fmt"
	"math/big"
	"os"
	"path/filepath"

	"wetee/test/contracts/cloud"
	"wetee/test/contracts/proxy"
	"wetee/test/contracts/subnet"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	chain "github.com/wetee-dao/ink.go"
	"github.com/wetee-dao/ink.go/pallet/revive"
	"github.com/wetee-dao/ink.go/pallet/system"
	"github.com/wetee-dao/ink.go/util"
	"github.com/wetee-dao/tee-dsecret/pkg/model"
)

func main() {
	var (
		chainURL string
		suri     string
		network  uint
		dir      string
	)

	flag.StringVar(&chainURL, "url", "", "blockchain websocket url")
	flag.StringVar(&suri, "suri", "//Alice", "signer secret uri")
	flag.UintVar(&network, "network", 42, "ss58 network id")
	flag.StringVar(&dir, "dir", ".", "workspace root directory (contains target/)")
	flag.Parse()

	if chainURL == "" {
		exitf("missing required flag: -url")
	}

	rootDir, err := filepath.Abs(dir)
	if err != nil {
		exitf("resolve dir: %v", err)
	}

	client, err := chain.InitClient([]string{chainURL}, true)
	if err != nil {
		exitf("init client: %v", err)
	}

	pk, err := chain.Sr25519PairFromSecret(suri, uint16(network))
	if err != nil {
		exitf("init signer: %v", err)
	}

	// show account info and ensure map account
	ensureMapAccount(client, pk)

	targetDir := filepath.Join(rootDir, "target")

	// Upload pod code
	podData, err := os.ReadFile(filepath.Join(targetDir, "pod.release.polkavm"))
	if err != nil {
		exitf("read pod code: %v", err)
	}
	podCodeHash, err := client.UploadInkCode(podData, &pk)
	if err != nil {
		exitf("upload pod code: %v", err)
	}

	// Deploy full system
	subnetImplAddress, _ := deploySubnetContract(client, pk, targetDir)
	cloudProxyAddress, _ := deployCloudContract(client, *subnetImplAddress, *podCodeHash, pk, targetDir)

	initSubnet(client, pk, subnetImplAddress.Hex())
	initWorker(client, pk, subnetImplAddress.Hex())

	fmt.Println("========================================")
	fmt.Println("subnet address (proxy2) => ", subnetImplAddress.Hex())
	fmt.Println("cloud  address (proxy1) => ", cloudProxyAddress.Hex())
	fmt.Println("========================================")
}

func deploySubnetContract(client *chain.ChainClient, pk chain.Signer, targetDir string) (*types.H160, *subnet.Subnet) {
	data, err := os.ReadFile(filepath.Join(targetDir, "subnet.release.polkavm"))
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

	proxyCode, err := os.ReadFile(filepath.Join(targetDir, "proxy.release.polkavm"))
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

func deployCloudContract(client *chain.ChainClient, subnetAddress types.H160, podCodeHash types.H256, pk chain.Signer, targetDir string) (*types.H160, *cloud.Cloud) {
	data, err := os.ReadFile(filepath.Join(targetDir, "cloud.release.polkavm"))
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

	proxyCode, err := os.ReadFile(filepath.Join(targetDir, "proxy.release.polkavm"))
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

func initSubnet(client *chain.ChainClient, pk chain.Signer, subnetAddress string) {
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

func initWorker(client *chain.ChainClient, pk chain.Signer, subnetAddress string) {
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

func ensureMapAccount(client *chain.ChainClient, pk chain.Signer) {
	ss58 := pk.SS58Address(42)
	h160 := pk.H160Address()

	// show account public key (SS58) and balance
	fmt.Println("Account SS58:", ss58)
	fmt.Println("Account H160:", h160.Hex())

	accountInfo, err := system.GetAccountLatest(client.Api().RPC.State, pk.AccountID())
	if err != nil {
		exitf("get account balance: %v", err)
	}
	fmt.Println("Account Free Balance:", accountInfo.Data.Free)

	// check account is mapped in revive
	_, isSome, err := revive.GetOriginalAccountLatest(client.Api().RPC.State, h160)
	if err != nil {
		exitf("get original account: %v", err)
	}
	if !isSome {
		runtimeCall := revive.MakeMapAccountCall()
		call, err := runtimeCall.AsCall()
		if err != nil {
			exitf("make map account call: %v", err)
		}

		fmt.Println("MakeMapAccount for", ss58)
		err = client.SignAndSubmit(&pk, call, true, 0)
		if err != nil {
			exitf("sign and submit map account: %v", err)
		}
		fmt.Println("MapAccount success")
	} else {
		fmt.Println("Account already mapped in revive")
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

func exitf(format string, args ...any) {
	fmt.Fprintf(os.Stderr, format+"\n", args...)
	os.Exit(1)
}
