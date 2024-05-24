use std::{fs, str};

use async_trait::async_trait;
use clap::{Arg, ArgGroup, ArgMatches, Command};

use casper_client::cli::{CliError, DictionaryItemStrParams};
use casper_types::PublicKey;

use crate::{command::ClientCommand, common, Success};

pub struct GetDictionaryItem;

/// This struct defines the order in which the args are shown for this subcommand's help message.
enum DisplayOrder {
    Verbose,
    NodeAddress,
    RpcId,
    StateRootHash,
    AccountHash,
    ContractHash,
    EntityAddr,
    DictionaryName,
    DictionaryItemKey,
    DictionarySeedURef,
    DictionaryAddress,
}

/// Handles providing the arg for and retrieval of the key.
mod key {
    use casper_types::AsymmetricType;

    use super::*;

    const ARG_VALUE_NAME: &str = "FORMATTED STRING or PATH";

    pub(super) fn arg(arg_name: &'static str, arg_help: &'static str, display_order: usize) -> Arg {
        Arg::new(arg_name)
            .long(arg_name)
            .required(false)
            .value_name(ARG_VALUE_NAME)
            .help(arg_help)
            .display_order(display_order)
    }

    pub(super) fn get(arg_name: &'static str, matches: &ArgMatches) -> Result<String, CliError> {
        let value = matches
            .get_one::<String>(arg_name)
            .map(String::as_str)
            .unwrap_or_default();

        // Try to read as a PublicKey PEM file first.
        if let Ok(public_key) = PublicKey::from_file(value) {
            return Ok(public_key.to_hex());
        }

        // Try to read as a hex-encoded PublicKey file next.
        if let Ok(hex_public_key) = fs::read_to_string(value) {
            let _ = PublicKey::from_hex(&hex_public_key).map_err(|error| {
                eprintln!(
                    "Can't parse the contents of {} as a public key: {}",
                    value, error
                );
                CliError::FailedToParsePublicKey {
                    context: "get dictionary item public key".to_string(),
                    error,
                }
            })?;
            return Ok(hex_public_key);
        }

        // Just return the value.
        Ok(value.to_string())
    }
}

mod account_hash {
    use super::*;

    pub(crate) const ARG_NAME: &str = "account-hash";
    const ARG_HELP: &str =
        "This must be a properly formatted account hash. The format for account hash is \
        \"account-hash-<HEX STRING>\".";

    pub(super) fn arg() -> Arg {
        key::arg(ARG_NAME, ARG_HELP, DisplayOrder::AccountHash as usize)
    }

    pub(super) fn get(matches: &ArgMatches) -> Result<String, CliError> {
        key::get(ARG_NAME, matches)
    }
}

mod contract_hash {
    use super::*;

    pub(crate) const ARG_NAME: &str = "contract-hash";
    const ARG_HELP: &str =
        "This must be a properly formatted contract hash. The format for contract hash is \
        \"hash-<HEX STRING>\".";

    pub(super) fn arg() -> Arg {
        key::arg(ARG_NAME, ARG_HELP, DisplayOrder::ContractHash as usize)
    }

    pub(super) fn get(matches: &ArgMatches) -> Result<String, CliError> {
        key::get(ARG_NAME, matches)
    }
}

mod entity_addr {
    use super::*;

    pub(crate) const ARG_NAME: &str = "entity-addr";
    const ARG_HELP: &str = "This must be a properly formatted entity address. The format is \
        \"entity-contract-<HEX STRING>\" for contracts and \"entity-account-<HEX STRING>\" for \
        accounts.";

    pub(super) fn arg() -> Arg {
        key::arg(ARG_NAME, ARG_HELP, DisplayOrder::EntityAddr as usize)
    }

    pub(super) fn get(matches: &ArgMatches) -> Result<String, CliError> {
        key::get(ARG_NAME, matches)
    }
}

/// Handles providing the arg for the named key under which the dictionary seed URef is stored.
mod dictionary_name {
    use super::*;

    pub(crate) const ARG_NAME: &str = "dictionary-name";
    const ARG_VALUE_NAME: &str = "STRING";
    const ARG_HELP: &str = "The named key under which the dictionary seed URef is stored.";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .required_unless_present_any([seed_uref::ARG_NAME, dictionary_address::ARG_NAME])
            .value_name(ARG_VALUE_NAME)
            .help(ARG_HELP)
            .display_order(DisplayOrder::DictionaryName as usize)
    }

    pub(super) fn get(matches: &ArgMatches) -> &str {
        matches
            .get_one::<String>(ARG_NAME)
            .map(String::as_str)
            .unwrap_or_default()
    }
}

/// Handles providing the arg for name of the key under which the dictionary item is stored.
mod dictionary_item_key {
    use super::*;

    pub(crate) const ARG_NAME: &str = "dictionary-item-key";
    const ARG_VALUE_NAME: &str = "STRING";
    const ARG_HELP: &str = "The dictionary item key formatted as a string.";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .required_unless_present(dictionary_address::ARG_NAME)
            .value_name(ARG_VALUE_NAME)
            .help(ARG_HELP)
            .display_order(DisplayOrder::DictionaryItemKey as usize)
    }

    pub(super) fn get(matches: &ArgMatches) -> &str {
        matches
            .get_one::<String>(ARG_NAME)
            .map(String::as_str)
            .unwrap_or_default()
    }
}

/// Handles providing the arg for and retrieval of the dictionary's seed URef.
mod seed_uref {
    use super::*;

    pub(crate) const ARG_NAME: &str = "seed-uref";
    const ARG_VALUE_NAME: &str = "FORMATTED STRING";
    const ARG_HELP: &str = "The dictionary's seed URef. This must be a properly formatted URef \
        \"uref-<HEX STRING>-<THREE DIGIT INTEGER>\"";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .required(false)
            .value_name(ARG_VALUE_NAME)
            .help(ARG_HELP)
            .display_order(DisplayOrder::DictionarySeedURef as usize)
    }

    pub(super) fn get(matches: &ArgMatches) -> &str {
        matches
            .get_one::<String>(ARG_NAME)
            .map(String::as_str)
            .unwrap_or_default()
    }
}

/// Handles providing the arg for and retrieval of the Dictionary address.
mod dictionary_address {
    use super::*;

    pub(crate) const ARG_NAME: &str = "dictionary-address";
    const ARG_VALUE_NAME: &str = "FORMATTED STRING";
    const ARG_HELP: &str = "The dictionary item's unique key.";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .required(false)
            .value_name(ARG_VALUE_NAME)
            .help(ARG_HELP)
            .display_order(DisplayOrder::DictionaryAddress as usize)
    }

    pub(super) fn get(matches: &ArgMatches) -> &str {
        matches
            .get_one::<String>(ARG_NAME)
            .map(String::as_str)
            .unwrap_or_default()
    }
}

#[async_trait]
impl ClientCommand for GetDictionaryItem {
    const NAME: &'static str = "get-dictionary-item";
    const ABOUT: &'static str = "Retrieve a stored value from a dictionary";

    fn build(display_order: usize) -> Command {
        Command::new(Self::NAME)
            .about(Self::ABOUT)
            .display_order(display_order)
            .arg(common::verbose::arg(DisplayOrder::Verbose as usize))
            .arg(common::node_address::arg(
                DisplayOrder::NodeAddress as usize,
            ))
            .arg(common::rpc_id::arg(DisplayOrder::RpcId as usize))
            .arg(common::state_root_hash::arg(
                DisplayOrder::StateRootHash as usize,
                true,
            ))
            .arg(account_hash::arg())
            .arg(contract_hash::arg())
            .arg(entity_addr::arg())
            .arg(seed_uref::arg())
            .arg(dictionary_address::arg())
            .arg(dictionary_name::arg())
            .arg(dictionary_item_key::arg())
            .group(
                ArgGroup::new("dictionary-identifier")
                    .arg(account_hash::ARG_NAME)
                    .arg(contract_hash::ARG_NAME)
                    .arg(entity_addr::ARG_NAME)
                    .arg(seed_uref::ARG_NAME)
                    .arg(dictionary_address::ARG_NAME)
                    .required(true),
            )
    }

    async fn run(matches: &ArgMatches) -> Result<Success, CliError> {
        let maybe_rpc_id = common::rpc_id::get(matches);
        let node_address = common::node_address::get(matches);
        let verbosity_level = common::verbose::get(matches);
        let state_root_hash = common::state_root_hash::get(matches)
            .unwrap_or_else(|| panic!("should have {} arg", common::state_root_hash::ARG_NAME));

        let account_hash = account_hash::get(matches)?;
        let contract_hash = contract_hash::get(matches)?;
        let entity_addr = entity_addr::get(matches)?;
        let dictionary_name = dictionary_name::get(matches);
        let seed_uref = seed_uref::get(matches);
        let dictionary_key = dictionary_address::get(matches);
        let dictionary_item_key = dictionary_item_key::get(matches);

        let dictionary_item_str_params = if !account_hash.is_empty() && !dictionary_name.is_empty()
        {
            DictionaryItemStrParams::AccountNamedKey {
                account_hash: &account_hash,
                dictionary_name,
                dictionary_item_key,
            }
        } else if !contract_hash.is_empty() && !dictionary_name.is_empty() {
            DictionaryItemStrParams::ContractNamedKey {
                hash_addr: &contract_hash,
                dictionary_name,
                dictionary_item_key,
            }
        } else if !entity_addr.is_empty() && !dictionary_name.is_empty() {
            DictionaryItemStrParams::EntityNamedKey {
                entity_addr: &entity_addr,
                dictionary_name,
                dictionary_item_key,
            }
        } else if !seed_uref.is_empty() {
            DictionaryItemStrParams::URef {
                seed_uref,
                dictionary_item_key,
            }
        } else if !dictionary_key.is_empty() {
            DictionaryItemStrParams::Dictionary(dictionary_key)
        } else {
            return Err(CliError::InvalidArgument {
                context: "dictionary item identifier",
                error: "mismatch of args".to_string(),
            });
        };

        casper_client::cli::get_dictionary_item(
            maybe_rpc_id,
            node_address,
            verbosity_level,
            state_root_hash,
            dictionary_item_str_params,
        )
        .await
        .map(Success::from)
    }
}
