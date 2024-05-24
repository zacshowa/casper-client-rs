use async_trait::async_trait;
use clap::{ArgMatches, Command};

use casper_client::cli::{CliError, DeployStrParams};

use super::{creation_common, transfer};
use crate::{command::ClientCommand, common, Success};

pub struct MakeTransfer;

#[async_trait]
impl ClientCommand for MakeTransfer {
    const NAME: &'static str = "make-transfer";
    const ABOUT: &'static str =
        "Create a transfer deploy and output it to a file or stdout. As a file, the deploy can \
        subsequently be signed by other parties using the 'sign-deploy' subcommand and then sent \
        to the network for execution using the 'send-deploy' subcommand";

    fn build(display_order: usize) -> Command {
        let subcommand = Command::new(Self::NAME)
            .about(Self::ABOUT)
            .display_order(display_order)
            .arg(creation_common::output::arg())
            .arg(transfer::amount::arg())
            .arg(transfer::target_account::arg())
            .arg(transfer::transfer_id::arg())
            .arg(common::force::arg(
                creation_common::DisplayOrder::Force as usize,
                true,
            ));
        let subcommand = creation_common::apply_common_payment_options(
            subcommand,
            Some(common::DEFAULT_TRANSFER_PAYMENT_AMOUNT),
        );
        creation_common::apply_common_creation_options(subcommand, false, false)
    }

    async fn run(matches: &ArgMatches) -> Result<Success, CliError> {
        creation_common::show_simple_arg_examples_and_exit_if_required(matches);
        creation_common::show_json_args_examples_and_exit_if_required(matches);

        let gas_price = creation_common::gas_price_tolerance::get(matches);

        let amount = transfer::amount::get(matches);
        let target_account = transfer::target_account::get(matches);
        let transfer_id = transfer::transfer_id::get(matches);

        let secret_key = common::secret_key::get(matches).unwrap_or_default();
        let timestamp = creation_common::timestamp::get(matches);
        let ttl = creation_common::ttl::get(matches);
        let chain_name = creation_common::chain_name::get(matches);

        let payment_str_params = creation_common::payment_str_params(matches);

        let maybe_output_path = creation_common::output::get(matches).unwrap_or_default();
        let session_account = creation_common::session_account::get(matches)?;
        let force = common::force::get(matches);

        casper_client::cli::make_transfer(
            maybe_output_path,
            amount,
            target_account,
            transfer_id,
            DeployStrParams {
                secret_key,
                timestamp,
                ttl,
                chain_name,
                session_account: &session_account,
                gas_price_tolerance: gas_price,
            },
            payment_str_params,
            force,
        )
        .map(|_| {
            Success::Output(if maybe_output_path.is_empty() {
                String::new()
            } else {
                format!("Wrote the transfer deploy to {}", maybe_output_path)
            })
        })
    }
}
