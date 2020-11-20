# Toolkit

This toolkit is a work-in-progress utility in order to create Polkadot transactions or to create script-able workflows and test cases.

## Building

```console
$ cargo build --bin toolkit
```

## Usage

The toolkit can either be used directly via the command line or by using scriptable workflows defined in YAML files.

### CLI

```bash
$ toolkit pallet-balances transfer --from alice --to bob --balance 100
```

More docs to come.

### YAML

In the YAML file:

```yaml
- vars:
    balance: 100

- name: Extrinsic with variable
  pallet_balances:
    transfer:
      from: alice
      to: bob
      balance: "{{ balance }}"

- name: Extrinsics with loops
  pallet_balances:
    transfer:
      from: "{{ item.from }}"
      to: "{{ item.to }}"
      balance: "{{ item.balance }}"
  loop:
    - { from: alice, to: bob, balance: 100 }
    - { from: alice, to: dave, balance: 200 }
    - { from: bob, to: alice, balance: 300 }
    - { from: dave, to: bob, balance: 400 }
```

In order to execute it:

```bash
$ toolkit path/to/file.yml
```

See `examples/` directory. More docs to come.
