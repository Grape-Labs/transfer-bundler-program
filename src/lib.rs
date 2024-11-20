#![cfg(not(feature = "no-entrypoint"))]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction::transfer as system_transfer,
    system_program,
};
use spl_token::{instruction::transfer as token_transfer, id as token_program_id};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum InstructionType {
    TokenTransfer(TransferInstructionData),
    NativeSolTransfer(NativeSolTransferData),
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Transfer {
    pub amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TransferInstructionData {
    pub transfers: Vec<Transfer>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct NativeSolTransferData {
    pub transfers: Vec<Transfer>,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction_type: InstructionType = InstructionType::try_from_slice(instruction_data)?;

    match instruction_type {
        InstructionType::TokenTransfer(data) => process_token_transfers(accounts, data),
        InstructionType::NativeSolTransfer(data) => process_native_sol_transfers(accounts, data),
    }
}

fn process_token_transfers(
    accounts: &[AccountInfo],
    transfer_instruction_data: TransferInstructionData,
) -> ProgramResult {
    let num_transfers = transfer_instruction_data.transfers.len();
    let expected_accounts = num_transfers * 2 + 2; // Each transfer requires 2 accounts, plus authority and token_program

    if accounts.len() != expected_accounts {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

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

        let ix = token_transfer(
            token_program.key,
            source_account.key,
            destination_account.key,
            authority_account.key,
            &[],                  // Signer seeds, if any
            transfer.amount,      // Amount to transfer
        )?;

        invoke(
            &ix,
            &[source_account.clone(), destination_account.clone(), authority_account.clone(), token_program.clone()],
        )?;
    }

    Ok(())
}

fn process_native_sol_transfers(
    accounts: &[AccountInfo],
    transfer_data: NativeSolTransferData,
) -> ProgramResult {
    let num_transfers = transfer_data.transfers.len();
    let expected_accounts = num_transfers * 2 + 1; // Each transfer requires 2 accounts, plus 1 system program

    if accounts.len() != expected_accounts {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Check if the first account is the system program
    let system_program_account = &accounts[0];
    if system_program_account.key != &system_program::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Process each transfer
    for (i, transfer) in transfer_data.transfers.iter().enumerate() {
        let source_account = &accounts[i * 2 + 1];
        let destination_account = &accounts[i * 2 + 2];

        if !source_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let ix = system_transfer(
            source_account.key,
            destination_account.key,
            transfer.amount,
        );

        invoke(&ix, &[source_account.clone(), destination_account.clone(), system_program_account.clone()])?;
    }

    Ok(())
}