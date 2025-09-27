# get shell path
SOURCE="$0"
while [ -h "$SOURCE"  ]; do
    DIR="$( cd -P "$( dirname "$SOURCE"  )" && pwd  )"
    SOURCE="$(readlink "$SOURCE")"
    [[ $SOURCE != /*  ]] && SOURCE="$DIR/$SOURCE"
done
DIR="$( cd -P "$( dirname "$SOURCE"  )" && pwd  )"

cd "$DIR/../../"
cargo contract build --release --manifest-path contracts/Subnet/Cargo.toml

cd $DIR/contracts
go-ink-gen -json ../../../target/ink/subnet/subnet.json

cd $DIR
go test -run ^TestSubnetUpdate$