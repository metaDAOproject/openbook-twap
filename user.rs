use anchor_lang::prelude::*;

pub const USER: &[u8] = b"user";

#[account]
#[derive(Default)]
pub struct User {
    pub owner: Pubkey,
    pub username: [u8; 16], 
    pub total_deposits: u64, 
    pub total_withdraws: u64, 
    pub incoming_tx: u32,
    pub outgoing_tx: u32,
    pub num_groups: u32,
    pub preferred_token: Option<Pubkey>,
}

impl User {
    pub fn new(
        &mut self,
        owner: Pubkey,
        username: [u8; 16],
    ) -> Result<()> {
        self.owner = owner;
        self.username = username;
        Ok(())
    }

    pub fn get_user_address(pubkey: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                USER,
                pubkey.as_ref(),
            ],
            &crate::ID,
        )
    }
    
    pub fn get_user_signer_seeds<'a>(
        pubkey: &'a Pubkey, 
        bump: &'a u8
    ) -> [&'a [u8]; 3] {
        [USER.as_ref(), pubkey.as_ref(), bytemuck::bytes_of(bump)]
    }

    pub fn increment_outgoing_transactions(&mut self) {
        self.outgoing_tx = self.outgoing_tx.saturating_add(1);
    }

    pub fn increment_incoming_transactions(&mut self) {
        self.incoming_tx = self.incoming_tx.saturating_add(1);
    }
    
    pub fn increment_withdrawals(&mut self) {
        self.total_withdraws = self.total_withdraws.saturating_add(1);
    }

    pub fn increment_groups(&mut self) {
        self.num_groups = self.num_groups.saturating_add(1);
    }

    pub fn set_preferred_token(&mut self, token: Pubkey) {
        self.preferred_token = Some(token);
    }

    pub fn disable_preferred_token(&mut self) {
        self.preferred_token = None;
    }
}
