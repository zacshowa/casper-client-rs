//! An API suitable for use by a CLI binary.
//!
//! It provides functions and types largely based around strings and integers, as would be expected
//! to be input by a CLI user.  The functions then parse these inputs into the expected Rust types
//! and pass them through to the equivalent API functions defined in the root of the library.
//!
//! # Common Parameters
//!
//! Many of the functions have similar parameters.  Descriptions for these common ones follow:
//!
//! * `maybe_rpc_id` - The JSON-RPC identifier, applied to the request and returned in the response.
//!   If it can be parsed as an `i64` it will be used as a JSON integer. If empty, a random `i64`
//!   will be assigned.  Otherwise the provided string will be used verbatim.
//! * `node_address` - The hostname or IP and port of the server, e.g. `http://127.0.0.1:7777`.
//! * `verbosity_level` - When `1`, the JSON-RPC request will be printed to `stdout` with long
//!   string fields (e.g. hex-formatted raw Wasm bytes) shortened to a string indicating the char
//!   count of the field.  When `verbosity_level` is greater than `1`, the request will be printed
//!   to `stdout` with no abbreviation of long fields.  When `verbosity_level` is `0`, the request
//!   will not be printed to `stdout`.
//! * `maybe_block_id` - Must be a hex-encoded, 32-byte hash digest or a `u64` representing the
//!   [`Block`] height or empty.  If empty, the latest `Block` known on the server will be used.

/// Deploy module.
pub mod deploy;
mod deploy_str_params;
mod dictionary_item_str_params;
mod error;
mod json_args;
mod parse;
mod payment_str_params;
mod session_str_params;
mod simple_args;
pub mod transaction;
pub mod transaction_str_params;

#[cfg(test)]
mod tests;

use serde::Serialize;

#[cfg(doc)]
use casper_types::{account::AccountHash, Key};
use casper_types::{Digest, URef};

use crate::cli::transaction_str_params::TransactionStrParams;
use crate::{
    rpcs::{
        results::{
            GetAccountResult, GetAuctionInfoResult, GetBalanceResult, GetBlockResult,
            GetBlockTransfersResult, GetChainspecResult, GetDeployResult, GetDictionaryItemResult,
            GetEraInfoResult, GetEraSummaryResult, GetNodeStatusResult, GetPeersResult,
            GetStateRootHashResult, GetValidatorChangesResult, ListRpcsResult, PutDeployResult,
            QueryBalanceResult, QueryGlobalStateResult, SpeculativeExecResult,
        },
        DictionaryItemIdentifier,
    },
    SuccessResponse,
};
#[cfg(doc)]
use crate::{Account, Block, Deploy, Error, StoredValue, Transfer};
#[cfg(doc)]
use casper_types::PublicKey;
pub use deploy_str_params::DeployStrParams;
pub use dictionary_item_str_params::DictionaryItemStrParams;
pub use error::CliError;
use json_args::JsonArg;
pub use json_args::{
    help as json_args_help, Error as JsonArgsError, ErrorDetails as JsonArgsErrorDetails,
};
pub use payment_str_params::PaymentStrParams;
pub use session_str_params::SessionStrParams;
pub use simple_args::help as simple_args_help;

/// Creates a [`Deploy`] and sends it to the network for execution.
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters) or the docs
/// of the individual parameter types.
pub async fn put_deploy(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    deploy_params: DeployStrParams<'_>,
    session_params: SessionStrParams<'_>,
    payment_params: PaymentStrParams<'_>,
) -> Result<SuccessResponse<PutDeployResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let deploy =
        deploy::with_payment_and_session(deploy_params, payment_params, session_params, false)?;
    crate::put_deploy(rpc_id, node_address, verbosity, deploy)
        .await
        .map_err(CliError::from)
}

/// Creates a [`Deploy`] and sends it to the specified node for speculative execution.
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters) or the docs
/// of the individual parameter types.
pub async fn speculative_put_deploy(
    maybe_block_id: &str,
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    deploy_params: DeployStrParams<'_>,
    session_params: SessionStrParams<'_>,
    payment_params: PaymentStrParams<'_>,
) -> Result<SuccessResponse<SpeculativeExecResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let deploy =
        deploy::with_payment_and_session(deploy_params, payment_params, session_params, false)?;
    let speculative_exec = parse::block_identifier(maybe_block_id)?;
    crate::speculative_exec(rpc_id, node_address, speculative_exec, verbosity, deploy)
        .await
        .map_err(CliError::from)
}
/// Creates a [`Deploy`] and outputs it to a file or stdout.
///
/// As a file, the `Deploy` can subsequently be signed by other parties using [`sign_deploy_file`]
/// and then sent to the network for execution using [`send_deploy_file`].
///
/// `maybe_output_path` specifies the output file path, or if empty, will print it to `stdout`.  If
/// `force` is true, and a file exists at `maybe_output_path`, it will be overwritten.  If `force`
/// is false and a file exists at `maybe_output_path`, [`Error::FileAlreadyExists`] is returned
/// and the file will not be written.
pub fn make_deploy(
    maybe_output_path: &str,
    deploy_params: DeployStrParams<'_>,
    session_params: SessionStrParams<'_>,
    payment_params: PaymentStrParams<'_>,
    force: bool,
) -> Result<(), CliError> {
    let output = parse::output_kind(maybe_output_path, force);
    let deploy =
        deploy::with_payment_and_session(deploy_params, payment_params, session_params, true)?;
    crate::output_deploy(output, &deploy).map_err(CliError::from)
}

/// Creates a [`Transaction`] and outputs it to a file or stdout.
///
/// As a file, the `Transaction` can subsequently be signed by other parties using [`Transation`]
/// and then sent to the network for execution using [`Transaction`].
///
/// `maybe_output_path` specifies the output file path, or if empty, will print it to `stdout`.  If
/// `force` is true, and a file exists at `maybe_output_path`, it will be overwritten.  If `force`
/// is false and a file exists at `maybe_output_path`, [`Error::FileAlreadyExists`] is returned
/// and the file will not be written.
pub fn make_transaction(
    maybe_output_path: &str,
    transaction_str_params: TransactionStrParams<'_>,
    session_params: SessionStrParams<'_>,
    force: bool,
) -> Result<(), CliError> {
    let output = parse::output_kind(maybe_output_path, force);
    let transaction =
        transaction::create_transaction(transaction_str_params, session_params, true)?;
    crate::output_transaction(output, &transaction).map_err(CliError::from)
}

/// Reads a previously-saved [`Deploy`] from a file, cryptographically signs it, and outputs it to a
/// file or stdout.
///
/// `maybe_output_path` specifies the output file path, or if empty, will print it to `stdout`.  If
/// `force` is true, and a file exists at `maybe_output_path`, it will be overwritten.  If `force`
/// is false and a file exists at `maybe_output_path`, [`Error::FileAlreadyExists`] is returned
/// and the file will not be written.
pub fn sign_deploy_file(
    input_path: &str,
    secret_key_path: &str,
    maybe_output_path: &str,
    force: bool,
) -> Result<(), CliError> {
    let secret_key = parse::secret_key_from_file(secret_key_path)?;
    let output = parse::output_kind(maybe_output_path, force);
    crate::sign_deploy_file(input_path, &secret_key, output).map_err(CliError::from)
}

/// Reads a previously-saved [`Deploy`] from a file and sends it to the network for execution.
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters).
pub async fn send_deploy_file(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    input_path: &str,
) -> Result<SuccessResponse<PutDeployResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let deploy = crate::read_deploy_file(input_path)?;
    crate::put_deploy(rpc_id, node_address, verbosity, deploy)
        .await
        .map_err(CliError::from)
}

/// Reads a previously-saved [`Deploy`] from a file and sends it to the specified node for
/// speculative execution.
/// For details of the parameters, see [the module docs](crate::cli#common-parameters).
pub async fn speculative_send_deploy_file(
    maybe_block_id: &str,
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    input_path: &str,
) -> Result<SuccessResponse<SpeculativeExecResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let speculative_exec = parse::block_identifier(maybe_block_id)?;
    let verbosity = parse::verbosity(verbosity_level);
    let deploy = crate::read_deploy_file(input_path)?;
    crate::speculative_exec(rpc_id, node_address, speculative_exec, verbosity, deploy)
        .await
        .map_err(CliError::from)
}

/// Transfers funds between purses.
///
/// * `amount` is a string to be parsed as a `U512` specifying the amount to be transferred.
/// * `target_account` is the [`AccountHash`], [`URef`] or [`PublicKey`] of the account to which the
///   funds will be transferred, formatted as a hex-encoded string.  The account's main purse will
///   receive the funds.
/// * `transfer_id` is a string to be parsed as a `u64` representing a user-defined identifier which
///   will be permanently associated with the transfer.
///
/// For details of other parameters, see [the module docs](crate::cli#common-parameters).
#[allow(clippy::too_many_arguments)]
pub async fn transfer(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    amount: &str,
    target_account: &str,
    transfer_id: &str,
    deploy_params: DeployStrParams<'_>,
    payment_params: PaymentStrParams<'_>,
) -> Result<SuccessResponse<PutDeployResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let deploy = deploy::new_transfer(
        amount,
        None,
        target_account,
        transfer_id,
        deploy_params,
        payment_params,
        false,
    )?;
    crate::put_deploy(rpc_id, node_address, verbosity, deploy)
        .await
        .map_err(CliError::from)
}

/// Creates a [`Deploy`] to transfer funds between purses, and sends it to the specified node for
/// speculative execution.
///
/// * `amount` is a string to be parsed as a `U512` specifying the amount to be transferred.
/// * `target_account` is the [`AccountHash`], [`URef`] or [`PublicKey`] of the account to which the
///   funds will be transferred, formatted as a hex-encoded string.  The account's main purse will
///   receive the funds.
/// * `transfer_id` is a string to be parsed as a `u64` representing a user-defined identifier which
///   will be permanently associated with the transfer.
///
/// For details of other parameters, see [the module docs](crate::cli#common-parameters).
#[allow(clippy::too_many_arguments)]
pub async fn speculative_transfer(
    maybe_block_id: &str,
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    amount: &str,
    target_account: &str,
    transfer_id: &str,
    deploy_params: DeployStrParams<'_>,
    payment_params: PaymentStrParams<'_>,
) -> Result<SuccessResponse<SpeculativeExecResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let deploy = deploy::new_transfer(
        amount,
        None,
        target_account,
        transfer_id,
        deploy_params,
        payment_params,
        false,
    )?;
    let speculative_exec = parse::block_identifier(maybe_block_id)?;
    crate::speculative_exec(rpc_id, node_address, speculative_exec, verbosity, deploy)
        .await
        .map_err(CliError::from)
}

/// Creates a transfer [`Deploy`] and outputs it to a file or stdout.
///
/// As a file, the `Deploy` can subsequently be signed by other parties using [`sign_deploy_file`]
/// and then sent to the network for execution using [`send_deploy_file`].
///
/// `maybe_output_path` specifies the output file path, or if empty, will print it to `stdout`.  If
/// `force` is true, and a file exists at `maybe_output_path`, it will be overwritten.  If `force`
/// is false and a file exists at `maybe_output_path`, [`Error::FileAlreadyExists`] is returned
/// and the file will not be written.
pub fn make_transfer(
    maybe_output_path: &str,
    amount: &str,
    target_account: &str,
    transfer_id: &str,
    deploy_params: DeployStrParams<'_>,
    payment_params: PaymentStrParams<'_>,
    force: bool,
) -> Result<(), CliError> {
    let output = parse::output_kind(maybe_output_path, force);
    let deploy = deploy::new_transfer(
        amount,
        None,
        target_account,
        transfer_id,
        deploy_params,
        payment_params,
        true,
    )?;
    crate::output_deploy(output, &deploy).map_err(CliError::from)
}

/// Retrieves a [`Deploy`] from the network.
///
/// `deploy_hash` must be a hex-encoded, 32-byte hash digest.  For details of the other parameters,
/// see [the module docs](crate::cli#common-parameters).
pub async fn get_deploy(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    deploy_hash: &str,
    finalized_approvals: bool,
) -> Result<SuccessResponse<GetDeployResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let deploy_hash = parse::deploy_hash(deploy_hash)?;
    crate::get_deploy(
        rpc_id,
        node_address,
        verbosity,
        deploy_hash,
        finalized_approvals,
    )
    .await
    .map_err(CliError::from)
}

/// Retrieves a [`Block`] from the network.
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters).
pub async fn get_block(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    maybe_block_id: &str,
) -> Result<SuccessResponse<GetBlockResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let maybe_block_id = parse::block_identifier(maybe_block_id)?;
    crate::get_block(rpc_id, node_address, verbosity, maybe_block_id)
        .await
        .map_err(CliError::from)
}

/// Retrieves all [`Transfer`] items for a [`Block`] from the network.
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters).
pub async fn get_block_transfers(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    maybe_block_id: &str,
) -> Result<SuccessResponse<GetBlockTransfersResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let maybe_block_id = parse::block_identifier(maybe_block_id)?;
    crate::get_block_transfers(rpc_id, node_address, verbosity, maybe_block_id)
        .await
        .map_err(CliError::from)
}

/// Retrieves a state root hash at a given [`Block`].
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters).
pub async fn get_state_root_hash(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    maybe_block_id: &str,
) -> Result<SuccessResponse<GetStateRootHashResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let maybe_block_id = parse::block_identifier(maybe_block_id)?;
    crate::get_state_root_hash(rpc_id, node_address, verbosity, maybe_block_id)
        .await
        .map_err(CliError::from)
}

/// Retrieves era information from the network at a given [`Block`].
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters).
pub async fn get_era_summary(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    maybe_block_id: &str,
) -> Result<SuccessResponse<GetEraSummaryResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let maybe_block_id = parse::block_identifier(maybe_block_id)?;
    crate::get_era_summary(rpc_id, node_address, verbosity, maybe_block_id)
        .await
        .map_err(CliError::from)
}

/// Retrieves a [`StoredValue`] from global state.
///
/// `maybe_block_id` or `maybe_state_root_hash` identify the global state root hash to be used for
/// the query.  Exactly one of these args should be an empty string.
///
/// `key` must be a formatted [`PublicKey`] or [`Key`].  `path` is comprised of components starting
/// from the `key`, separated by `/`s.  It may be empty.
///
/// For details of other parameters, see [the module docs](crate::cli#common-parameters).
pub async fn query_global_state(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    maybe_block_id: &str,
    maybe_state_root_hash: &str,
    key: &str,
    path: &str,
) -> Result<SuccessResponse<QueryGlobalStateResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let global_state_identifier =
        parse::global_state_identifier(maybe_block_id, maybe_state_root_hash)?;
    let key = parse::key_for_query(key)?;
    let path = if path.is_empty() {
        vec![]
    } else {
        path.split('/').map(ToString::to_string).collect()
    };

    crate::query_global_state(
        rpc_id,
        node_address,
        verbosity,
        global_state_identifier,
        key,
        path,
    )
    .await
    .map_err(CliError::from)
}

/// Retrieves a purse's balance from global state.
///
/// `maybe_block_id` or `maybe_state_root_hash` identify the global state root hash to be used for
/// the query.  If both are empty, the latest block is used.
///
/// `purse_id` can be a properly-formatted public key, account hash or URef.
///
/// For details of other parameters, see [the module docs](crate::cli#common-parameters).
pub async fn query_balance(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    maybe_block_id: &str,
    maybe_state_root_hash: &str,
    purse_id: &str,
) -> Result<SuccessResponse<QueryBalanceResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let maybe_global_state_identifier =
        parse::global_state_identifier(maybe_block_id, maybe_state_root_hash)?;
    let purse_identifier = parse::purse_identifier(purse_id)?;

    crate::query_balance(
        rpc_id,
        node_address,
        verbosity,
        maybe_global_state_identifier,
        purse_identifier,
    )
    .await
    .map_err(CliError::from)
}

/// Retrieves a [`StoredValue`] from a dictionary at a given state root hash.
///
/// `state_root_hash` must be a hex-encoded, 32-byte hash digest.
///
/// `dictionary_item_str_params` contains dictionary item identifier options for this query.  See
/// [`DictionaryItemStrParams`] for more details.
///
/// For details of other parameters, see [the module docs](crate::cli#common-parameters).
pub async fn get_dictionary_item(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    state_root_hash: &str,
    dictionary_item_str_params: DictionaryItemStrParams<'_>,
) -> Result<SuccessResponse<GetDictionaryItemResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let state_root_hash =
        Digest::from_hex(state_root_hash).map_err(|error| CliError::FailedToParseDigest {
            context: "state root hash in get_dictionary_item",
            error,
        })?;
    let dictionary_item_identifier =
        DictionaryItemIdentifier::try_from(dictionary_item_str_params)?;

    crate::get_dictionary_item(
        rpc_id,
        node_address,
        verbosity,
        state_root_hash,
        dictionary_item_identifier,
    )
    .await
    .map_err(CliError::from)
}

/// Retrieves a purse's balance at a given state root hash.
///
/// `state_root_hash` must be a hex-encoded, 32-byte hash digest.
///
/// `purse` is a URef, formatted as e.g.
/// ```text
/// uref-0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20-007
/// ```
///
/// For details of other parameters, see [the module docs](crate::cli#common-parameters).
pub async fn get_balance(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    state_root_hash: &str,
    purse: &str,
) -> Result<SuccessResponse<GetBalanceResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let state_root_hash =
        Digest::from_hex(state_root_hash).map_err(|error| CliError::FailedToParseDigest {
            context: "state root hash in get_balance",
            error,
        })?;
    let purse = URef::from_formatted_str(purse).map_err(|error| CliError::FailedToParseURef {
        context: "purse in get_balance",
        error,
    })?;

    crate::get_balance(rpc_id, node_address, verbosity, state_root_hash, purse)
        .await
        .map_err(CliError::from)
}

/// Retrieves an [`Account`] at a given [`Block`].
///
/// `public_key` is the public key as a formatted string associated with the `Account`.
///
/// For details of other parameters, see [the module docs](crate::cli#common-parameters).
pub async fn get_account(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    maybe_block_id: &str,
    account_identifier: &str,
) -> Result<SuccessResponse<GetAccountResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let maybe_block_id = parse::block_identifier(maybe_block_id)?;
    let account_identifier = parse::account_identifier(account_identifier)?;

    crate::get_account(
        rpc_id,
        node_address,
        verbosity,
        maybe_block_id,
        account_identifier,
    )
    .await
    .map_err(CliError::from)
}

/// Retrieves the bids and validators at a given [`Block`].
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters).
pub async fn get_auction_info(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    maybe_block_id: &str,
) -> Result<SuccessResponse<GetAuctionInfoResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let maybe_block_id = parse::block_identifier(maybe_block_id)?;
    crate::get_auction_info(rpc_id, node_address, verbosity, maybe_block_id)
        .await
        .map_err(CliError::from)
}

/// Retrieves the status changes of the active validators on the network.
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters).
pub async fn get_validator_changes(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
) -> Result<SuccessResponse<GetValidatorChangesResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    crate::get_validator_changes(rpc_id, node_address, verbosity)
        .await
        .map_err(CliError::from)
}

/// Retrieves the IDs and addresses of the specified node's peers.
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters).
pub async fn get_peers(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
) -> Result<SuccessResponse<GetPeersResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    crate::get_peers(rpc_id, node_address, verbosity)
        .await
        .map_err(CliError::from)
}

/// Retrieves the status of the specified node.
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters).
pub async fn get_node_status(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
) -> Result<SuccessResponse<GetNodeStatusResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    crate::get_node_status(rpc_id, node_address, verbosity)
        .await
        .map_err(CliError::from)
}

/// Retrieves the Chainspec of the network.
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters).
pub async fn get_chainspec(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
) -> Result<SuccessResponse<GetChainspecResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    crate::get_chainspec(rpc_id, node_address, verbosity)
        .await
        .map_err(CliError::from)
}

/// Retrieves the interface description (the schema including examples in OpenRPC format) of the
/// JSON-RPC server's API.
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters).
pub async fn list_rpcs(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
) -> Result<SuccessResponse<ListRpcsResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    crate::list_rpcs(rpc_id, node_address, verbosity)
        .await
        .map_err(CliError::from)
}

/// JSON-encode and pretty-print the given value to stdout at the given verbosity level.
///
/// When `verbosity_level` is `0`, nothing is printed.  For `1`, the value is printed with long
/// string fields shortened to a string indicating the character count of the field.  Greater than
/// `1` is the same as for `1` except without abbreviation of long fields.
pub fn json_pretty_print<T: ?Sized + Serialize>(
    value: &T,
    verbosity_level: u64,
) -> Result<(), CliError> {
    let verbosity = parse::verbosity(verbosity_level);
    crate::json_pretty_print(value, verbosity).map_err(CliError::from)
}

/// Retrieves era information from the network at a given switch [`Block`].
///
/// For details of the parameters, see [the module docs](crate::cli#common-parameters).
#[deprecated(
    since = "2.0.0",
    note = "prefer 'get_era_summary' as it doesn't require a switch block"
)]
pub async fn get_era_info(
    maybe_rpc_id: &str,
    node_address: &str,
    verbosity_level: u64,
    maybe_block_id: &str,
) -> Result<SuccessResponse<GetEraInfoResult>, CliError> {
    let rpc_id = parse::rpc_id(maybe_rpc_id);
    let verbosity = parse::verbosity(verbosity_level);
    let maybe_block_id = parse::block_identifier(maybe_block_id)?;
    #[allow(deprecated)]
    crate::get_era_info(rpc_id, node_address, verbosity, maybe_block_id)
        .await
        .map_err(CliError::from)
}
