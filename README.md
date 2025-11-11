# transactions-engine
This transaction engine treats all the lines form the input csv file as transactions and processes them one by one. It supports 5 differet types of transactions:

1. Deposit (client_id, transaction_id, amount)
2. Withdrawal (client_id, transaction_id, amount)
3. Dispute (client_id, transaction_id)
4. Resolve (client_id, transaction_id)
5. Chargeback (client_id, transaction_id)

## Modules
1. `type.rs` defines the transaction types needed to deserialize the transactions from the csv file. For representing the amounts we are using `rust-decimal` crate that helps with the 4 digits precision.
2. `engine.rs` processes the transaction one by one. The trasaction engine keeps a `HashMap` with all the client details. For each transaction it identifies the correct client and the transaction type. It is also in charge of writing the current state of the system to stdout in csv format.
3. `client.rs` defines the state of a client account: the total amount, the amount that it is under dispute, the amount that is available, whether of not the account is locked, a list with all processed transactions and a list with transactions that are currently under dispute. A transaction will only be processed if the account of the client has not been locked.

## How to run
In order to run the examples from this repo the following command can be used:
```
cargo run <input_file> > <output_file>
```
Logs are disabled by default, if you want to enable them, set RUST_LOG with the targeted log level.

## Testing
The core modules include unit tests. Some example inputs are also included in the `examples/` folder.

## Possible improvements
- Use multiple threads for processing multiple transactions at the same time. This would also mean that the code should be redesigned to be multithread-safe.
- Remove old processed transaction from the client accounts after a period of time.
- Add a more diverse data set for testing.