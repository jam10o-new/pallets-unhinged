# ratio

Ratio aims to be a drop-in replacement wherever a `Currency` is used, with the goal of, when used in conjunction with mechanisms to earn value doing positive work, creating economies with built in long-term reputational systems and non-exploitative/voluntary value sinks.

Users have two balances and a ratio, where ratio is defined as the ratio between a Positive Balance and a Negative Balance.

Transfers can either: 
    * reduce the Positive Balance of the sender, down to a minimum of their total negative balance, and reduce the Negative Balance of the receiver. (improve total ratio, decrease liquidity)
    * increase the Negative Balance of the sender, up to a max of their positive value, and increase the Positive Balance of the receiver. (worsen total ratio, increase liquidity)

Users can also "reduce the stubbornness" of their ratio/rebalance their ratio by reducing both of their Balances by an equal factor to maintain the same ratio. (reduce liquidity, maintain personal ratio, total ratio moves towards current personal ratio)

Ratio can be used as a reputation system by observing the absolute value of the User's Positive Balance or by observing the relative value of Positive Balance and Negative Balance, this reputation can be used to inform systems like governance, be used to inform QoS in other pallets, or otherwise prioritize or benefit users.

Used as a currency, Positive Balance accrues value due to all of the mechanisms that reduce supply of Ratio, and the higher marginal value of Positive Balance for new entrants into the ecosystem than older.