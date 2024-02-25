## Circom GKR
circom-gkr is a command line interface for proving and verifying code written in circom using a GKR backend. 

## Dependency
- you should already have circom cli installed
  - see: https://docs.circom.io/getting-started/installation

## Installation
```shell
git clone https://github.com/iammadab/zk.git
cd zk/circom-gkr
cargo install --path .
```

## Help
```shell
circom-gkr --help
```

## Commands
### Compile
Uses the circom binary to compile the specified circom code to .r1cs and .wasm
```shell
circom-gkr compile <path-to-circom-file>
```

### Generate witness
After specifying the program input, the next step is to generate the prover witness. 
```shell
circom-gkr generate-witness <path-to-circom-file>
```

### Generate proof
```shell
circom-gkr prove <path-to-circom-file>
```

### Verify proof
```shell
circom-gkr verify <path-to-circom-file>
```

### Prove and Verify
this command generates and verifies the proof in one step
```shell
circom-gkr prove-verify <path-to-circom-file>
```