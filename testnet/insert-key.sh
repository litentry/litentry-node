RPC_PORT1=9901
curl http://localhost:$RPC_PORT1 -H "Content-Type:application/json;charset=utf-8" -d "@node01-insert-aura.json"
curl http://localhost:$RPC_PORT1 -H "Content-Type:application/json;charset=utf-8" -d "@node01-insert-gran.json"
RPC_PORT2=9903
curl http://localhost:$RPC_PORT2 -H "Content-Type:application/json;charset=utf-8" -d "@node02-insert-aura.json"
curl http://localhost:$RPC_PORT2 -H "Content-Type:application/json;charset=utf-8" -d "@node02-insert-gran.json"
