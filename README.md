# Staked Token Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [UpdateStakeConfig](#UpdateStakeConfig)
            * [SetDistributorsStatus](#SetDistributorsStatus)
            * [AddDistributors](#AddDistributors)
            * [SetDistributors](#SetDistributors)
    * [User](#User)
        * Messages
            * [Account](#Account)
            * [DisablePermitKey](#DisablePermitKey)
            * [Claim](#Claim)
        * Queries
            * [Receive](#Receive)
              * [Bond](#Bond)
              * [Reward](#Reward)
              * [Unbond](#Unbond)
            * [Dates](#Dates)
            * [TotalClaimed](#TotalClaimed)
            * [Account](#Account)

# Introduction
Contract responsible to handle snip20 staking
