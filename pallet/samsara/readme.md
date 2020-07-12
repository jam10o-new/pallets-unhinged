# samara

The goal of `pallet-samara` is to provide an alternative to `Currency`/token based economics, and the `Weight` system of FRAME, while providing a low-configuration/drop-in replacement for anywhere a substrate `Currency` is expected.

## Terminology

- Provide: Increase the supply of something over a period of time.
- Providable: A thing that can be `Provide`d.
- Desire: "Wish For" something to have it's supply increased (ie, your balance of some arbitrary token), and assign a relative value to it.
- Desirable: A thing that can be `Desire`d.
- Karma: A non-transferrable token that incentivises inactivity. 
- Long Count: The period of time in which `samara` recalculates what qualifies as valuable.
- Short Count: The period of time in which `samara` distributes value.


## Design 

The core of `samara` lies in the concept of maximizing "marginal perceived value" where all participants in an economy operate as much/as little as they wish to `provide` to the economy, and declare values for things they wish to be `provided`. Every Short Count, `samara` matches "production" to "desire" in a way that maximizes the total value all participants believe has been `provided` to them. Things can be `provided` immediately and then have their value tallied at the end of the Short Count (suboptimal) or things Providers mark as `providable` can be allocated via the distribution mechanism of `samara` at the end of the Short Count.

Game theoretically, `samara` if designed naively, incentivises activity over inactivity (whoever plays the "paperclip game" and "trades up" most often, ends up having the most  value), so the system itself is intentionally "samarad" in two ways to avoid that:
* If undeclared, all participants wish "Karma" is provided to them by default, which is a non-transferrable token - outside of being `provided` between participants - which cannot be partially provided, and "compounds" if received while already held. The purpose of "Karma" is to incentivise inaction (as opposed to other antispam mechanisms which impose a cost on action). 
* Anything that has been designated value by any pariticipant at any point over the current Long Count, but has not been "desired" by any participant within this Short Count is distributed to Karma holders based on their current holding of Karma.

Karma provides an alternative to participants hoarding "non-perishables" ("Store of Value"), while adaquately being non-monopolistic itself. The more "Karmic" a participant is, the more compounding value Karma has for them, so "Karma Whales" naturally turn into sinks to maintain the value of Karma itself.

In the scenario where a participant desires something(s), but meeting that desire would lead to a suboptimal total perceived value for the entire ecosystem, that desire's perceived value is allocated to Karma.

In the scenario where a participant desires something(s), but no other participant has provided it, it remains as a desire, with it's value scaling upwards in a way that does not effect the remainder of percieved value allocatable to other desires (ie, if every participant has 100 "allocatable perceived value", and a participant uses 50 of it on their value of some item, in the next short count, it would scale upwards to `50*factor`, but the participant would still have `100-50` allocatable value, not `100-50*factor`)

In combination with a decentralized data-provider, or trusted escrow, "desires" can be declared in the "real world", which may be used to request goods or services. 

## Implementation Ideas:

When a participant transfers `something(s)` `Desirable` to another participant, it is described as the "desire" to increase the amount of `something(s)` the latter participant controls, if they do not directly control that `something(s)` themselves. This may either be in the form of an atomic transfer ("Alice wants Bob to receive a 50 Cent Coin") with a value assigned to it, or a "per unit value" that is not all-or-nothing.

Any extrinsic that provides something `Providable<ProvisionManager, Type>` that matches an existing `desire` counts as something that is `provided` by the `origin` of that extrinisic. `Providable` things may either be directly `provided` or marked as `providing` and then automatically allocated or not to allow for optimization. 

## Informal Spec