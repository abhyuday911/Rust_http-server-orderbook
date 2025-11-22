## HTTP Endpoints

- signup
- signin
- onramp
- create_limit_order
- create_market_order
- get_orderbook

## Things to look into.

- storing data in-memory
- Data structure to store bids and asks - BTreeMaps
- tokios mpsc vs std mpsc (async vs sync)

## No clue about the terms
- oneshot channels 
- mutexes 
- arc


##


<!-- 

current: 
    mpsc done and data is transmitted to engine from routes 
    
    create order & order struct

    create orderBook state and send throughout the routes / send it to the engine?
    
    the engine mutates the value in the order-book
    


-->


<!-- 
    current tasks: 
        just settle the market order (jitna hi ho)
            - think about the replying back later
 -->

<!-- 

next:
    
    Middleware to look-up user credibility.  

    Make engine reply enum with respect to which the route will send Ok() / Conflict() / InternalServerError() according to the reply from the engine.

    MarketOrder {
        partially fullfilled (bought x quantity -> with y amount of money),
        rejected (no offers found to be traded)
        completely fullfilled (order fullfilled -> total money spent y)
    }

    What would happen if the engine fulfills half of em and shuts down mid-way.

    Update the user details {
        current limit orders: [order-ids],
        current hold of assets {
            btc : 5,
            doge: 6
        }
    }
    
 -->


