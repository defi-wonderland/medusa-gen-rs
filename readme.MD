# Medusa Template Generator

This is a tool to generate a set of contracts for a Medusa testing campaign, following Wonderland usage.
The following contracts are generated, according to this structure (I know, we call the child "parent" for...reason):
- fuzz/
  - FuzzTest.sol
  - handlers/
    - HandlerParent.sol
    - HandlerA.sol
    - HandlerB.sol
    - HandlerC.sol
  - properties/
    - PropertyParent.sol
    - PropertyA.sol
    - PropertyB.sol
    - PropertyC.sol
  - Setup.sol

The inheritance tree is as follows, FuzzTest is the entry point:
- FuzzTest, inherit ParentProperty
- ParentProperty, inherit each properties contract
- PropertyA, PropertyB, PropertyC, etc inherit HandlerParent
- HandlerParent, inherit each handler contract
- HandlerA, HandlerB, HandlerC, etc inherit Setup

## Installation

This is an early alpha version, only available by building from sources:
```bash
cargo install --git https://github.com/drgorillamd/medusa-gen.rs
```

## Usage

```bash
medusa-gen -n 3 -p 4
```

### Options
--solc, -s: Solidity compiler version, default is 0.8.23
--nb-handlers, -n: Number of handlers, default is 2
--nb-properties, -p: Number of properties, default is 2
