# Compute Unit Inspector

This Rust program inspects the Solana validator's produced slots within the current epoch and computes the total and average Compute Units (CUs) consumed in those slots. It also tracks how many transactions are votes, how many fail, and how many succeed.

## Overview

1. **Fetch Leader Schedule**: The program retrieves the leader schedule from a specified Solana RPC endpoint and identifies which slots are assigned to a particular validator.
2. **Retrieve Block Data**: It fetches blocks for those slots and extracts:
   - Compute Units (CUs) consumed by each transaction.
   - Transaction success/failure.
   - Vote vs. non-vote transactions.
3. **Calculate Statistics**: 
   - Average percentage of the total CU capacity used per slot.
   - Average number of vote transactions, succeeded transactions, and failed transactions per slot.

## Table of Contents
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Example Output](#example-output)
- [License](#license)

## Prerequisites

- [Rust](https://www.rust-lang.org/) (>= 1.60.0 recommended)
- A Solana RPC endpoint (default is `https://api.mainnet-beta.solana.com` in this code).

## Installation

1. **Clone this repository**:
    ```bash
    git clone https://github.com/<your-username>/compute-unit-inspector.git
    cd compute-unit-inspector
    ```
2. **Build**:
    ```bash
    cargo build --release
    ```
3. **Install** (optional, to have a system-wide binary):
    ```bash
    cargo install --path .
    ```

Alternatively you can run the following:
```bash
cargo run <validator_identity>
```

## Usage

The program requires exactly one argument: the validator's public key (`validator_identity`). For example:

```bash
./target/release/compute-unit-inspector <validator_identity>
```

## Command-line Arguments
<validator_identity>: The Solana validator identity key you want to inspect.

### Example:

```bash
./target/release/compute-unit-inspector 11111111111111111111111111111111
```

### Configuration
By default, the program uses:

RPC URL: https://api.mainnet-beta.solana.com
TOTAL_CAPABLE_CU: 48_000_000
If you wish to change the RPC endpoint, you can edit the following line in main():

```
rust
let rpc_url = "https://api.mainnet-beta.solana.com"; // Replace with your RPC URL
```
### Example Output
Below is an example snippet of what the output might look like:

```
yaml
Validator 11111111111111111111111111111111 has 128 total slots in the leader schedule.
Validator 11111111111111111111111111111111 has 96 produced slots up to the current slot (123456789).
Slot: 123450001, CU Used: 300000, % of Total Capable CU: 0.63%, Vote Transactions: 1, Succeeded Transactions: 10, Failed Transactions: 2
Slot: 123450015, CU Used: 450000, % of Total Capable CU: 0.94%, Vote Transactions: 1, Succeeded Transactions: 13, Failed Transactions: 1
...
----------------------------------------
Epoch 350 Stats
Average CU per Slot: 0.75%
Average Vote Transactions per Slot: 1
Average Succeeded Transactions per Slot: 10
Average Failed Transactions per Slot: 1
Done calculating CU for validator's slots.
License
This project is open source and available under the MIT License. Feel free to use, modify, and distribute this software in accordance with the license terms.
```
