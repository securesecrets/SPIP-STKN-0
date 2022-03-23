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
            * [ContractStatus](#ContractStatus)
    * [User](#User)
        * Messages
            * [Receive](#Receive)
                * [Bond](#Bond)
                * [Reward](#Reward)
                * [Unbond](#Unbond)
            * [Unbond](#Unbond)
            * [ClaimUnbond](#ClaimUnbond)
            * [ClaimRewards](#ClaimRewards)
            * [StakeRewards](#StakeRewards)
            * [ExposeBalance](#ExposeBalance)
            * [ExposeBalanceWithCooldown](#ExposeBalanceWithCooldown)
        * Queries
            * [StakeConfig](#StakeConfig)
            * [TotalStaked](#TotalStaked)
            * [StakeRate](#StakeRate)
            * [Unbonding](#Unbonding)
            * [Unfunded](#Unfunded)
            * [Staked](#Staked)
            * [Distributors](#Distributors)
            * [WithPermit](#WithPermit)
              * [Staked](#StakedWithPermit)
            
# Introduction
Allows for Snip20 liquid staking

# Sections

## Init
##### Request
| Name                | Type         | Description                                                                | optional |
|---------------------|--------------|----------------------------------------------------------------------------|----------|
| name                | string       | Name of the staked token                                                   | No       |
| admin               | string       | Smart contract admin                                                       | Yes      |
| symbol              | string       | Staked token symbol                                                        | No       |
| decimals            | u8           | The staked tokens decimals, will copy from the contract if not set         | Yes      |
| share_decimals      | u8           | Must be more than 2x the decimals amount, needed for precision             | No       |
| prng_seed           | string       | Base64 encoded seed for the random key generation                          | No       |
| public_total_supply | bool         | Variable to determine if the tokens total supply is public                 | No       |
| unbond_time         | u64          | Waiting time for unbonding                                                 | No       |
| staked_token        | Contract     | Address and hash of the staked token                                       | No       |
| treasury            | string       | Address of the treasury involved                                           | Yes      |
| limit_transfers     | bool         | Limits to where the staked tokens can be sent, useful for maintenance      | No       |
| distributors        | string array | Addresses allowed to be transferred to or from when transfers are limited  | Yes      |

##Admin

### Messages

#### UpdateStakeConfig
Updated the staked tokens configuration file
##### Request
| Name             | Type   | Description                           | optional |
|------------------|--------|---------------------------------------|----------|
| unbond_time      | u64    | Changes the unbonding time            | yes      |
| disable_treasury | bool   | Disables the current treasury address | no       |
| treasury         | string | Replaces the treasury address         | yes      |
| padding          | string | Used to pad messages                  | yes      |

#### SetDistributorsStatus
Sets if distributor limits are enabled or not
##### Request
| Name    | Type   | Description                  | optional |
|---------|--------|------------------------------|----------|
| enables | bool   | Sets the distribution status | no       |
| padding | string | Used to pad messages         | yes      |

#### AddDistributors
Adds distributors to the list of allowed addresses
##### Request
| Name         | Type         | Description                                    | optional |
|--------------|--------------|------------------------------------------------|----------|
| distributors | string array | Adds the addresses to the allowed distributors | no       |
| padding      | string       | Used to pad messages                           | yes      |

#### SetDistributors
Sets distributors to the list of allowed addresses
##### Request
| Name         | Type         | Description                                         | optional |
|--------------|--------------|-----------------------------------------------------|----------|
| distributors | string array | Sets the new addresses for the allowed distributors | no       |
| padding      | string       | Used to pad messages                                | yes      |

#### SetContractStatus
Can limit certain contract interactions for maintenance and security purposes. Status levels are NormalRun,
StopBonding, StopAllButUnbond, StopAll
##### Request
| Name    | Type                | Description                  | optional |
|---------|---------------------|------------------------------|----------|
| level   | ContractStatusLevel | Stops certain contract logic | no       |
| padding | string              | Used to pad messages         | yes      |

##User

### Messages

#### Receive
Snip20 function used to interact with a contract that just got tokens sent to it
##### Request
| Name    | Type   | Description                                          | optional |
|---------|--------|------------------------------------------------------|----------|
| sender  | string | The send msg signer                                  | no       |
| from    | string | Funds provider                                       | no       |
| amount  | string | Amount received                                      | no       |
| msg     | string | Base64 encoded msg that contains the type of deposit | yes      |
| memo    | string | Additional written context for the tx                | yes      |
| padding | string | Used to pad messages                                 | yes      |

###### Msg Type
####### Bond
Used for bonding the send tokens, can add a `useFrom` tag to specify the usage of `from` instead of `sender`
####### Rewards
Adds the tokens as bonding rewards
####### Unbond
Adds the tokens as unbonding amounts

#### Unbond
Unbonds the given amount, must be less or equal than the users staked amount
##### Request
| Name    | Type   | Description          | optional |
|---------|--------|----------------------|----------|
| amount  | strung | amount to unbond     | no       |
| padding | string | Used to pad messages | yes      |

#### ClaimUnbond
Claims the unbonded amount
##### Request
| Name    | Type   | Description          | optional |
|---------|--------|----------------------|----------|
| padding | string | Used to pad messages | yes      |

#### ClaimRewards
Claims the available rewards
##### Request
| Name    | Type   | Description          | optional |
|---------|--------|----------------------|----------|
| padding | string | Used to pad messages | yes      |

#### StakeRewards
Claims and stakes available rewards
##### Request
| Name    | Type   | Description          | optional |
|---------|--------|----------------------|----------|
| padding | string | Used to pad messages | yes      |

#### ExposeBalance
Exposes the users current staked token balance
##### Request
| Name      | Type   | Description                                                | optional |
|-----------|--------|------------------------------------------------------------|----------|
| recipient | string | Where the token amount will be shown                       | no       |
| code_hash | string | Optional code hash of the token showing                    | yes      |
| msg       | string | Base64 encoded msg that will be forwarded to the recipient | yes      |
| memo      | string | Additional written context for the tx                      | yes      |
| padding   | string | Used to pad messages                                       | yes      |

#### ExposeBalanceWithCooldown
Exposes the users staked balance that is not in cooldown (useful for voting)
##### Request
| Name      | Type   | Description                                                | optional |
|-----------|--------|------------------------------------------------------------|----------|
| recipient | string | Where the token amount will be shown                       | no       |
| code_hash | string | Optional code hash of the token showing                    | yes      |
| msg       | string | Base64 encoded msg that will be forwarded to the recipient | yes      |
| memo      | string | Additional written context for the tx                      | yes      |
| padding   | string | Used to pad messages                                       | yes      |