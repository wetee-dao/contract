# get shell path
SOURCE="$0"
while [ -h "$SOURCE"  ]; do
    DIR="$( cd -P "$( dirname "$SOURCE"  )" && pwd  )"
    SOURCE="$(readlink "$SOURCE")"
    [[ $SOURCE != /*  ]] && SOURCE="$DIR/$SOURCE"
done
DIR="$( cd -P "$( dirname "$SOURCE"  )" && pwd  )"

cd "$DIR/../../"
cargo contract build --release --manifest-path contracts/Cloud/Cargo.toml

cd $DIR/contracts
go-ink-gen -json ../../../target/ink/cloud/cloud.json

cd $DIR
go test -run ^TestCloudUpdate$