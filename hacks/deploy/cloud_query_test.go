package contracts

import (
	"fmt"
	"math/big"
	"testing"
	"wetee/test/contracts/cloud"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	chain "github.com/wetee-dao/ink.go"
	"github.com/wetee-dao/ink.go/util"
)

// TestSubnetQuerySideChainKey tests querying side chain key from subnet contract
func TestCloudQuerySecrets(t *testing.T) {
	client, err := chain.InitClient([]string{TestChainUrl}, true)
	if err != nil {
		panic(err)
	}

	pk, err := chain.Sr25519PairFromSecret("//Alice", 42)
	if err != nil {
		util.LogWithPurple("Sr25519PairFromSecret", err)
		panic(err)
	}

	cloudIns, err := cloud.InitCloudContract(client, CloudAddress)
	if err != nil {
		util.LogWithPurple("InitCloudContract", err)
		panic(err)
	}

	err = cloudIns.ExecCreateSecret([]byte("test"), types.H256{}, chain.ExecParams{
		Signer:    &pk,
		PayAmount: types.NewU128(*big.NewInt(0)),
	})
	if err != nil {
		util.LogWithPurple("ExecCreateSecret", err)
		panic(err)
	}

	secrets, _, err := cloudIns.QueryUserSecrets(
		pk.H160Address(),
		util.NewNone[uint64](),
		100,
		chain.DryRunParams{
			Origin:    pk.AccountID(),
			PayAmount: types.NewU128(*big.NewInt(0)),
		},
	)
	if err != nil {
		util.LogWithPurple("QueryUserSecrets", err)
		panic(err)
	}

	fmt.Printf("Secrets: %+v\n", secrets)
}
