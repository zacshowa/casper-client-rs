use serde::{Deserialize, Serialize};

use casper_types::{
    contract_messages::Messages, execution::ExecutionResultV2, BlockHash, ProtocolVersion,
    Transaction,
};

pub(crate) const SPECULATIVE_EXEC_TXN_METHOD: &str = "speculative_exec_txn";

/// Params for "speculative_exec_txn" RPC request.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SpeculativeExecTxnParams {
    /// Transaction to execute.
    pub transaction: Transaction,
}

impl SpeculativeExecTxnParams {
    /// Creates a new `SpeculativeExecTxnParams`.
    pub fn new(transaction: Transaction) -> Self {
        SpeculativeExecTxnParams {
            transaction,
        }
    }
}

/// Result for "speculative_exec_txn" and "speculative_exec" RPC responses.
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SpeculativeExecTxnResult {
    /// The RPC API version.
    pub api_version: ProtocolVersion,
    /// Hash of the block on top of which the transaction was executed.
    pub block_hash: BlockHash,
    /// Result of the execution.
    pub execution_result: ExecutionResultV2,
    /// Messages emitted during execution.
    pub messages: Messages,
}
