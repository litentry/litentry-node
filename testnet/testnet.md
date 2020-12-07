# Testnet

telnet 18.163.198.168 30333

## dev environment
/home/ec2-user/litentry-node/target/release/litentry-node --dev \
--cors==all \
 > /home/ec2-user/logs/litentry.log 2>&1 &

## generate key for seed node
^C[ec2-user@ip-172-31-17-223 ~]$ subkey inspect //Alice
Secret Key URI `//Alice` is account:
  Secret seed:      0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a
  Public key (hex): 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
  Account ID:       0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
  SS58 Address:     5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY


## create spec for testnet
/home/ec2-user/litentry-node/target/release/litentry-node build-spec --disable-default-bootnode > litentry.json

## Start the seed node
/home/ec2-user/litentry-node/target/release/litentry-node \
  --chain /home/ec2-user/litentry-node/testnet/litentry.json \
  > /home/ec2-user/logs/litentry.log 2>&1 &

## Start the alice node in local
target/release/litentry-node --chain testnet/litentry.json --bootnodes /ip4/18.163.198.168/tcp/30333/p2p/12D3KooWCSctENzjm1mML2Ym5i6nBue79a7ch3mYGPVMXxRHkSLZ  --alice --validator

## Start the alice in aws
/home/ec2-user/litentry-node/target/release/litentry-node \
  --chain litentry.json \
  --alice \
  --validator \
  --bootnodes /ip4/18.163.198.168/tcp/30333/p2p/12D3KooWCSctENzjm1mML2Ym5i6nBue79a7ch3mYGPVMXxRHkSLZ > /home/ec2-user/logs/litentry.log 2>&1 &


