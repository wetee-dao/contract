# nohup wetee-node --dev --base-path  ./out/chain-data --rpc-external --rpc-methods=unsafe --unsafe-rpc-external --rpc-cors=all &

cd out/
nohup kube-explorer --kubeconfig=/home/wetee/.kube/config --http-listen-port=9898 --https-listen-port=0 &