# How to use the blockscout as block explorer for chain

## 1. Start the chain

The blockscout need lots of rpc api to run node, so need build as:

```bash
cd ./amax.eva.chain
make release-tracing
```

use the params to allow eth apis:

```bash
./target/release/amax-eva --dev -d ./chain-datas --ws-external  --rpc-external --rpc-port 9999 --ethapi debug --ethapi trace --ethapi txpool 
```

also we can start a tmp chain for tests:

```bash
./target/release/amax-eva --dev --tmp --ws-external  --rpc-external --rpc-port 9999 --ethapi debug --ethapi trace --ethapi txpool 
```

## 2. Build the blockscout by docker

blockscout need lots of dependencies, and to run it will add too much packages for our env, so we can use docker to make it clearly.

```bash
git clone https://github.com/chain-developer/blockscout.git
git checkout for-amax-eva
```

change the env file in `blockscout/docker-compose/envs/common-blockscout.env`

need to change this params:

- `ETHEREUM_JSONRPC_HTTP_URL`, use the url for amax-eva rpc port, which is `http://local-ip:9999/`
- `ETHEREUM_JSONRPC_TRACE_URL`, same as `ETHEREUM_JSONRPC_HTTP_URL`

then run the docker images by docker-compose:

```bash
cd blockscout/docker-compose
docker-compose up --build
```

the explorer run in `http://localhost:4000`.

## 3. Build with proxy

to build the image will need a long time, if need proxy for network, export `HTTPS_PROXY` in environment.

we can use the docker-compose file at `blockscout/docker-compose-cn` to use the proxy, we need add `HTTPS_PROXY` in  `blockscout/docker-compose-cn/envs/common-blockscout.env`.

## 4. Close the blockscout

use `docker-compose down` to make sure the blockscout is shutdown.
