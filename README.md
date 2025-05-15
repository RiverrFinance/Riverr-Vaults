# Vault Canister

The Vault Canister serves as an asset manager for the entire Quotex Protocol infrastructure. Each vault houses one collateral asset and its corresponding lock token. This document provides a detailed explanation of the Vault Canister's functionality, including depositing tokens, trading, providing leverage, and staking.

## Mechanism of Operation

### Depositing Tokens

Individuals can deposit tokens into the vault by approving spending by the vault and calling the `deposit` function. This process involves the following steps:

1. **Approval**: The user must first approve the Vault Canister to spend their tokens.
2. **Deposit**: The user then calls the `deposit` function to transfer the tokens into the vault.

After depositing, users have two main options:

- **Trading**: Use the deposited tokens to trade on supported markets.
- **Leverage Provision**: Act as leverage providers to earn interest.
- **Locking Qtokens**: Lock Qtokens for some specified span of time and earn more from fees .

## Trading
<p>When trading on any market, the market calls the Vault Canister to lock up the required amount of collateral from the user's account before opening a position. This ensures that the necessary collateral is secured for the trade. Note that only markets with the vault-specified token as collateral are supported. If the user is trading on leverage, it also locks up the amount specified as leverage if it is available.</p>

## Leverage Provision
Depositors can act as Leverage providers and provide their liquidity to be utilised by traders as leverage in their positions.The Traders the interest rate is calculated on an hourly basis ,but is only repaid when the trader position is closed or liquidated .

### QTokens
Leverage  providers get the Qtoken for providing leverage ,this is basically a 1:1 virtual token representing the amount of assets deposited i.e Users get exactly
100 Qtokens for depositing 100 tokens .Also note fees are paid in qtokens.
### To Provide Leverage
Users provide leverage by calling the provide leverage function and the same amount of Qtokens gets minted and sent to user

### To Withdraw Leverage
Users call ICRC2 approval function on the Qtoken to approve spending by the vault canister,specifying the amount and then call withdraw leverage function on the vault canister <br>
  <b>NOTE</b> : Withdrawals can only be made if the vault has that amount of liqudity available at that time,which is dependent on the current pool utilization rate baiscally the amount of debt owned by traders comapred to the total amount provided by leverage providers

## Locking QTokens
<p>Users can lockup QTokens gotten from providing leverage for a specific peroid of time and earn greater yield.same can be done too with QTokens gotten from external mrkets</P>
The current lock time spans available are: 
- **2 months**: Offers a base yield.
- **6 months**: Offers a higher yield than the 2-month span.
- **12 months**: Offers the highest yield among the available spans.
The prospective yields increase in the same order, providing better returns for longer staking periods.
