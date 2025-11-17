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

next hurdle:
    
    it seems market_orders don't go into the btreeMap (do more research on how they should be dealt with)

    User balances and orderbook stored in a saparate thread in a variable (research more on this and why this way)
 
    
 -->


