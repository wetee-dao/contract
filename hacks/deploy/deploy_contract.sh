#!/usr/bin/env bash
set -euo pipefail

SOURCE="$0"
while [ -h "$SOURCE" ]; do
    DIR="$(cd -P "$(dirname "$SOURCE")" && pwd)"
    SOURCE="$(readlink "$SOURCE")"
    [[ $SOURCE != /* ]] && SOURCE="$DIR/$SOURCE"
done
DIR="$(cd -P "$(dirname "$SOURCE")" && pwd)"
ROOT="$(cd "$DIR/../../" && pwd)"

ENV="local"
CHAIN_URL=""
CONTRACT_NAME=""
CONTRACT_DIR=""
CODE_PATH=""
SURI=""
NETWORK="42"
BUILD="true"
FULL_INIT="false"

usage() {
    cat <<EOF
Usage:
  $(basename "$0") --name <contract-name> [options]
  $(basename "$0") --full-init [options]

Options:
  --env <env>        Environment: local | test | main, default: local
  --url <url>        Blockchain websocket url (overrides --env)
  --name <name>      Contract name, e.g. cloud / subnet / proxy
  --dir <dir>        Contract crate directory (directory containing Cargo.toml)
  --code <path>      Compiled .polkavm file path, default: <workspace-root>/target/<name>.release.polkavm
  --suri <secret>    Signer secret uri, default: //Alice
  --network <id>     SS58 network id, default: 42
  --build <bool>     Whether to run cargo wrevive build first, default: true
  --full-init        Deploy the full system (pod + subnet + cloud + proxy + init)

Presets:
  local              wss://xiaobai.asyou.me:30001/ws
  test               wss://asset-hub-paseo-rpc.n.dwellir.com
  main               wss://polkadot-asset-hub-rpc.polkadot.io

Examples:
  $(basename "$0") --name cloud --dir "$ROOT/revives/Cloud"
  $(basename "$0") --env test --name proxy --dir "$ROOT/revives/proxy" --suri //Bob
  $(basename "$0") --env main --name subnet --dir "$ROOT/revives/Subnet"
  $(basename "$0") --full-init --env local
  $(basename "$0") --full-init --env test
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --env) ENV="$2"; shift 2 ;;
        --url) CHAIN_URL="$2"; shift 2 ;;
        --name) CONTRACT_NAME="$2"; shift 2 ;;
        --dir) CONTRACT_DIR="$2"; shift 2 ;;
        --code) CODE_PATH="$2"; shift 2 ;;
        --suri) SURI="$2"; shift 2 ;;
        --network) NETWORK="$2"; shift 2 ;;
        --build) BUILD="$2"; shift 2 ;;
        --full-init) FULL_INIT="true"; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown argument: $1" >&2; usage; exit 1 ;;
    esac
done

# Resolve SURI: explicit --suri > .key.<env> > .key > default //Alice
if [[ -z "$SURI" ]]; then
    KEY_FILE=""
    if [[ -f "$DIR/.key.$ENV" ]]; then
        KEY_FILE="$DIR/.key.$ENV"
    elif [[ -f "$DIR/.key" ]]; then
        KEY_FILE="$DIR/.key"
    fi

    if [[ -n "$KEY_FILE" ]]; then
        SURI="$(cat "$KEY_FILE")"
        if [[ -z "$SURI" ]]; then
            echo "Warning: $KEY_FILE is empty, using default //Alice" >&2
            SURI="//Alice"
        fi
    else
        SURI="//Alice"
    fi
fi

# Resolve URL from environment preset if not explicitly provided
case "$ENV" in
    local)
        [[ -z "$CHAIN_URL" ]] && CHAIN_URL="wss://xiaobai.asyou.me:30001/ws"
        ;;
    test)
        [[ -z "$CHAIN_URL" ]] && CHAIN_URL="wss://asset-hub-paseo-rpc.n.dwellir.com"
        ;;
    main)
        [[ -z "$CHAIN_URL" ]] && CHAIN_URL="wss://polkadot-asset-hub-rpc.polkadot.io"
        ;;
    *)
        echo "Unknown environment: $ENV" >&2
        usage
        exit 1
        ;;
esac

cd "$DIR"

if [[ "$FULL_INIT" == "true" ]]; then
    if [[ "$BUILD" == "true" ]]; then
        cargo wrevive build -p pod
        cargo wrevive build -p subnet
        cargo wrevive build -p cloud
        cargo wrevive build -p proxy
    fi

    go run ./cmd/deploy-full \
        -url "$CHAIN_URL" \
        -dir "$ROOT" \
        -suri "$SURI" \
        -network "$NETWORK"
    exit 0
fi

# Single-contract deployment
if [[ -z "$CONTRACT_NAME" ]]; then
    usage
    exit 1
fi

if [[ -z "$CONTRACT_DIR" ]]; then
    if [[ -d "$ROOT/revives/$CONTRACT_NAME" ]]; then
        CONTRACT_DIR="$ROOT/revives/$CONTRACT_NAME"
    elif [[ -d "$ROOT/revives/${CONTRACT_NAME^}" ]]; then
        CONTRACT_DIR="$ROOT/revives/${CONTRACT_NAME^}"
    else
        echo "Unable to infer contract dir, please pass --dir" >&2
        exit 1
    fi
fi

if [[ ! -f "$CONTRACT_DIR/Cargo.toml" ]]; then
    echo "No Cargo.toml found in contract dir: $CONTRACT_DIR" >&2
    exit 1
fi

WORKSPACE_ROOT="$CONTRACT_DIR"
while [[ "$WORKSPACE_ROOT" != "/" ]]; do
    if [[ -f "$WORKSPACE_ROOT/Cargo.toml" ]] && grep -q "^\[workspace\]" "$WORKSPACE_ROOT/Cargo.toml"; then
        break
    fi
    WORKSPACE_ROOT="$(dirname "$WORKSPACE_ROOT")"
done

if [[ "$WORKSPACE_ROOT" == "/" ]]; then
    echo "Unable to locate workspace root from contract dir: $CONTRACT_DIR" >&2
    exit 1
fi

if [[ -z "$CODE_PATH" ]]; then
    CODE_PATH="$WORKSPACE_ROOT/target/$CONTRACT_NAME.release.polkavm"
fi

if [[ "$BUILD" == "true" ]]; then
    cargo wrevive build --manifest-path "$CONTRACT_DIR/Cargo.toml"
fi

go run ./cmd/deploy-contract \
    -url "$CHAIN_URL" \
    -name "$CONTRACT_NAME" \
    -dir "$WORKSPACE_ROOT" \
    -code "$CODE_PATH" \
    -suri "$SURI" \
    -network "$NETWORK"
