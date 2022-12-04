# JSON-RPC 2.0 for fedimint
## Run clientd
### 1. Setup fedimint
Follow the [instructions](https://github.com/fedimint/fedimint/blob/master/docs/dev-running.md) to set up a federation and a client configuration.
If you did not choose a working directory manually it will be set in your environment as `$FM_CFG_DIR`.
```bash
$ echo $FM_CFG_DIR; # should output the configuration directory
```
### 2. Run clientd with the cfg
Now you can run `clientd` like this:
```bash
$ export RUST_LOG=info # useful telemetry, use error or trace if needed
$ cargo run --bin clientd -- --workdir=$FM_CFG_DIR
```
If you set `RUST_LOG=info` you should see on what address the server is listening:
```bash
    Finished dev [unoptimized + debuginfo] target(s) in 0.24s
     Running `target/debug/clientd --workdir=../client_workdir/cfg/`
2022-12-03T18:22:14.212356Z  INFO clientd::server: server address: 127.0.0.1:35573

```
You can start using the API via websocket or http. Curl the health check to see if the server is really alive:
```bash
curl -vX POST http://127.0.0.1:43337\
   -H 'Content-Type: application/json'\
   -d '{"jsonrpc": "2.0", "method": "health_check", "id": 0}'
# should return a response object like this:
# {"jsonrpc":"2.0","result":{"HealthCheck":[]},"id":0}
```