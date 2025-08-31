#!/bin/bash

# get shell path
SOURCE="$0"
while [ -h "$SOURCE"  ]; do
    DIR="$( cd -P "$( dirname "$SOURCE"  )" && pwd  )"
    SOURCE="$(readlink "$SOURCE")"
    [[ $SOURCE != /*  ]] && SOURCE="$DIR/$SOURCE"
done
DIR="$( cd -P "$( dirname "$SOURCE"  )" && pwd  )"
cd $DIR/../

# nohup wetee-node --dev --base-path  ./out/chain-data --rpc-external --rpc-methods=unsafe --unsafe-rpc-external --rpc-cors=all &

cd out/
nohup kube-explorer --kubeconfig=/home/wetee/.kube/config --http-listen-port=9898 --https-listen-port=0 &