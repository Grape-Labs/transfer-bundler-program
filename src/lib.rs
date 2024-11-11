#![cfg(not(feature = "no-entrypoint"))]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program::{invoke},
    program_error::ProgramError,
    pubkey::Pubkey,
    msg,
};

use spl_token::instruction::transfer as token_transfer;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Transfer {
    pub amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TransferInstructionData {
    pub transfers: Vec<Transfer>,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Deserialize instruction data
    let transfer_instruction_data = TransferInstructionData::try_from_slice(instruction_data)?;
    let num_transfers = transfer_instruction_data.transfers.len();

    let expected_accounts = num_transfers * 2 + 2; // Each transfer requires 2 accounts, plus authority and token_program
    if accounts.len() < expected_accounts {
        msg!("Insufficient accounts provided.");
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let account_info_iter = &mut accounts.iter();

    // Authority and token program accounts
    let authority_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    for (i, transfer) in transfer_instruction_data.transfers.iter().enumerate() {
        // Accounts for each transfer
        let source_account = next_account_info(account_info_iter)?;
        let destination_account = next_account_info(account_info_iter)?;

        msg!("Performing transfer {} of {}", i + 1, num_transfers);
        let ix = token_transfer(
            token_program.key,
            source_account.key,
            destination_account.key,
            authority_account.key,
            &[],
            transfer.amount,
        )?;

        invoke(
            &ix,
            &[
                source_account.clone(),
                destination_account.clone(),
                authority_account.clone(),
                token_program.clone(),
            ],
        )?;
    }

    Ok(())
}

