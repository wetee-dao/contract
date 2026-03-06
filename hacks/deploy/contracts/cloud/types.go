package cloud

import (
	"fmt"

	"github.com/centrifuge/go-substrate-rpc-client/v4/scale"
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/wetee-dao/ink.go/util"
)

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

type AssetInfo struct { // Enum
	Native *[]byte   // 0
	ERC20  *struct { // 1
		F0 []byte
		F1 types.H256
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
			F1 types.H256
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

type PodType struct { // Enum
	CPU    *bool // 0
	GPU    *bool // 1
	SCRIPT *bool // 2
}

func (ty PodType) Encode(encoder scale.Encoder) (err error) {
	if ty.CPU != nil {
		err = encoder.PushByte(0)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.GPU != nil {
		err = encoder.PushByte(1)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.SCRIPT != nil {
		err = encoder.PushByte(2)
		if err != nil {
			return err
		}
		return nil
	}
	return fmt.Errorf("unrecognized enum")
}

func (ty *PodType) Decode(decoder scale.Decoder) (err error) {
	variant, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}
	switch variant {
	case 0: // Base
		t := true
		ty.CPU = &t
		return
	case 1: // Base
		t := true
		ty.GPU = &t
		return
	case 2: // Base
		t := true
		ty.SCRIPT = &t
		return
	default:
		return fmt.Errorf("unrecognized enum")
	}
}

type TEEType struct { // Enum
	SGX *bool // 0
	CVM *bool // 1
}

func (ty TEEType) Encode(encoder scale.Encoder) (err error) {
	if ty.SGX != nil {
		err = encoder.PushByte(0)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.CVM != nil {
		err = encoder.PushByte(1)
		if err != nil {
			return err
		}
		return nil
	}
	return fmt.Errorf("unrecognized enum")
}

func (ty *TEEType) Decode(decoder scale.Decoder) (err error) {
	variant, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}
	switch variant {
	case 0: // Base
		t := true
		ty.SGX = &t
		return
	case 1: // Base
		t := true
		ty.CVM = &t
		return
	default:
		return fmt.Errorf("unrecognized enum")
	}
}

type Pod struct { // Composite
	Name       []byte
	Owner      types.H160
	PodAddress types.H160
	Ptype      PodType
	StartBlock uint32
	TeeType    TEEType
	Level      byte
	PayAssetId uint32
}
type Command struct { // Enum
	SH   *[]byte // 0
	BASH *[]byte // 1
	ZSH  *[]byte // 2
	NONE *bool   // 3
}

func (ty Command) Encode(encoder scale.Encoder) (err error) {
	if ty.SH != nil {
		err = encoder.PushByte(0)
		if err != nil {
			return err
		}
		err = encoder.Encode(*ty.SH)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.BASH != nil {
		err = encoder.PushByte(1)
		if err != nil {
			return err
		}
		err = encoder.Encode(*ty.BASH)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.ZSH != nil {
		err = encoder.PushByte(2)
		if err != nil {
			return err
		}
		err = encoder.Encode(*ty.ZSH)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.NONE != nil {
		err = encoder.PushByte(3)
		if err != nil {
			return err
		}
		return nil
	}
	return fmt.Errorf("unrecognized enum")
}

func (ty *Command) Decode(decoder scale.Decoder) (err error) {
	variant, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}
	switch variant {
	case 0: // Inline
		ty.SH = new([]byte)
		err = decoder.Decode(ty.SH)
		if err != nil {
			return err
		}
		return
	case 1: // Inline
		ty.BASH = new([]byte)
		err = decoder.Decode(ty.BASH)
		if err != nil {
			return err
		}
		return
	case 2: // Inline
		ty.ZSH = new([]byte)
		err = decoder.Decode(ty.ZSH)
		if err != nil {
			return err
		}
		return
	case 3: // Base
		t := true
		ty.NONE = &t
		return
	default:
		return fmt.Errorf("unrecognized enum")
	}
}

type Service struct { // Enum
	Tcp        *uint16 // 0
	Udp        *uint16 // 1
	Http       *uint16 // 2
	Https      *uint16 // 3
	ProjectTcp *uint16 // 4
	ProjectUdp *uint16 // 5
}

func (ty Service) Encode(encoder scale.Encoder) (err error) {
	if ty.Tcp != nil {
		err = encoder.PushByte(0)
		if err != nil {
			return err
		}
		err = encoder.Encode(*ty.Tcp)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.Udp != nil {
		err = encoder.PushByte(1)
		if err != nil {
			return err
		}
		err = encoder.Encode(*ty.Udp)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.Http != nil {
		err = encoder.PushByte(2)
		if err != nil {
			return err
		}
		err = encoder.Encode(*ty.Http)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.Https != nil {
		err = encoder.PushByte(3)
		if err != nil {
			return err
		}
		err = encoder.Encode(*ty.Https)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.ProjectTcp != nil {
		err = encoder.PushByte(4)
		if err != nil {
			return err
		}
		err = encoder.Encode(*ty.ProjectTcp)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.ProjectUdp != nil {
		err = encoder.PushByte(5)
		if err != nil {
			return err
		}
		err = encoder.Encode(*ty.ProjectUdp)
		if err != nil {
			return err
		}
		return nil
	}
	return fmt.Errorf("unrecognized enum")
}

func (ty *Service) Decode(decoder scale.Decoder) (err error) {
	variant, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}
	switch variant {
	case 0: // Inline
		ty.Tcp = new(uint16)
		err = decoder.Decode(ty.Tcp)
		if err != nil {
			return err
		}
		return
	case 1: // Inline
		ty.Udp = new(uint16)
		err = decoder.Decode(ty.Udp)
		if err != nil {
			return err
		}
		return
	case 2: // Inline
		ty.Http = new(uint16)
		err = decoder.Decode(ty.Http)
		if err != nil {
			return err
		}
		return
	case 3: // Inline
		ty.Https = new(uint16)
		err = decoder.Decode(ty.Https)
		if err != nil {
			return err
		}
		return
	case 4: // Inline
		ty.ProjectTcp = new(uint16)
		err = decoder.Decode(ty.ProjectTcp)
		if err != nil {
			return err
		}
		return
	case 5: // Inline
		ty.ProjectUdp = new(uint16)
		err = decoder.Decode(ty.ProjectUdp)
		if err != nil {
			return err
		}
		return
	default:
		return fmt.Errorf("unrecognized enum")
	}
}

type ContainerDisk struct { // Composite
	Id   uint64
	Path []byte
}
type Env struct { // Enum
	Env *struct { // 0
		F0 []byte
		F1 []byte
	}
	File *struct { // 1
		F0 []byte
		F1 []byte
	}
	Encrypt *struct { // 2
		F0 []byte
		F1 uint64
	}
}

func (ty Env) Encode(encoder scale.Encoder) (err error) {
	if ty.Env != nil {
		err = encoder.PushByte(0)
		if err != nil {
			return err
		}

		err = encoder.Encode(ty.Env.F0)
		if err != nil {
			return err
		}

		err = encoder.Encode(ty.Env.F1)
		if err != nil {
			return err
		}

		return nil
	}

	if ty.File != nil {
		err = encoder.PushByte(1)
		if err != nil {
			return err
		}

		err = encoder.Encode(ty.File.F0)
		if err != nil {
			return err
		}

		err = encoder.Encode(ty.File.F1)
		if err != nil {
			return err
		}

		return nil
	}

	if ty.Encrypt != nil {
		err = encoder.PushByte(2)
		if err != nil {
			return err
		}

		err = encoder.Encode(ty.Encrypt.F0)
		if err != nil {
			return err
		}

		err = encoder.Encode(ty.Encrypt.F1)
		if err != nil {
			return err
		}

		return nil
	}
	return fmt.Errorf("unrecognized enum")
}

func (ty *Env) Decode(decoder scale.Decoder) (err error) {
	variant, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}
	switch variant {
	case 0: // Tuple
		ty.Env = &struct {
			F0 []byte
			F1 []byte
		}{}

		err = decoder.Decode(&ty.Env.F0)
		if err != nil {
			return err
		}

		err = decoder.Decode(&ty.Env.F1)
		if err != nil {
			return err
		}

		return
	case 1: // Tuple
		ty.File = &struct {
			F0 []byte
			F1 []byte
		}{}

		err = decoder.Decode(&ty.File.F0)
		if err != nil {
			return err
		}

		err = decoder.Decode(&ty.File.F1)
		if err != nil {
			return err
		}

		return
	case 2: // Tuple
		ty.Encrypt = &struct {
			F0 []byte
			F1 uint64
		}{}

		err = decoder.Decode(&ty.Encrypt.F0)
		if err != nil {
			return err
		}

		err = decoder.Decode(&ty.Encrypt.F1)
		if err != nil {
			return err
		}

		return
	default:
		return fmt.Errorf("unrecognized enum")
	}
}

type Container struct { // Composite
	Image   []byte
	Command Command
	Port    []Service
	Cpu     uint32
	Mem     uint32
	Disk    []ContainerDisk
	Gpu     uint32
	Env     []Env
}
type Tuple_34 struct { // Tuple
	F0 uint64
	F1 Container
}
type Tuple_36 struct { // Tuple
	F0 uint64
	F1 Pod
	F2 []Tuple_34
	F3 byte
}
type Tuple_39 struct { // Tuple
	F0 uint64
	F1 uint32
	F2 uint32
	F3 byte
}
type Secret struct { // Composite
	K      []byte
	Hash   types.H256
	Minted bool
}
type Tuple_44 struct { // Tuple
	F0 uint64
	F1 Secret
}
type Disk struct { // Enum
	SecretSSD *struct { // 0
		F0 []byte
		F1 []byte
		F2 uint32
	}
}

func (ty Disk) Encode(encoder scale.Encoder) (err error) {
	if ty.SecretSSD != nil {
		err = encoder.PushByte(0)
		if err != nil {
			return err
		}

		err = encoder.Encode(ty.SecretSSD.F0)
		if err != nil {
			return err
		}

		err = encoder.Encode(ty.SecretSSD.F1)
		if err != nil {
			return err
		}

		err = encoder.Encode(ty.SecretSSD.F2)
		if err != nil {
			return err
		}

		return nil
	}
	return fmt.Errorf("unrecognized enum")
}

func (ty *Disk) Decode(decoder scale.Decoder) (err error) {
	variant, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}
	switch variant {
	case 0: // Tuple
		ty.SecretSSD = &struct {
			F0 []byte
			F1 []byte
			F2 uint32
		}{}

		err = decoder.Decode(&ty.SecretSSD.F0)
		if err != nil {
			return err
		}

		err = decoder.Decode(&ty.SecretSSD.F1)
		if err != nil {
			return err
		}

		err = decoder.Decode(&ty.SecretSSD.F2)
		if err != nil {
			return err
		}

		return
	default:
		return fmt.Errorf("unrecognized enum")
	}
}

type Tuple_55 struct { // Tuple
	F0 uint64
	F1 Disk
}
type Ip struct { // Composite
	Ipv4   util.Option[uint32]
	Ipv6   util.Option[types.U128]
	Domain util.Option[[]byte]
}
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
type Tuple_66 struct { // Tuple
	F0 uint64
	F1 K8sCluster
	F2 []byte
}
type Tuple_71 struct { // Tuple
	F0 Container
	F1 []util.Option[Disk]
}
type Tuple_72 struct { // Tuple
	F0 uint64
	F1 Tuple_71
}
type Tuple_74 struct { // Tuple
	F0 uint64
	F1 Pod
	F2 []Tuple_72
	F3 uint32
	F4 uint32
	F5 byte
}
type Tuple_77 struct { // Tuple
	F0 Pod
	F1 []Tuple_34
	F2 uint32
	F3 byte
}
type EditType struct { // Enum
	INSERT *bool   // 0
	UPDATE *uint64 // 1
	REMOVE *uint64 // 2
}

func (ty EditType) Encode(encoder scale.Encoder) (err error) {
	if ty.INSERT != nil {
		err = encoder.PushByte(0)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.UPDATE != nil {
		err = encoder.PushByte(1)
		if err != nil {
			return err
		}
		err = encoder.Encode(*ty.UPDATE)
		if err != nil {
			return err
		}
		return nil
	}

	if ty.REMOVE != nil {
		err = encoder.PushByte(2)
		if err != nil {
			return err
		}
		err = encoder.Encode(*ty.REMOVE)
		if err != nil {
			return err
		}
		return nil
	}
	return fmt.Errorf("unrecognized enum")
}

func (ty *EditType) Decode(decoder scale.Decoder) (err error) {
	variant, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}
	switch variant {
	case 0: // Base
		t := true
		ty.INSERT = &t
		return
	case 1: // Inline
		ty.UPDATE = new(uint64)
		err = decoder.Decode(ty.UPDATE)
		if err != nil {
			return err
		}
		return
	case 2: // Inline
		ty.REMOVE = new(uint64)
		err = decoder.Decode(ty.REMOVE)
		if err != nil {
			return err
		}
		return
	default:
		return fmt.Errorf("unrecognized enum")
	}
}

type ContainerInput struct { // Composite
	Etype     EditType
	Container Container
}
