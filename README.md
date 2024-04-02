# rust-trade

simple backend that 

1. records some signals from trading view and puts them in influxdb.
2. sets up an internal API for a react front end (badly).
3. serves html and txt to the front end and does some other stuff
4. has a few tests of the barter-rs system in the bin folder, these were corrupted in a moment of l-user fail and temporarily dead.

it uses two servers for external and internal, dont ask me why. the internal communication is not encrypted.

# .env 

you must have a .env file with your credentials in and the location of your SSL key:
```
INFLUXDB_HOST="http://localhost:8086"
INFLUXDB_ORG=""
INFLUXDB_TOKEN=""

SSL_PUB=""
SSL_PRV=""
```

# External Port:
It's on port 1025 for now and you can forward external SSL to it. 

if you are opening your external SSL port to signals it might be useful to filter out anything but these trading view IP addresses:
```
 52.89.214.238             
 34.212.75.30              
 54.218.53.128             
 52.32.178.7               
```
# Internal API
you can test the internal API like so:

endpoint test:

`curl http://localhost:8080/api/test`

signal test:

signals are stored in influxdb like this: 
```
pub struct SignalData {
    pub strat: String,               // Strategy name or identifier
    pub timescale: String,           // Timescale, e.g., in minutes 1,5,240 etc. 
    pub exchange: String,            // Exchange name, e.g., "binance", "kucoin"
    pub alert_type: String,          // Type of alert, e.g., "Buy", "Sell"
    pub pair: String,                // Trading pair, e.g., "BTC-USDT"
    pub amount: f64,                 // Amount as a floating-point number
    pub price: f64,                  // Price at signal time as a floating-point number
    pub current_alert_number: i32,   // Current alert number as an integer
}
```

signals are from trading view in this format:

`NAME {{interval}} EXHCANGE SIDE {{ticker}} AMOUNT {{close}} ALERTNUMBER`

so a typical entry in trading view might be:

`SMA {{interval}} binance sell {{ticker}} 1000 {{close}} 1`

you can test your endpoint without trading view:
`curl -k -X POST -d 'WOOHA 3 kucoin sell AVAXUSDT 1000 32.1 1' https://localhost:1025/`

to test getting a list of doc files from your public/documents/ folder you can do this:

`curl http://localhost:8080/api/get_docs`

to test retrieving a doc you can do this:

`curl -v http://localhost:8080/api/get_file/test.txt`

to test killing your entire database(WARNING!!!!) you can do this:

`curl http://localhost:8080/api/clear_database`

