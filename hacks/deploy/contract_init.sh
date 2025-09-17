# get shell path
SOURCE="$0"
while [ -h "$SOURCE"  ]; do
    DIR="$( cd -P "$( dirname "$SOURCE"  )" && pwd  )"
    SOURCE="$(readlink "$SOURCE")"
    [[ $SOURCE != /*  ]] && SOURCE="$DIR/$SOURCE"
done
DIR="$( cd -P "$( dirname "$SOURCE"  )" && pwd  )"

cd "$DIR/../../"

# cargo contract build --release --manifest-path contracts/Pod/Cargo.toml
# cargo contract build --release --manifest-path contracts/Subnet/Cargo.toml
# cargo contract build --release --manifest-path contracts/Cloud/Cargo.toml

cd $DIR
go test -run ^TestContractInit$