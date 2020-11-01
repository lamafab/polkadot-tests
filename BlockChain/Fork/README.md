# Fork Testing

Those tests verify whether the Polkadot Host implementation can correctly
transition between forks.

## Description

The testing data includes block chains which contain forks, meaning that
multiple blocks are built on top of the same parent block. Each block must only
have access to the data of the fork it is on. The storage space must therefore
be segregated based on those forks.

Those are basic concepts that a state machine is supposed to support.

## Requirements

The Polkadot Host implementation must offer the following functionality:

* Import a genesis state from file (see specification ...).
* Import and execute a block from file (see specification ...).
    * The block is executed by sending it to the `Core_execute_block` runtime function.

## Testing Process

The Polkadot Host implementation loads the genesis state as specified in the
`genesis.json` file. The genesis state includes the custom Runtime (compiled
from `src/runtime/`) which creates a log statement of the state root after a
block is executed by calling the `Core_execute_block` runtime function. The
tester captures that log statement and compares it against the expected result.

After each block is executed and the comparison of the log statement and the
expected result is valid, the Polkadot Host implementation is compliant.

### Workflow

* Import genesis state.
* For each block:
    * Execute block.
    * Capture log statement.
    * Compare the log statement to the expected result.

## Test Data

Multiple tests can be found in the `tests/` directory. Each test is structured as:

```json
{
   "name": "Fork Awareness",
   "type": "BlockChainFork",
   "description": "Multiple blocks were produced on the same parent block",
   "genesis": "genesis.json",
   "data":[
      {
         "block": "Block 1",
         "header":{
            "parentHash":"0xd380bee22de487a707cbda65dd9d4e2188f736908c42cf390c8919d4f7fc547c",
            "number":"0x1",
            "stateRoot":"0x01045dae0c5d93a84c3dc1f0131126aa6aa1feb26d10f029166fc0c607468968",
            "extrinsicsRoot":"0xa9439bbc818bd95eadb2c5349bef77ee7cc80a282fcceb9670c2c12f939211b4",
            "digest":{
               "logs":[
                  "0x0642414245b50103000000009ddecc0f00000000a8a9c1d717f3904506e333d0ebbf4eed297d50ab9b7c57458b10182f1c84025ef09d3fb5b5f4cb81688939e6363f95aa8d91645fa7b8abc0a6f37812c777c307df51071082d3ff89d4e1b5ad8f5cd3711ada74292c4808237bdf2b076edb280c",
                  "0x05424142450101f66230eb71705213dd10256e3ca5af07492ac420128ecb8bc98f1fcd1f74986d348addbabd4813f0022835b21d720ecadce66a57480d87dfd51d77f3474cb68b"
               ]
            }
         },
         "extrinsics":[
            "0x280403000bb07fa1517201",
            "0x1004140000"
         ],
         "postState": "0x8d4ea2ea4e834faa1ed492f66f0b28ea56fc9061b7e89623114968e2cf59987a"
      },
      {
         "block": "Block 2",
         ...
      }
   ]
}
```

After the tester runs the Polkadot Host implementation with the specified genesis
state, it must execute each block and check the log statement created by the
Runtime against the `post_state` result.
