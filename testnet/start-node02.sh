ip=127.0.0.1
p2p_port=30334
node_identity=$1
./target/debug/litentry-node -d /tmp/1 --chain litentry --rpc-external --ws-external --rpc-methods Unsafe --rpc-cors all --ws-port 9902 --port 30445 --rpc-port 9903 --validator --bootnodes /ip4/$ip/tcp/$p2p_port/p2p/$node_identity
