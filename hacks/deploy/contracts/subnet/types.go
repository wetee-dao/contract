package subnet

import (
	"fmt"

	"github.com/centrifuge/go-substrate-rpc-client/v4/scale"
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/wetee-dao/ink.go/util"
)

type K8sCluster struct { // Composite
	Name          []byte
	Owner         types.H160
	Level         byte
	RegionId      uint32
	StartBlock    uint32
	StopBlock     util.Option[uint32]
	TerminalBlock util.Option[uint32]
	P2pId         util.AccountId
	Ip            Ip
	Port          uint32
	Status        byte
}
type Ip struct { // Composite
	Ipv4   util.Option[uint32]
	Ipv6   util.Option[types.U128]
	Domain util.Option[[]byte]
}
type SecretNode struct { // Composite
	Name          []byte
	Owner         types.H160
	ValidatorId   util.AccountId
	P2pId         util.AccountId
	StartBlock    uint32
	TerminalBlock util.Option[uint32]
	Ip            Ip
	Port          uint32
	Status        byte
}
type Tuple_63 struct { // Tuple
	F0 uint64
	F1 uint32
}
type AssetInfo struct { // Enum
	Native *[]byte   // 0
	ERC20  *struct { // 1
		F0 []byte
		F1 uint32
	}
}

func (ty AssetInfo) Encode(encoder scale.Encoder) (err error) {
	if ty.Native != nil {
		err = encoder.PushByte(0)
		if err != nil {
			return err
		}
		err = encoder.Encode(*ty.Native)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.ERC20 != nil {
		err = encoder.PushByte(1)
		if err != nil {
			return err
		}

		err = encoder.Encode(ty.ERC20.F0)
		if err != nil {
			return err
		}

		err = encoder.Encode(ty.ERC20.F1)
		if err != nil {
			return err
		}

		return nil
	}
	return fmt.Errorf("unrecognized enum")
}

func (ty *AssetInfo) Decode(decoder scale.Decoder) (err error) {
	variant, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}
	switch variant {
	case 0: // Inline
		ty.Native = new([]byte)
		err = decoder.Decode(ty.Native)
		if err != nil {
			return err
		}
		return
	case 1: // Tuple
		ty.ERC20 = &struct {
			F0 []byte
			F1 uint32
		}{}

		err = decoder.Decode(&ty.ERC20.F0)
		if err != nil {
			return err
		}

		err = decoder.Decode(&ty.ERC20.F1)
		if err != nil {
			return err
		}

		return
	default:
		return fmt.Errorf("unrecognized enum")
	}
}

type RunPrice struct { // Composite
	CpuPer       uint64
	CvmCpuPer    uint64
	MemoryPer    uint64
	CvmMemoryPer uint64
	DiskPer      uint64
	GpuPer       uint64
}
type Error struct { // Enum
	NotEnoughBalance          *bool // 0
	MustCallByMainContract    *bool // 1
	WorkerNotExist            *bool // 2
	WorkerNotOwnedByCaller    *bool // 3
	WorkerStatusNotReady      *bool // 4
	WorkerMortgageNotExist    *bool // 5
	TransferFailed            *bool // 6
	WorkerIsUseByUser         *bool // 7
	NodeNotExist              *bool // 8
	SecretNodeAlreadyExists   *bool // 9
	SetCodeFailed             *bool // 10
	EpochNotExpired           *bool // 11
	InvalidSideChainSignature *bool // 12
	NodeIsRunning             *bool // 13
	InvalidSideChainCaller    *bool // 14
	RegionNotExist            *bool // 15
}

func (ty Error) Encode(encoder scale.Encoder) (err error) {
	if ty.NotEnoughBalance != nil {
		err = encoder.PushByte(0)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.MustCallByMainContract != nil {
		err = encoder.PushByte(1)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.WorkerNotExist != nil {
		err = encoder.PushByte(2)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.WorkerNotOwnedByCaller != nil {
		err = encoder.PushByte(3)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.WorkerStatusNotReady != nil {
		err = encoder.PushByte(4)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.WorkerMortgageNotExist != nil {
		err = encoder.PushByte(5)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.TransferFailed != nil {
		err = encoder.PushByte(6)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.WorkerIsUseByUser != nil {
		err = encoder.PushByte(7)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.NodeNotExist != nil {
		err = encoder.PushByte(8)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.SecretNodeAlreadyExists != nil {
		err = encoder.PushByte(9)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.SetCodeFailed != nil {
		err = encoder.PushByte(10)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.EpochNotExpired != nil {
		err = encoder.PushByte(11)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.InvalidSideChainSignature != nil {
		err = encoder.PushByte(12)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.NodeIsRunning != nil {
		err = encoder.PushByte(13)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.InvalidSideChainCaller != nil {
		err = encoder.PushByte(14)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.RegionNotExist != nil {
		err = encoder.PushByte(15)
		if err != nil {
			return err
		}
		return nil
	}
	return fmt.Errorf("unrecognized enum")
}

func (ty *Error) Decode(decoder scale.Decoder) (err error) {
	variant, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}
	switch variant {
	case 0: // Base
		t := true
		ty.NotEnoughBalance = &t
		return
	case 1: // Base
		t := true
		ty.MustCallByMainContract = &t
		return
	case 2: // Base
		t := true
		ty.WorkerNotExist = &t
		return
	case 3: // Base
		t := true
		ty.WorkerNotOwnedByCaller = &t
		return
	case 4: // Base
		t := true
		ty.WorkerStatusNotReady = &t
		return
	case 5: // Base
		t := true
		ty.WorkerMortgageNotExist = &t
		return
	case 6: // Base
		t := true
		ty.TransferFailed = &t
		return
	case 7: // Base
		t := true
		ty.WorkerIsUseByUser = &t
		return
	case 8: // Base
		t := true
		ty.NodeNotExist = &t
		return
	case 9: // Base
		t := true
		ty.SecretNodeAlreadyExists = &t
		return
	case 10: // Base
		t := true
		ty.SetCodeFailed = &t
		return
	case 11: // Base
		t := true
		ty.EpochNotExpired = &t
		return
	case 12: // Base
		t := true
		ty.InvalidSideChainSignature = &t
		return
	case 13: // Base
		t := true
		ty.NodeIsRunning = &t
		return
	case 14: // Base
		t := true
		ty.InvalidSideChainCaller = &t
		return
	case 15: // Base
		t := true
		ty.RegionNotExist = &t
		return
	default:
		return fmt.Errorf("unrecognized enum")
	}
}
func (ty *Error) Error() string {
	if ty.NotEnoughBalance != nil {
		return "NotEnoughBalance"
	}

	if ty.MustCallByMainContract != nil {
		return "MustCallByMainContract"
	}

	if ty.WorkerNotExist != nil {
		return "WorkerNotExist"
	}

	if ty.WorkerNotOwnedByCaller != nil {
		return "WorkerNotOwnedByCaller"
	}

	if ty.WorkerStatusNotReady != nil {
		return "WorkerStatusNotReady"
	}

	if ty.WorkerMortgageNotExist != nil {
		return "WorkerMortgageNotExist"
	}

	if ty.TransferFailed != nil {
		return "TransferFailed"
	}

	if ty.WorkerIsUseByUser != nil {
		return "WorkerIsUseByUser"
	}

	if ty.NodeNotExist != nil {
		return "NodeNotExist"
	}

	if ty.SecretNodeAlreadyExists != nil {
		return "SecretNodeAlreadyExists"
	}

	if ty.SetCodeFailed != nil {
		return "SetCodeFailed"
	}

	if ty.EpochNotExpired != nil {
		return "EpochNotExpired"
	}

	if ty.InvalidSideChainSignature != nil {
		return "InvalidSideChainSignature"
	}

	if ty.NodeIsRunning != nil {
		return "NodeIsRunning"
	}

	if ty.InvalidSideChainCaller != nil {
		return "InvalidSideChainCaller"
	}

	if ty.RegionNotExist != nil {
		return "RegionNotExist"
	}
	return "Unknown"
}

type Tuple_99 struct { // Tuple
	F0 AssetInfo
	F1 types.U256
}
type Tuple_105 struct { // Tuple
	F0 uint64
	F1 K8sCluster
}
type Tuple_114 struct { // Tuple
	F0 uint64
	F1 SecretNode
}
type Tuple_117 struct { // Tuple
	F0 uint64
	F1 SecretNode
	F2 uint32
}
type EpochInfo struct { // Composite
	Epoch          uint32
	EpochSolt      uint32
	LastEpochBlock uint32
	Now            uint32
	SideChainPub   types.H160
}
