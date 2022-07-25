use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token;
use anchor_spl::token::{MintTo, Token}; // CPI (Cross-Program Interface) API for interacting with the SPL (Solana Program Library) shared memory program.
use mpl_token_metadata::instruction::{
    create_master_edition_v3, create_metadata_accounts_v2,
};

declare_id!("F7ouUnvdPYxiuyjUdaq9mN6n6E41Uh6u9FgjKTEiYHx5");  // Replace with your program ID on the Solana network you are using (most likely the devnet network).

#[derive(Accounts)]
pub struct MasterEditionNFT<'info> {
    // TODO? Change name to MintMasterEditionNFT?
    #[account(mut)]
    pub mint_authority: Signer<'info>, // ? has authority to do the mint? Who is it?
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    // #[account(mut)]
    pub token_program: Program<'info, Token>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_metadata_program: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub payer: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub rent: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
}

#[program]
pub mod master_edition_example {
    use super::*;

    pub fn create_master_edition(
        // Many tutorials will call this function `mint_nft` instead of `create_master_edition`. 
        // Although it is minting an NFT, I call it `create_master_edition` which is more descriptive of the NFT type created.
        // Below we define a function for creating a "copy" of the Master Edition, simply called Edition NFT. These Edition NFT are sometimes referred to as "Prints" of the Master Edition.
        // To read more about Master Edition and Edition NFT, refer to: https://docs.metaplex.com/programs/token-metadata/accounts
        ctx: Context<MasterEditionNFT>,
        creator_key: Pubkey,
        uri: String,
        title: String,
    ) -> Result<()> {

        // Step 1. We mint a new token that will represent the NFT on the blockchain.

        // 1.a. Here we initiate a struct with the accounts relevant for the minting.
        // 1.a. The MintTo structure to follow is given by Metaplex.
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(), // This is the mint account.
            to: ctx.accounts.token_account.to_account_info(), // This is the token account that will receive the token.
            authority: ctx.accounts.mint_authority.to_account_info(), // This refers to the account with the power to mint tokens.
        };

        // 1.b. We will then call the Solana Token Program to mint the token. We essentially need to ask the Solana program to mint a token on our behalf.
        
        // 1.b.1. To do so, we need to represent the Token Program as a struct compatible with anchor. This struct is called AccountInfo.
        // 1.b.1. To see the content of the AccountInfo struct, refer to: https://docs.rs/anchor-lang/0.9.0/anchor_lang/prelude/struct.AccountInfo.html
        let cpi_program = ctx.accounts.token_program.to_account_info(); 

        // 1.b.2. We define the context in order to specify non-argument inputs for cross-program-invocations (From the official doc: https://docs.rs/anchor-lang/0.6.0/anchor_lang/struct.CpiContext.html)
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // 1.b.3. And finally we mint the token.
        token::mint_to(cpi_ctx, 1)?; // We mint 1 token to token_account. This token is a token of the Mint account. It represents 1 NFT. The uniqueness of the NFT is guaranteed by the Master Edition.
        
        msg!("Token Minted !!!");
        // Step 1. is over. We minted one token. 

        
        // Step 2. We create the Metadata account associated with the Mint account. 
        // The Metadata Account is responsible for storing additional data attached to tokens. Read more: https://docs.metaplex.com/programs/token-metadata/accounts
        
        // The structures below are the one required by the Metaplex program.
        let account_info = vec![
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        let creator = vec![
            mpl_token_metadata::state::Creator {
                address: creator_key,
                verified: false,
                share: 100, 
            },
            mpl_token_metadata::state::Creator {
                address: ctx.accounts.mint_authority.key(),
                verified: false,
                share: 0, 
            },
        ];

        let symbol = std::string::ToString::to_string("symb"); // TODO

        invoke( 
            // `Invoke` enables us to run a cross-program instruction (The official doc: https://docs.rs/solana-program/1.6.2/solana_program/program/fn.invoke.html)
            // We will use a Program from Metaplex that will create the Metadata account for us. Read more: https://docs.metaplex.com/programs/token-metadata/instructions#create-a-metadata-account
            // The Rust documentation of the program: https://docs.rs/mpl-token-metadata/1.2.0/mpl_token_metadata/instruction/fn.create_metadata_accounts_v2.html
            &create_metadata_accounts_v2( 
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.mint.key(),
                ctx.accounts.mint_authority.key(),
                ctx.accounts.payer.key(),
                ctx.accounts.payer.key(),
                title,
                symbol,
                uri,
                Some(creator),
                0, // By setting to 0, we do not take any fee
                true,
                false,
                None,
                None,
            ),
            account_info.as_slice(), // TODO
        )?;

        msg!("Metadata Account Created !!!"); 
        // Step 2. is over. We create the Metadata account associated to the Mint account (and thus associated to the Token account). 

        // Step 3. We create the Master Edition NFT
        // Now that we have our Token account with one token, and our Metadata account setup, we can create the Master Edition account.
        // The Mater Edition guarantees that we have an NFT. (Read more: https://docs.metaplex.com/programs/token-metadata/accounts)

        // The structures below are the one required by the Metaplex program.
        let master_edition_infos = vec![
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        invoke( // Similar to Step 2.
            &create_master_edition_v3(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.master_edition.key(),
                ctx.accounts.mint.key(),
                ctx.accounts.payer.key(),
                ctx.accounts.mint_authority.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.payer.key(),
                None, // When set to None, an unlimited amount of copies can be created ; When set to Some(0) => 0 NFT cannot be copied ; When set to Some(1) => Only 1 NFT cannot be copied ; Etc.
            ),
            master_edition_infos.as_slice(),
        )?;
        msg!("Master Edition Nft Minted !!!"); 
        // Step 3. is over. That's it, our Master Edition NFT has been created.

        Ok(())
    }
}
