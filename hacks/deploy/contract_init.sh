# get shell path
SOURCE="$0"
while [ -h "$SOURCE"  ]; do
    DIR="$( cd -P "$( dirname "$SOURCE"  )" && pwd  )"
    SOURCE="$(readlink "$SOURCE")"
    [[ $SOURCE != /*  ]] && SOURCE="$DIR/$SOURCE"
done
DIR="$( cd -P "$( dirname "$SOURCE"  )" && pwd  )"

cd "$DIR/../../"

cargo wrevive build -p pod
cargo wrevive build -p subnet
cargo wrevive build -p cloud
cargo wrevive build -p proxy

cd $DIR/contracts

go-ink-gen -json ../../../target/cloud.json
go-ink-gen -json ../../../target/subnet.json
go-ink-gen -json ../../../target/proxy.json

cd $DIR
go test -run ^TestContractInit$