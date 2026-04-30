package main

import (
	"crypto/rand"
	"flag"
	"fmt"
	"math/big"
	"os"
	"path/filepath"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	chain "github.com/wetee-dao/ink.go"
	"github.com/wetee-dao/ink.go/pallet/revive"
	"github.com/wetee-dao/ink.go/pallet/system"
	"github.com/wetee-dao/ink.go/util"
)

func main() {
	var (
		chainURL     string
		contractName string
		contractDir  string
		codePath     string
		suri         string
		network      uint
		debug        bool
	)

	flag.StringVar(&chainURL, "url", "", "blockchain websocket url, e.g. ws://127.0.0.1:9944")
	flag.StringVar(&contractName, "name", "", "contract name, e.g. cloud")
	flag.StringVar(&contractDir, "dir", ".", "contract workspace root directory")
	flag.StringVar(&codePath, "code", "", "compiled polkavm file path; defaults to <dir>/target/<name>.release.polkavm")
	flag.StringVar(&suri, "suri", "//Alice", "signer secret uri")
	flag.UintVar(&network, "network", 42, "ss58 network id")
	flag.BoolVar(&debug, "debug", true, "enable client debug logs")
	flag.Parse()

	if chainURL == "" {
		exitf("missing required flag: -url")
	}
	if contractName == "" {
		exitf("missing required flag: -name")
	}

	rootDir, err := filepath.Abs(contractDir)
	if err != nil {
		exitf("resolve dir: %v", err)
	}
	if codePath == "" {
		codePath = filepath.Join(rootDir, "target", contractName+".release.polkavm")
	}
	codePath, err = filepath.Abs(codePath)
	if err != nil {
		exitf("resolve code path: %v", err)
	}

	code, err := os.ReadFile(codePath)
	if err != nil {
		exitf("read contract code %s: %v", codePath, err)
	}

	client, err := chain.InitClient([]string{chainURL}, debug)
	if err != nil {
		exitf("init client: %v", err)
	}

	signer, err := chain.Sr25519PairFromSecret(suri, uint16(network))
	if err != nil {
		exitf("init signer: %v", err)
	}

	// show account info and ensure map account
	ensureMapAccount(client, signer)

	address, err := client.DeployContract(
		util.InkCode{Upload: &code},
		&signer,
		types.NewU128(*big.NewInt(0)),
		util.InkContractInput{
			Selector: "0x00000000",
			Args:     []any{},
		},
		util.NewSome(genSalt()),
	)
	if err != nil {
		exitf("deploy %s: %v", contractName, err)
	}

	fmt.Println("deploy success")
	fmt.Println("contract:", contractName)
	fmt.Println("code:", codePath)
	fmt.Println("chain:", chainURL)
	fmt.Println("address:", address.Hex())
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
	var salt [32]byte
	if _, err := rand.Read(salt[:]); err != nil {
		exitf("gen salt: %v", err)
	}
	return salt
}

func exitf(format string, args ...any) {
	fmt.Fprintf(os.Stderr, format+"\n", args...)
	os.Exit(1)
}
