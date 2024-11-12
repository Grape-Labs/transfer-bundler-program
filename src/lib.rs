#![cfg(not(feature = "no-entrypoint"))]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use spl_token::{instruction::transfer as token_transfer, id as token_program_id};

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
    
    // Calculate expected number of accounts
    let expected_accounts = num_transfers * 2 + 2; // Each transfer requires 2 accounts, plus authority and token_program
    if accounts.len() != expected_accounts {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Authority and token program accounts
    let authority_account = &accounts[0];
    let token_program = &accounts[1];

    // Verify token program and authority once outside the loop
    if token_program.key != &token_program_id() {
        return Err(ProgramError::InvalidArgument);
    }
    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Process each transfer
    for (i, transfer) in transfer_instruction_data.transfers.iter().enumerate() {
        let source_account = &accounts[2 + i * 2];
        let destination_account = &accounts[3 + i * 2];

        // Create transfer instruction
        let ix = token_transfer(
            token_program.key,
            source_account.key,
            destination_account.key,
            authority_account.key,
            &[],                  // Signer seeds, if any
            transfer.amount,      // Amount to transfer
        )?;

        // Execute the transfer instruction
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