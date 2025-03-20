# Vault Canister

The Vault Canister serves as an asset manager for the entire Quotex Protocol infrastructure. Each vault houses one collateral asset and its corresponding stake token. This document provides a detailed explanation of the Vault Canister's functionality, including depositing tokens, trading, providing leverage, and staking.

## Mechanism of Operation

### Depositing Tokens

Individuals can deposit tokens into the vault by approving spending by the vault and calling the `deposit` function. This process involves the following steps:
1. **Approval**: The user must first approve the Vault Canister to spend their tokens.
2. **Deposit**: The user then calls the `deposit` function to transfer the tokens into the vault.

After depositing, users have two main options:
- **Trading**: Use the deposited tokens to trade on supported markets.
- **Leverage Providing**: Act as leverage providers to earn interest.

### Trading

When trading on any market, the market calls the Vault Canister to lock up the required amount of collateral from the user's account before opening a position. This ensures that the necessary collateral is secured for the trade. Note that only markets with the vault-specified token as collateral are supported. If the user is trading on leverage, it also locks up the amount specified as leverage if it is available.

### Providing Leverage

Depositors can act as leverage providers by providing liquidity for traders to trade with leverage. These liquidity providers receive interest hourly. The process is as follows:
1. **Provide Liquidity**: Users provide liquidity by depositing tokens into the vault.
2. **Receive qTokens**: Users receive the same amount of qTokens of their respective asset (e.g., qICP for ICP) in a 1:1 ratio.
3. **Earn Interest**: Liquidity providers earn interest hourly based on the amount of liquidity provided.

These qTokens can be further staked in one of the available stake spans to earn better yields.

## Staking

The current stake spans available are:
- **2 months**: Offers a base yield.
- **6 months**: Offers a higher yield than the 2-month span.
- **12 months**: Offers the highest yield among the available spans.

The prospective yields increase in the same order, providing better returns for longer staking periods.

## Example Code

### Deposit Function

Below is an example of how to use the `deposit` function to deposit tokens into the vault:
