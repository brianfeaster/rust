//! # External Chat Service Robot
mod util;  pub use crate::util::*;
mod comm;  use crate::comm::*;
mod srvs;  use crate::srvs::*;
mod db;    use crate::db::*;
use ::std::{env,
    error::Error,
    mem::transmute,
    collections::{HashMap, HashSet},
    str::{from_utf8},
    fs::{read_to_string, write},
    sync::{Arc, Mutex},
 };
use ::std::boxed::Box;
use ::log::*;
use ::regex::{Regex};
use ::datetime::{Instant, Duration, LocalDate, LocalTime, LocalDateTime, DatePiece,
    Weekday::{Sunday, Saturday} };
use ::openssl::ssl::{SslConnector, SslAcceptor, SslFiletype, SslMethod};
use ::actix_web::{web, App, HttpRequest, HttpServer, HttpResponse, Route,
    client::{Client, Connector} };
use ::macros::*;

////////////////////////////////////////////////////////////////////////////////

const QUOTE_DELAY_SECS :i64 = 60;

//////////////////////////////////////////////////////////////////////////////
// Rust primitives enahancements

trait AsI64 { fn as_i64 (&self, i:usize) -> Bresult<i64>; }
impl AsI64 for Vec<Option<String>> {
    fn as_i64 (&self, i:usize) -> Bresult<i64> {
        Ok(self.get(i).ok_or("Can't index vector")?
           .as_ref().ok_or("Can't parse i64 from None")?
           .parse::<i64>()?)
    }
}

trait AsF64 { fn as_f64 (&self, i:usize) -> Bresult<f64>; }
impl AsF64 for Vec<Option<String>> {
    fn as_f64 (&self, i:usize) -> Bresult<f64> {
        Ok(self.get(i).ok_or("Can't index vector")?
           .as_ref().ok_or("Can't parse f64 from None")?
           .parse::<f64>()?)
    }
}

trait AsStr { fn as_istr (&self, i:usize) -> Bresult<&str>; }
impl AsStr for Vec<Option<String>> {
    fn as_istr (&self, i:usize) -> Bresult<&str> {
        Ok(self.get(i)
            .ok_or("can't index vector")?.as_ref()
            .ok_or("can't infer str from None")? )
    }
}

trait AsString { fn as_string (&self, i:usize) -> Bresult<String>; }
impl AsString for Vec<Option<String>> {
    fn as_string (&self, i:usize) -> Bresult<String> {
        Ok(self.get(i)
            .ok_or("can't index vector")?.as_ref()
            .ok_or("can't infer str from None")?
            .to_string() )
    }
}

trait GetI64 { fn get_i64 (&self, k:&str) -> Bresult<i64>; }
impl GetI64 for HashMap<String, String> {
    fn get_i64 (&self, k:&str) -> Bresult<i64> {
        Ok(self
            .get(k).ok_or(format!("Can't find key '{}'", k))?
            .parse::<i64>()?)
    }
}
trait GetF64 { fn get_f64 (&self, k:&str) -> Bresult<f64>; }
impl GetF64 for HashMap<String, String> {
    fn get_f64 (&self, k:&str) -> Bresult<f64> {
        Ok(self
            .get(k).ok_or(format!("Can't find key '{}'", k))?
            .parse::<f64>()?)
    }
}

trait GetStr { fn get_str (&self, k:&str) -> Bresult<String>; }
impl GetStr for HashMap<String, String> {
    fn get_str (&self, k:&str) -> Bresult<String> {
        Ok(self
            .get(k)
            .ok_or(format!("Can't find key '{}'", k))?
            .to_string() )
    }
}

trait GetChar { fn get_char (&self, k:&str) -> Bresult<char>; }
impl GetChar for HashMap<String, String> {
    fn get_char (&self, k:&str) -> Bresult<char> {
        Ok(self
            .get(k)
            .ok_or(format!("Can't find key '{}'", k))?
            .to_string()
            .chars()
            .nth(0)
            .ok_or(format!("string lacks character key '{}'", k))? )

    }
}

/*
// Trying to implement without using ? for the learns.
impl AsStr for Vec<Option<String>> {
    fn as_istr (&self, i:usize) -> Bresult<&str> {
        self.get(i) // Option< Option<String> >
        .ok_or("can't index vector".into()) // Result< Option<String>>, Bresult<> >
        .map_or_else(
            |e| e, // the previous error
            |o| o.ok_or("can't infer str from None".into())
                .map( |r| &r.to_string()) ) // Result< String, Bresult >
    }
}
*/

////////////////////////////////////////////////////////////////////////////////
/// Abstracted primitive types helpers

/// Decide if a ticker's price should be refreshed given its last lookup time.
///    Market   PreMarket   Regular  AfterHours       Closed
///       PST   0900Zpre..  1430Z..  2100Z..0100Zaft  0100Z..9000Z
///       PDT   0800Z..     1330Z..  2000Z..2400Z     0000Z..0800Z
///  Duration   5.5h        6.5h     4h               8h
fn most_recent_trading_hours (
    dst_hours_adjust: i8,
    now: LocalDateTime,
) -> Bresult<(LocalDateTime, LocalDateTime)> {
    // Absolute time (UTC) when US pre-markets open 1AMPT / 0900Z|0800Z
    let start_time = LocalTime::hm(9 - dst_hours_adjust, 0)?;  // TODO create function describing the hour maths
    let start_duration = Duration::of(start_time.to_seconds());
    let market_hours_duration = Duration::of( LocalTime::hm(16, 30)?.to_seconds() ); // Add 30 minutes for delayed orders

    // Consider the current trading date relative to now. Since pre-markets
    // start at 0900Z (0800Z during daylight savings) treat that as the
    // start of the trading day so subtract that many hours (shift to midnight)
    // to shift the yesterday/today cutoff.
    let now_norm_date :LocalDate = (now - start_duration).date();

    // How many days ago (in seconds) were the markets open?
    let since_last_market_duration =
        Duration::of( LocalTime::hm(24, 0)?.to_seconds() )
        * match now_norm_date.weekday() { Sunday=>2, Saturday=>1, _=>0 };

    // Absolute time the markets opened/closed last. Could be future time.
    let time_open = LocalDateTime::new(now_norm_date, start_time) - since_last_market_duration;
    let time_close = time_open + market_hours_duration; // Add extra 30m for delayed market orders.

    info!("most_recent_trading_hours  now:{:?}  now_norm_date:{:?}  since_last_market_duration:{:?}  time_open:{:?}  time_close:{:?}",
        now, now_norm_date, since_last_market_duration, time_open, time_close);

    Ok((time_open, time_close))
}

fn trading_hours_p (dst_hours_adjust:i8, now:i64) -> Bresult<bool>{
    let now = LocalDateTime::at(now);
    let (time_open, time_close) = most_recent_trading_hours(dst_hours_adjust, now)?;
    Ok(time_open <= now && now <= time_close)
}

fn update_ticker_p (
    envstruct: &EnvStruct,
    cached: i64,
    now:    i64,
    traded_all_day: bool
) -> Bresult<bool> {
    info!("update_ticker_p  cached:{}  now:{}  traded_all_day:{}", cached, now, traded_all_day);

    if traded_all_day { return Ok(cached+(envstruct.quote_delay_secs) < now) }

    let lookup_throttle_duration = Duration::of( 60 * envstruct.quote_delay_secs ); // Lookup tickers every 2 minutes

    // Do everything in DateTime
    let cached = LocalDateTime::at(cached);
    let now = LocalDateTime::at(now);

    let (time_open, time_close) = most_recent_trading_hours(envstruct.dst_hours_adjust, now)?;

    info!("update_ticker_p  cached:{:?}  now:{:?}  time_open:{:?}  time_close:{:?}",
        cached, now, time_open, time_close);

    Ok(cached <= time_close
        && time_open <= now
        && (cached + lookup_throttle_duration < now  ||  time_close <= now))
}

/// Round a float at the specified decimal offset
///  println!("{:?}", num::Float::integer_decode(n) );
fn roundfloat (num:f64, dec:i32) -> f64 {
    let fac = 10f64.powi(dec);
    let num_incremented = unsafe { transmute::<u64, f64>(transmute::<f64, u64>(num) + 1) };
    (num_incremented * fac).round() / fac
}

// Round for numbers that were serialized funny: -0.00999999 => -0.01, -1.3400116 => -1.34
fn roundqty (num:f64) -> f64 { roundfloat(num, 4) }
fn roundcents (num:f64) -> f64 { roundfloat(num, 2) }

fn round (f:f64) -> String {
    if 0.0 == f {
        format!(".00")
    } else if f < 1.0 {
        format!("{:.4}", roundqty(f))
            .trim_start_matches('0').to_string()
    } else {
        format!("{:.2}", roundcents(f))
    }
}
// number to -> .1234 1.12 999.99 1.99k
fn roundkilofy (f:f64) -> String {
    if 0.0 == f {
        format!(".00")
    } else if 1000.0 <= f {
        format!("{:.2}k", roundcents(f/1000.0))
    } else if f < 1.0 {
        format!("{:.4}", roundqty(f))
            .trim_start_matches('0').to_string()
    } else {
        format!("{:.2}", roundcents(f))
            .trim_start_matches('0').to_string()
    }
}

// Trim trailing . and 0
fn num_simp (num:&str) -> String {
    num
    .trim_start_matches("0")
    .trim_end_matches("0")
    .trim_end_matches(".")
    .into()
}

// Try to squish number to 3 digits: "100%"  "99.9%"  "10.0%"  "9.99%"  "0.99%""
fn percent_squish (num:f64) -> String {
    if 100.0 <= num{ return format!("{:.0}", num) }
    if 10.0 <= num { return format!("{:.1}", num) }
    if 1.0 <= num  { return format!("{:.2}", num) }
    if num < 0.001 { return format!("{:.1}", num) }
    format!("{:.3}", num).trim_start_matches('0').into() // .0001 ... .9999
}

// Stringify float to two decimal places unless under 10
// where it's 4 decimal places with trailing 0s truncate.
fn money_pretty (n:f64) -> String {
    if n < 10.0 {
        let mut np = format!("{:.4}", n);
        np = regex_to_hashmap(r"^(.*)0$", &np).map_or(np, |c| c["1"].to_string() );
        regex_to_hashmap(r"^(.*)0$", &np).map_or(np, |c| c["1"].to_string() )
    } else {
        format!("{:.2}", n)
    }
}

fn percentify (a: f64, b: f64) -> f64 {
    (b-a)/a*100.0
}

// (5,"a","bb") => "a..bb"
fn _pad_between(width: usize, a:&str, b:&str) -> String {
    let lenb = b.len();
    format!("{: <pad$}{}", a, b, pad=width-lenb)
}

/// Return hashmap of the regex capture groups, if any.
fn regex_to_hashmap (re: &str, msg: &str) -> Option<HashMap<String, String>> {
    let regex = Regex::new(re).unwrap();
    regex.captures(msg).map(|captures| { // Consider capture groups or return None
        regex
            .capture_names()
            .enumerate()
            .filter_map(|(i, capname)| { // Over capture group names (or indices)
                capname.map_or(
                    captures
                        .get(i) // Get match via index.  This could be null which is filter by filter_map
                        .map(|capmatch| (i.to_string(), capmatch.as_str().into())),
                    |capname| {
                        captures
                            .name(capname) // Get match via capture name.  Could be null which is filter by filter_map
                            .map(|capmatch| (capname.into(), capmatch.as_str().into()))
                    },
                )
            })
            .collect()
    })
}

fn regex_to_vec (re: &str, msg: &str) -> Bresult<Vec<Option<String>>> {
    Regex::new(re)?
    .captures(msg) // An Option<Captures>
    .map_or(Ok(Vec::new()), // Return Ok empty vec if None...
        |captures|          // ...or return Ok Vec of Option<Vec>
        Ok(captures.iter() // Iterator over Option<Match>
            .map( |o_match| // None or Some<String>
                    o_match.map( |mtch| mtch.as_str().into() ) )
            .collect()))
}

// Transforms @user_nickname to {USER_ID}
fn deref_ticker (dbconn:&Connection, s:&str) -> Option<String> {
    // Expects:  @shrewm   Returns: Some(308188500)
    if &s[0..1] == "@" {
        // Quote is ID of whomever this nick matches
        getsql!(dbconn, "SELECT id FROM entitys WHERE name=?", &s[1..])
            .ok()
            .filter( |v| 1 == v.len() )
            .map( |v| v[0].get("id").unwrap().to_string() )
    } else { None }
}

// Transforms {USER_ID} to @user_nickname
fn reference_ticker (dbconn:&Connection, t :&str) -> String {
    if is_self_stonk(t) {
        getsql!(dbconn, "SELECT name FROM entitys WHERE id=?", t) // Result<Vec, Err>
        .map_err( |e| error!("{:?}", e) )
        .ok()
        .filter( |v| 1 == v.len() )
        .map( |v| format!("@{}", v[0].get("name").unwrap()) )
        .unwrap_or(t.to_string())
    } else {
        t.to_string()
    }
}

fn is_self_stonk (s: &str) -> bool {
    Regex::new(r"^[0-9]+$").unwrap().find(s).is_some()
}

////////////////////////////////////////////////////////////////////////////////
/// Helpers on complex types

fn get_bank_balance (cmdstruct:&CmdStruct) -> Bresult<f64> {
    let dbconn = &cmdstruct.dbconn;
    let id = cmdstruct.id;
    let res = getsql!(dbconn, "SELECT * FROM accounts WHERE id=?", id)?;
    Ok(
        if res.is_empty() {
            let sql = getsql!(dbconn, "INSERT INTO accounts VALUES (?, ?)", id, 1000.0)?;
            info!("{:?}", sql);
            1000.0
        } else {
            res[0]
            .get("balance".into())
            .ok_or("balance missing from accounts table")?
            .parse::<f64>()
            .or( Err("can't parse balance field from accounts table") )?
        }
    )
}

pub fn sql_table_order_insert (dbconn:&Connection, id:i64, ticker:&str, qty:f64, price:f64, time:i64) -> Bresult<Vec<HashMap<String, String>>> {
    getsql!(dbconn, "INSERT INTO orders VALUES (?, ?, ?, ?, ?)", id, ticker, qty, price, time)
}

////////////////////////////////////////////////////////////////////////////////
/// Blobs

#[derive(Debug)]
pub struct EnvStruct {
    url_api:          String, // Telgram API URL
    sqlite_filename:  String, // Local SQLite filename
    chat_id_default:  i64,    // Fallback/default Telegram channel id
    quote_delay_secs: i64,    // Delay between remote stock quote queries
    dst_hours_adjust: i8,     // 1 = DST, 0 = ST
    // Muteables Previous message editing
    message_id_read:    i64,   // ?? Message id of the incoming messsage?
    message_id_write:   i64,   // ?? Message id of the outgoing message?
    message_buff_write: String,
    fmt_quote:          String, // Default format string for quotes
    fmt_position:       String, // Default format string for portfolio positions
}

type Env = Arc<Mutex<EnvStruct>>;

////////////////////////////////////////
#[derive(Debug)]
pub struct CmdStruct { // Represents in incoming Telegram message
    env: Env,
    dbconn: Connection,
    id: i64, // Entity that sent the message message.from.id
    at: i64, // Group (or entity/DM) that the message was sent in message.chat.id
    to: i64, // To whom the message is addressed (contains a message.reply_to_message.from.id) defaults to id
    id_level: i64, // Echo levels of each sendable.
    at_level: i64,
    to_level: i64,
    message_id: i64,
    fmt_quote: String,
    fmt_position: String,
    msg: String
}

type Cmd = Arc<Mutex<CmdStruct>>;

//Old way when structs were being passed by value/copy.  (Copy back important altered state for next time)
//env.message_id_read = cmd.env.message_id_read;
//env.message_id_write = cmd.env.message_id_write;
//env.message_buff_write.clear();
//env.message_buff_write.push_str(&cmd.env.message_buff_write);

impl CmdStruct {
    fn _level (&self, level:i64) -> MsgCmd { MsgCmd::from(self).level(level) }
    fn _to (&self, to:i64) -> MsgCmd { MsgCmd::from(self)._to(to) }
    // Creates a Cmd object from Env and Telegram message body.
    // It is passed to a bunch of async functions so must exist
    // for as long as there are async waiting.
    fn parse_cmd(env:Env, body: &web::Bytes) -> Bresult<Cmd> {
        let envstruct = env.lock().unwrap();
        let dbconn = Connection::new(&envstruct.sqlite_filename)?; // TODO move this to env
        let json: Value = bytes2json(&body)?;
        let inline_query = &json["inline_query"];

        let edited_message = &json["edited_message"];
        let message = if edited_message.is_object() { edited_message } else { &json["message"] };
        let message_id = getin_i64(message, &["message_id"]).unwrap_or(0);

        let (id, at, to, msg) =
            if inline_query.is_object() {
                let id = getin_i64(inline_query, &["from", "id"])?;
                let msg = getin_str(inline_query, &["query"])?.to_string();
                (id, id, id, msg) // Inline queries are strictly DMs (no associated "at" groups nor reply "to" messages)
            } else if message.is_object() {
                let id = getin_i64(message, &["from", "id"])?;
                let at = getin_i64(message, &["chat", "id"])?;
                let msg = getin_str(message, &["text"])?.to_string();
                if let Ok(to) = getin_i64(&message, &["reply_to_message", "from", "id"]) {
                    (id, at, to, msg) // Reply to message
                } else {
                    (id, at, id, msg) // TODO: should "to" be the "at" group and not the "id" sender?  Check existing logic first.
                }
            } else { Err("Nothing to do.")? };

        // Create hash map id -> echoLevel
        let echo_levels =
            getsql!(dbconn, "SELECT id, echo FROM entitys NATURAL JOIN modes")?.iter()
            .map( |hm| // tuple -> hashmap
                 ( hm.get_i64("id").unwrap(),
                   hm.get_i64("echo").unwrap() ) )
            .collect::<HashMap<i64,i64>>();

        let rows = getsql!(dbconn, r#"SELECT entitys.id, COALESCE(quote, "") AS quote, coalesce(position, "") AS position FROM entitys LEFT JOIN formats ON entitys.id = formats.id WHERE entitys.id=?"#,
            id)?;

        let (mut fmt_quote, mut fmt_position) =
            if rows.is_empty() {
                Err( format!("{} missing from entitys", id))?
            } else {
                (rows[0].get_str("quote")?, rows[0].get_str("position")? )
            };
        if fmt_quote.is_empty() { fmt_quote = envstruct.fmt_quote.to_string() }
        if fmt_position.is_empty() { fmt_position = envstruct.fmt_position.to_string() }
        info!("fmt_quote {:?}  fmt_position {:?}", fmt_quote, fmt_position);

        Ok(Cmd::new(Mutex::new(CmdStruct{
            env: env.clone(),
            dbconn, id, at, to,
            id_level: echo_levels.get(&id).map(|v|*v).unwrap_or(2_i64),
            at_level: echo_levels.get(&at).map(|v|*v).unwrap_or(2_i64),
            to_level: echo_levels.get(&to).map(|v|*v).unwrap_or(2_i64),
            message_id, fmt_quote, fmt_position, msg
        })))
    }
} // Cmd

////////////////////////////////////////

#[derive(Debug)]
pub struct Quote {
    pub cmd: Cmd,
    pub ticker: String,
    pub price: f64, // Current known price
    pub last: f64,  // Previous day's closing regular market price
    pub amount: f64, // Delta change since last
    pub percent: f64, // Delta % change since last
    pub market: String, // 'p're 'r'egular 'p'ost
    pub hours: i64, // 16 or 24 (hours per day market trades)
    pub exchange: String,// Keep track of this to filter "PNK" exchanged securities.
    pub title: String, // Full title/name of security
    pub updated: bool // Was this generated or pulled from cache
}

impl Quote { // Query internet for ticker details
    async fn new_market_quote (cmd:Cmd, ticker: &str) -> Bresult<Self> {
        let json = srvs::get_ticker_raw(ticker).await?;

        let details = getin(&json, &["context", "dispatcher", "stores", "QuoteSummaryStore", "price"]);
        if details.is_null() { Err("Unable to find quote data in json key 'QuoteSummaryStore'")? }
        info!("{}", details);

        let title_raw =
            &getin_str(&details, &["longName"])
            .or_else( |_e| getin_str(&details, &["shortName"]) )?;
        let mut title :&str = title_raw;
        let title = loop { // Repeatedly strip title of useless things
            let title_new = title
                .trim_end_matches(".")
                .trim_end_matches(" ")
                .trim_end_matches(",")
                .trim_end_matches("Inc")
                .trim_end_matches("Corp")
                .trim_end_matches("Corporation")
                .trim_end_matches("Holdings")
                .trim_end_matches("Brands")
                .trim_end_matches("Company")
                .trim_end_matches("USD");
            if title == title_new { break title_new.to_string() }
            title = title_new;
        };
        info!("title pretty {} => {}", title_raw, &title);

        let exchange = getin_str(&details, &["exchange"])?;

        let hours = getin(&details, &["volume24Hr"]);
        let hours :i64 = if hours.is_object() && 0 != hours.as_object().unwrap().keys().len() { 24 } else { 16 };

        let previous_close   = getin_f64(&details, &["regularMarketPreviousClose", "raw"]).unwrap_or(0.0);
        let pre_market_price = getin_f64(&details, &["preMarketPrice", "raw"]).unwrap_or(0.0);
        let reg_market_price = getin_f64(&details, &["regularMarketPrice", "raw"]).unwrap_or(0.0);
        let pst_market_price = getin_f64(&details, &["postMarketPrice", "raw"]).unwrap_or(0.0);

        // This array will be sorted on market time for the latest market data.  Prices
        // are relative to the previous day's market close, including pre and post markets,
        // because most online charts I've come across ignore previous day's after market
        // closing price for next day deltas.  Non-regular markets are basically forgotten
        // after the fact.
        let mut details = [
            (pre_market_price, reg_market_price, "p", getin_i64(&details, &["preMarketTime"]).unwrap_or(0)),
            (reg_market_price, previous_close,   "r", getin_i64(&details, &["regularMarketTime"]).unwrap_or(0)),
            (pst_market_price, previous_close,   "a", getin_i64(&details, &["postMarketTime"]).unwrap_or(0))];

        /* // Log all prices for sysadmin requires "use ::datetime::ISO"
        use ::datetime::ISO;
        error!("{} \"{}\" ({}) {}hrs\n{:.2} {:.2} {} {:.2}%\n{:.2} {:.2} {} {:.2}%\n{:.2} {:.2} {} {:.2}%",
            ticker, title, exchange, hours,
            LocalDateTime::from_instant(Instant::at(details[0].3)).iso(), details[0].0, details[0].1, details[0].2,
            LocalDateTime::from_instant(Instant::at(details[1].3)).iso(), details[1].0, details[1].1, details[1].2,
            LocalDateTime::from_instant(Instant::at(details[2].3)).iso(), details[2].0, details[2].1, details[2].2); */

        details.sort_by( |a,b| b.3.cmp(&a.3) ); // Find latest quote details

        let price = details[0].0;
        let last = details[0].1;
        Ok(Quote{
            cmd,
            ticker:  ticker.to_string(),
            price, last,
            amount:  roundqty(price-last),
            percent: percentify(last,price),
            market:  details[0].2.to_string(),
            hours, exchange, title,
            updated: true})
    } // Quote::new_market_quote 
}

impl Quote {// Query local cache or internet for ticker details
    async fn get_market_quote (cmd:&Cmd, ticker:&str) -> Bresult<Self> {
        // Make sure not given referenced stonk. Expected symbols:  GME  308188500   Illegal: @shrewm
        if &ticker[0..1] == "@" { Err("Illegal ticker")? }
        let is_self_stonk = is_self_stonk(ticker);
        let now :i64 = Instant::now().seconds();

        let (res, is_in_table, is_cache_valid) = {
            let cmdstruct = cmd.lock().unwrap();
            let res = getsql!(
                cmdstruct.dbconn,
                "SELECT ticker, price, last, market, hours, exchange, time, title FROM stonks WHERE ticker=?",
                ticker)?;
            let is_in_table = !res.is_empty();
            let is_cache_valid = 
                is_in_table && (is_self_stonk || { 
                    let hm = &res[0];
                    let timesecs       = hm.get_i64("time")?;
                    let traded_all_day = 24 == hm.get_i64("hours")?;
                    !update_ticker_p(&cmdstruct.env.lock().unwrap(), timesecs, now, traded_all_day)?
                });
            (res, is_in_table, is_cache_valid)
        };

        let quote =
            if is_cache_valid { // Is in cache so use it
                let hm = &res[0];
                let price = hm.get_f64("price")?;
                let last = hm.get_f64("last")?;
                Quote{
                    cmd: cmd.clone(),
                    ticker: hm.get_str("ticker")?,
                    price, last,
                    amount:   roundqty(price-last),
                    percent:  percentify(last,price),
                    market:   hm.get_str("market")?,
                    hours:    hm.get_i64("hours")?,
                    exchange: hm.get_str("exchange")?,
                    title:    hm.get_str("title")?,
                    updated:  false}
            } else if is_self_stonk { // FNFT is not in cache so create
                Quote {
                    cmd: cmd.clone(),
                    ticker: ticker.to_string(),
                    price:   0.0,
                    last:    0.0,
                    amount:  0.0,
                    percent: 0.0,
                    market:  "r".to_string(),
                    hours:   24,
                    exchange:"™BOT".to_string(),
                    title:   "FNFT".to_string(),
                    updated: true
                }
            } else { // Quote not in cache so query internet
                Quote::new_market_quote(cmd.clone(), ticker).await?
            };

        if !is_self_stonk { // Cached FNFTs are only updated during trading/settling.
            let dbconn = &cmd.lock().unwrap().dbconn;
            if is_in_table {
                getsql!(dbconn, "UPDATE stonks SET price=?, last=?, market=?, time=? WHERE ticker=?",
                    quote.price, quote.last, &*quote.market, now, quote.ticker.as_str())?
            } else {
                getsql!(dbconn, "INSERT INTO stonks VALUES(?,?,?,?,?,?,?,?)",
                    &*quote.ticker, quote.price, quote.last, &*quote.market, quote.hours, &*quote.exchange, now, &*quote.title)?
            };
        }

        Ok(quote)
    } // Quote::get_market_quote
}

fn fmt_decode_to (c: &str, s: &mut String) {
    match c {
        "%" => s.push_str("%"),
        "b" => s.push_str("*"),
        "i" => s.push_str("_"),
        "u" => s.push_str("__"),
        "s" => s.push_str("~"),
        "q" => s.push_str("`"),
        "n" => s.push_str("\n"),
        c => { s.push_str("%"); s.push_str(c) }
    }
}

fn amt_as_glyph (amt: f64) -> (&'static str, &'static str) {
    if 0.0 < amt {
        (from_utf8(b"\xF0\x9F\x9F\xA2").unwrap(), from_utf8(b"\xE2\x86\x91").unwrap()) // Green circle, Arrow up
    } else if 0.0 == amt {
        (from_utf8(b"\xF0\x9F\x94\xB7").unwrap(), " ") // Blue diamond, nothing
    } else {
        (from_utf8(b"\xF0\x9F\x9F\xA5").unwrap(), from_utf8(b"\xE2\x86\x93").unwrap()) // Red square, Arrow down
    }
}

impl Quote { // Format the quote/ticker using its format string IE: 🟢ETH-USD@2087.83! ↑48.49 2.38% Ethereum USD CCC
    fn format_quote (&self) -> Bresult<String> {
        let cmdstruct = self.cmd.lock().unwrap();

        let dbconn = &cmdstruct.dbconn;
        let fmt = cmdstruct.fmt_quote.to_string();
        let gain_glyphs = amt_as_glyph(self.amount);

        Ok(Regex::new("(?s)(%([A-Za-z%])|.)").unwrap()
            .captures_iter(&fmt)
            .fold(String::new(), |mut s, cap| {
                if let Some(m) = cap.get(2) {
                    match m.as_str() {
                    "A" => s.push_str(gain_glyphs.1), // Arrow
                    "B" => s.push_str(&format!("{}", money_pretty(self.amount.abs()))), // inter-day delta
                    "C" => s.push_str(&percent_squish(self.percent.abs())), // inter-day percent
                    "D" => s.push_str(gain_glyphs.0), // red/green light
                    "E" => s.push_str(&reference_ticker(&dbconn, &self.ticker).replacen("_", "\\_", 10000)),  // ticker symbol
                    "F" => s.push_str(&format!("{}", money_pretty(self.price))), // current stonk value
                    "G" => s.push_str(&self.title),  // ticker company title
                    "H" => s.push_str( // Market regular, pre, after
                        if self.hours == 24 { "∞" }
                        else { match &*self.market { "r"=>"", "p"=>"ρ", "a"=>"α", _=>"?" } },
                    ),
                    "I" => s.push_str( if self.updated { "·" } else { "" } ),
                    c => fmt_decode_to(c, &mut s) }
                } else {
                    s.push_str(&cap[1])
                }
                s
            } )
        )
    } // Quote.format_quote
}

////////////////////////////////////////

#[derive(Debug)]
struct Position {
    quote: Option<Quote>,
    id:    i64,
    ticker:String,
    qty:   f64,
    price: f64,
}

impl Position {
    async fn update_quote(&mut self, cmd: &Cmd) -> Bresult<&Position> {
        self.quote = Some(Quote::get_market_quote(cmd, &self.ticker).await?);
        Ok(self)
    }
}

impl Position {
    // Return vector instead of option in case of box/short positions.
    fn get_position (cmd:&Cmd, id: i64, ticker: &str) -> Bresult<Vec<Position>> {
        let dbconn = &cmd.lock().unwrap().dbconn;
        Ok(getsql!(dbconn, "SELECT qty,price FROM positions WHERE id=? AND ticker=?", id, ticker )?
        .iter()
        .map( |pos|
            Position {
                quote:  None,
                id,
                ticker: ticker.to_string(),
                qty:    pos.get_f64("qty").unwrap(),
                price:  pos.get_f64("price").unwrap()
            } )
        .collect())
    }
}

impl Position {
    fn get_users_positions (cmdstruct:&CmdStruct) -> Bresult<Vec<Position>> {
        let id = cmdstruct.id;
        Ok(getsql!(cmdstruct.dbconn, "SELECT ticker,qty,price FROM positions WHERE id=?", id)?
        .iter()
        .map( |pos|
            Position {
                quote: None,
                id,
                ticker: pos.get_str("ticker").unwrap(),
                qty: pos.get_f64("qty").unwrap(),
                price: pos.get_f64("price").unwrap()
            } )
        .collect())
    }
}

impl Position {
    async fn query(cmd:&Cmd, id:i64, ticker:&str) -> Bresult<Position> {
        let mut hm = Position::get_position(cmd, id, ticker)?;

        if 2 <= hm.len() {
            Err(format!("For {} ticker {} has {} positions, expect 0 or 1", id, ticker, hm.len()))?
        }

        let mut hm = // Consider the quote or a 0 quantity quote
            if 0 == hm.len() {
                Position {
                    quote: None,
                    id,
                    ticker: ticker.to_string(),
                    qty: 0.0,
                    price: 0.0 }
            } else {
                hm.pop().unwrap()
            };
        hm.update_quote(cmd).await?;
        Ok(hm)
    }
}

impl Position { // Format the position using its format string.
    fn format_position (&mut self, cmdstruct:&CmdStruct) -> Bresult<String> {
        let dbconn = &cmdstruct.dbconn;
        let fmt = &cmdstruct.fmt_position;

        let qty = self.qty;
        let cost = self.price;
        let price = self.quote.as_ref().unwrap().price;
        let last = self.quote.as_ref().unwrap().last;

        let basis = qty*cost;
        let value = qty*price;
        let last_value = qty*last;

        let gain = value - basis;
        let day_gain = value - last_value;

        let gain_percent = percentify(cost, price).abs();
        let day_gain_percent = percentify(last, price).abs();

        let gain_glyphs = amt_as_glyph(price-cost);
        let day_gain_glyphs = amt_as_glyph(price-last);

        Ok(Regex::new("(?s)(%([A-Za-z%])|.)").unwrap()
        .captures_iter(fmt)
        .fold( String::new(), |mut s, cap| {
            if let Some(m) = cap.get(2) { match m.as_str() {
                "A" => s.push_str( &format!("{:.2}", roundcents(value)) ), // value
                "B" => s.push_str( &round(gain.abs())), // gain
                "C" => s.push_str( gain_glyphs.1), // Arrow
                "D" => s.push_str( &percent_squish(gain_percent)), // gain%
                "E" => s.push_str( gain_glyphs.0 ), // Color
                "F" => s.push_str( &reference_ticker(dbconn, &self.ticker).replacen("_", "\\_", 10000) ), // Ticker
                "G" => s.push_str( &money_pretty(price) ), // latet stonks value
                "H" => s.push_str( &if 0.0 == qty { "0".to_string() } else { qty.to_string().trim_start_matches('0').to_string()} ), // qty
                "I" => s.push_str( &roundkilofy(cost) ), // cost
                "J" => s.push_str( day_gain_glyphs.0 ), // day color
                "K" => s.push_str( &money_pretty(day_gain) ), // inter-day delta
                "L" => s.push_str( day_gain_glyphs.1 ), // day arrow
                "M" => s.push_str( &percent_squish(day_gain_percent) ), // inter-day percent
                c => fmt_decode_to(c, &mut s) }
            } else {
                s.push_str(&cap[1]);
            }
            s
        } ) )
    }
}

////////////////////////////////////////

#[derive(Debug)]
struct Trade {
    cmd: Cmd,
    ticker: String,
    action: char, // '+' buy or '-' sell
    is_dollars: bool, // amt represents dollars or fractional quantity
    amt: Option<f64>
}

impl Trade {
    fn new (cmd:&Cmd) -> Option<Self> {
        //                         _____ticker____  _+-_  _$_   ____________amt_______________
        match regex_to_hashmap(r"^([A-Za-z0-9^.-]+)([+-])([$])?([0-9]+\.?|([0-9]*\.[0-9]{1,4}))?$", &cmd.lock().unwrap().msg) {
            Some(caps) => 
                Some(Self {
                    cmd: cmd.clone(),
                    ticker:     caps.get("1").unwrap().to_uppercase(),
                    action:     caps.get("2").unwrap().chars().nth(0).unwrap(),
                    is_dollars: caps.get("3").is_some(),
                    amt:        caps.get("4").map(|m| m.parse::<f64>().unwrap()) }),
                None => return None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Bot's Do Handler Helpers
////////////////////////////////////////////////////////////////////////////////

// For do_quotes Return set of tickers that need attention
pub fn extract_tickers (txt :&str) -> HashSet<String> {
    let mut tickers = HashSet::new();
    let re = Regex::new(r"^[@^]?[A-Z_a-z][-.0-9=A-Z_a-z]*$").unwrap(); // BRK.A ^GSPC BTC-USD don't end in - so a bad-$ trade doesn't trigger this
    for s in txt.split(" ") {
        let w = s.split("$").collect::<Vec<&str>>();
        if 2 == w.len() {
            let mut idx = 42;
            if w[0]!=""  &&  w[1]=="" { idx = 0; } // "$" is to the right of ticker symbol
            if w[0]==""  &&  w[1]!="" { idx = 1; } // "$" is to the left of ticker symbol
            if 42!=idx && re.find(w[idx]).is_some() { // ticker characters only
                let ticker = if &w[idx][0..1] == "@" {
                    w[idx].to_string()
                } else {
                    w[idx].to_string().to_uppercase()
                };
                tickers.insert(ticker );
            }
        }
    }
    tickers
} // extract_tickers

// for do_quotes
// Valid "ticker" strings:  GME, 308188500, @shrewm, local level 2 quotes.  Used by do_quotes only
async fn get_quote_pretty (cmd:&Cmd, ticker :&str) -> Bresult<String> {
    let (ticker, bidask) = { // Sub block so the Cmd lock is released/dropped before .await
        let dbconn = &cmd.lock().unwrap().dbconn;
        let ticker = deref_ticker(dbconn, ticker).unwrap_or(ticker.to_string());
        let bidask =
            if is_self_stonk(&ticker) {
                let mut asks = format!("*Asks:*");
                for ask in getsql!(dbconn, "SELECT -qty AS qty, price FROM exchange WHERE qty<0 AND ticker=? order by price;", &*ticker)? {
                    asks.push_str(
                        &format!(" `{}@{}`",
                        num_simp(ask.get("qty").unwrap()),
                        ask.get_f64("price")?));
                }
                let mut bids = format!("*Bids:*");
                for bid in getsql!(dbconn, "SELECT qty, price FROM exchange WHERE 0<qty AND ticker=? order by price desc;", &*ticker)? {
                    bids.push_str(
                        &format!(" `{}@{}`",
                        num_simp(bid.get("qty").unwrap()),
                        bid.get_f64("price")?));
                }
                format!("\n{}\n{}", asks, bids)
            } else {
                "".to_string()
            };
        (ticker, bidask)
    };

    let quote = Quote::get_market_quote(cmd, &ticker).await?;

    Ok(format!("{}{}",
        quote.format_quote()?,
        bidask))
} // get_quote_pretty


////////////////////////////////////////////////////////////////////////////////
// Bot's Do Handlers -- The Meat And Potatos.  The Bread N Butter.  The Works.
////////////////////////////////////////////////////////////////////////////////

async fn do_echo (cmd:&Cmd) -> Bresult<&'static str> {

    let (caps, at) = {
        let cmdstruct = cmd.lock().unwrap();
        let caps :Vec<Option<String>> = regex_to_vec("^/echo ?([0-9]+)?", &cmdstruct.msg)?;
        if caps.is_empty() { return Ok("SKIP") }
        (caps, cmdstruct.at)
    };

    match caps.as_i64(1) {
        Ok(echo) => { // Update existing echo level
            if 2 < echo {
                let msg = "*echo level must be 0…2*";
                send_msg_markdown_id(cmd.clone().into(), &msg).await?;
                Err(msg)?
            }
            send_msg_markdown( MsgCmd::from(cmd.clone()).level(0), &format!("`echo {} set verbosity {:.0}%`",
                echo, echo as f64/0.02)).await?;
            let cmdstruct = cmd.lock().unwrap();
            getsql!( &cmdstruct.dbconn, "UPDATE modes set echo=? WHERE id=?", echo, at)?;
        }
        Err(_) => { // Create or return existing echo level
            let mut echo = 2;
            {
                let cmdstruct = cmd.lock().unwrap();
                let rows = getsql!(cmdstruct.dbconn, "SELECT echo FROM modes WHERE id=?", at)?;
                if rows.len() == 0 { // Create/set echo level for this channel
                    getsql!(cmdstruct.dbconn, "INSERT INTO modes values(?, ?)", at, echo)?;
                } else {
                    echo = rows[0].get_i64("echo")?;
                }
            }
            send_msg_markdown(
                MsgCmd::from(cmd.clone()).level(0),
                &format!("`echo {} verbosity at {:.0}%`", echo, echo as f64/0.02)).await?;
        }
    }

    Ok("COMPLETED.")
}

pub async fn do_help (cmd:&Cmd) -> Bresult<&'static str> {
    let delay = {
        let cmdstruct = cmd.lock().unwrap();
        if Regex::new(r"/help").unwrap().find(&cmdstruct.msg).is_none() { return Ok("SKIP") }
        let envstruct = cmdstruct.env.lock().unwrap();
        envstruct.quote_delay_secs
    };
    send_msg_markdown(cmd.clone().into(), &format!(
"`          ™Bot Commands          `
`/echo 2` `Echo level (verbose 2…0 quiet)`
`word: ` `Definition lookup`
`+1    ` `Like someone's post (via reply)`
`+?    ` `Like leaderboard`
`/stonks` `Your Stonkfolio`
`/orders` `Your @shares and bid/ask orders`
`/yolo  ` `Stonks leaderboard`
`gme$   ` `Quote ({}min delay)`
`gme+   ` `Buy max GME shares`
`gme-   ` `Sell all GME shares`
`gme+3  ` `Buy 3 shares (min qty 0.0001)`
`gme-5  ` `Sell 5 share`
`gme+$18` `Buy $18 worth (min $0.01)`
`gme-$.9` `Sell 90¢ worth`
`@usr+2@3` `Bid/buy 2sh of '@usr' at $3`
`@usr-5@4` `Ask/sell 5sh of '@usr' at $4`
`/rebalance -9.99 AMC 40 QQQ 60 ...`
   `rebalances AMC/40% QQQ/60%, offset -$9.99`
`/fmt [?]     ` `Show format strings, help`
`/fmt [qp] ...` `Set quote/position fmt str`", delay)).await?;
    Ok("COMPLETED.")
}

async fn do_curse (cmd:&Cmd) -> Bresult<&'static str> {
    {
        let cmdstruct = cmd.lock().unwrap();
        if Regex::new(r"/curse").unwrap().find(&cmdstruct.msg).is_none() { return Ok("SKIP") }
    }
    send_msg_markdown(
        cmd.clone().into(),
        ["shit", "piss", "fuck", "cunt", "cocksucker", "motherfucker", "tits"][::rand::random::<usize>()%7]
    ).await?;
    Ok("COMPLETED.")
}

// Trying to delay an action
async fn do_repeat (cmd :&Cmd) -> Bresult<&'static str> {

    let (delay, msg) = {
        let cmdstruct = cmd.lock().unwrap();
        let caps =
            match Regex::new(r"^/repeat ([0-9]+) (.*)$").unwrap().captures(&cmdstruct.msg) {
                Some(caps) => caps,
                None => return Ok("SKIP".into())
            };
        let delay = caps[1].parse::<u64>()?;
        let msg = caps[2].to_string();
        (delay, msg)
    };

    //let msgcmd = MsgCmd::from(&*cmdstruct);
    //std::thread::spawn( move || {
        //let rt = ::tokio::runtime::Runtime::new().expect("Cannot create tokio runtime");
        //std::thread::sleep(std::time::Duration::from_secs(delay));
        //error!("{:?}", send_msg(cmd.into(), &format!("{} {}", delay, msg)).await);

    //});
    //task::sleep(std::time::Duration::from_secs(delay)).await;
    //println!("send_msg => {:?}", send_msg(cmd.into(), &format!("{}", msg)).await);

    ::tokio::time::delay_until(tokio::time::Instant::now() + std::time::Duration::from_secs(delay)).await;
    let msgcmd :MsgCmd = cmd.clone().into();
    let res = send_msg(msgcmd, &format!("{} {}", delay, msg)).await;
    println!("{:?}", res);
    //::tokio::spawn(async move {
    //});
    Ok("COMPLETED.")
}

async fn do_like (cmd:&Cmd) -> Bresult<String> {
    let text = {
        let cmdstruct = cmd.lock().unwrap();
        let amt =
            match Regex::new(r"^([+-])1")?.captures(&cmdstruct.msg) {
                None => return Ok("SKIP".into()),
                Some(cap) =>
                    if cmdstruct.id == cmdstruct.to { return Ok("SKIP self plussed".into()); }
                    else if &cap[1] == "+" { 1 } else { -1 }
        };
        // Load database of users
        let mut people :HashMap<i64, String> = HashMap::new();
        for l in read_to_string("tmbot/users.txt").unwrap().lines() {
            let mut v = l.split(" ");
            let id = v.next().ok_or("User DB malformed.")?.parse::<i64>().unwrap();
            let name = v.next().ok_or("User DB malformed.")?.to_string();
            people.insert(id, name);
        }
        info!("{:?}", people);

        // Load/update/save likes
        let likes = read_to_string( format!("tmbot/{}", cmdstruct.to) )
            .unwrap_or("0".to_string())
            .lines()
            .nth(0).unwrap()
            .parse::<i32>()
            .unwrap() + amt;

        info!("update likes in filesystem {:?} by {} to {:?}  write => {:?}",
            cmdstruct.to, amt, likes,
            write( format!("tmbot/{}", cmdstruct.to), likes.to_string()));

        let sfrom = cmdstruct.id.to_string();
        let sto = cmdstruct.to.to_string();
        let fromname = people.get(&cmdstruct.id).unwrap_or(&sfrom);
        let toname   = people.get(&cmdstruct.to).unwrap_or(&sto);
        format!("{}{}{}", fromname, num2heart(likes), toname)
    };

    send_msg(cmd.clone().into(), &text).await?;

    Ok("COMPLETED.".into())
}

async fn do_like_info (cmd:&Cmd) -> Bresult<&'static str> {
    let text = {
        let cmdstruct = cmd.lock().unwrap();
        if cmdstruct.msg != "+?" { return Ok("SKIP"); }
        let mut likes = Vec::new();
        // Over each user in file
        for l in read_to_string("tmbot/users.txt").unwrap().lines() {
            let mut v = l.split(" ");
            let id = v.next().ok_or("User DB malformed.")?;
            let name = v.next().ok_or("User DB malformed.")?.to_string();
            // Read the user's count file
            let count =
                read_to_string( "tmbot/".to_string() + &id )
                .unwrap_or("0".to_string())
                .trim()
                .parse::<i32>().or(Err("user like count parse i32 error"))?;
            likes.push((count, name));
        }
        likes.sort_by(|a,b| b.0.cmp(&a.0));
        likes.iter()
            .map( |(likecount, username)|
                format!("{}{} ", username, num2heart(*likecount)) )
            .collect::<Vec<String>>().join("")
    };
    //info!("HEARTS -> msg tmbot {:?}", send_msg(env, chat_id, &(-6..=14).map( |n| num2heart(n) ).collect::<Vec<&str>>().join("")).await);
    send_msg(MsgCmd::from(cmd.clone()).level(1), &text).await?;
    Ok("COMPLETED.")
}

async fn do_def (cmd:&Cmd) -> Bresult<&'static str> {

    let word = {
        let cmdstruct = cmd.lock().unwrap();
        let cap = Regex::new(r"^([A-Za-z-]+):$").unwrap().captures(&cmdstruct.msg);
        if cap.is_none() { return Ok("SKIP"); }
        cap.unwrap()[1].to_string()
    };

    info!("looking up {:?}", word);

    let mut msg = String::new();

    // Definitions

    let defs = get_definition(&word).await?;

    if defs.is_empty() {
        send_msg_markdown_id(cmd.clone().into(), &format!("*{}* def is empty", &word)).await?;
    } else {
        msg.push_str( &format!("*{}", &word) );
        if 1 == defs.len() { // If multiple definitions, leave off the colon
            msg.push_str( &format!(":* {}", defs[0].to_string().replacen("`", "\\`", 10000)));
        } else {
            msg.push_str( &format!(" ({})* {}", 1, defs[0].to_string().replacen("`", "\\`", 10000)));
            for i in 1..std::cmp::min(4, defs.len()) {
                msg.push_str( &format!(" *({})* {}", i+1, defs[i].to_string().replacen("`", "\\`", 10000)));
            }
        }
    }

    // Synonym

    let mut syns = get_syns(&word).await?;

    if syns.is_empty() {
        send_msg_markdown(cmd.clone().into(), &format!("*{}* synonyms is empty", &word)).await?;
    } else {
        if msg.is_empty() {
            msg.push_str( &format!("*{}:* _", &word) );
        } else {
            msg.push_str("\n_");
        }
        syns.truncate(12);
        msg.push_str( &syns.join(", ") );
        msg.push_str("_");
    }

    if !msg.is_empty() {
        send_msg_markdown( MsgCmd::from(cmd.clone()).level(1), &msg).await?;
    }
    Ok("COMPLETED.")
}

async fn do_sql (cmd:&Cmd) -> Bresult<&'static str> {

    let rows = {
        let cmdstruct = cmd.lock().unwrap();
        if cmdstruct.id != 308188500 { return Ok("do_sql invalid user"); }
        let rev = regex_to_vec("^(.*)ß$", &cmdstruct.msg)?;
        let expr :&str = if rev.is_empty() { return Ok("SKIP") } else { rev.as_istr(1)? };
        getsql!(cmdstruct.dbconn, expr)
    };

    let results =
        match rows {
            Err(e)  => {
                send_msg( MsgCmd::from(cmd.clone()), &format!("{:?}", e)).await?;
                return Err(e)
            }
            Ok(r) => r
        };

    // SQL rows to single string.
    let msg =
        results.iter().fold(
            String::new(),
            |mut b, vv| {
                vv.iter().for_each( |(key,val)| {b+=&format!(" {}:{}", key, val);} );
                b+="\n";
                b
            } );

    if msg.is_empty() { send_msg(cmd.clone().into(), "empty results").await?; }
    else { send_msg(cmd.clone().into(), &msg).await?; }

    Ok("COMPLETED.")
}

async fn do_quotes (cmd :&Cmd) -> Bresult<&'static str> {

    let (tickers, msg_id_changed) = {
        let cmdstruct = cmd.lock().unwrap();
        let envstruct = cmdstruct.env.lock().unwrap();
        (extract_tickers(&cmdstruct.msg),
         envstruct.message_id_read != cmdstruct.message_id)
    };

    if tickers.is_empty() { return Ok("SKIP") }

    if msg_id_changed {
        let id = send_msg(MsgCmd::from(cmd.clone()).level(1), "…").await?;
        let cmdstruct = cmd.lock().unwrap();
        let mut envstruct = cmdstruct.env.lock().unwrap();
        envstruct.message_id_read = cmdstruct.message_id;
        envstruct.message_id_write = id;
        envstruct.message_buff_write.clear();
    }

    let mut didwork = false;
    for ticker in &tickers {
        // Catch error and continue looking up tickers
        match get_quote_pretty(cmd, ticker).await {
            Ok(res) => {
                info!("get_quote_pretty {:?}", res);
                didwork = true;
                let (msg_id_write, msg_buff_write) = {
                    let cmdstruct = cmd.lock().unwrap();
                    let mut envstruct = cmdstruct.env.lock().unwrap();
                    envstruct.message_buff_write.push_str(&res);
                    envstruct.message_buff_write.push('\n');
                    ( envstruct.message_id_write,
                      envstruct.message_buff_write.to_string())
                };
                send_edit_msg_markdown(MsgCmd::from(cmd.clone()).level(1), msg_id_write, &msg_buff_write).await?;
            },
            e => { glogd!("get_quote_pretty => ", e); }
        }
    }

    // Notify if no results (message not saved)
    if !didwork {
        let msgcmd = MsgCmd::from(cmd.clone()).level(1);
        let cmdstruct = cmd.lock().unwrap();
        let envstruct = cmdstruct.env.lock().unwrap();
        send_edit_msg_markdown(
            msgcmd,
            envstruct.message_id_write,
            &format!("{}\nno results", &envstruct.message_buff_write)
        ).await?;
    }
    Ok("COMPLETED.")
}

async fn do_portfolio (cmd:&Cmd) -> Bresult<&'static str> {
    let (is_new_id, positions) = {
        let cmdstruct = cmd.lock().unwrap();
        if Regex::new(r"/STONKS").unwrap().find(&cmdstruct.msg.to_uppercase()).is_none() { return Ok("SKIP"); }
        let envstruct = cmdstruct.env.lock().unwrap();
        (envstruct.message_id_read != cmdstruct.message_id,
         Position::get_users_positions(&cmdstruct)?)
    };

    if is_new_id {
        // Keep message_id of this placeholder message so we continue to update it
        let new_write_id = send_msg( MsgCmd::from(cmd.clone()).level(1), "…").await?;

        let cmdstruct = cmd.lock().unwrap();
        let mut envstruct = cmdstruct.env.lock().unwrap();

        envstruct.message_id_read = cmdstruct.message_id;
        envstruct.message_id_write = new_write_id;
        envstruct.message_buff_write.clear();
    }

    let mut total = 0.0;
    for mut pos in positions {
        if !is_self_stonk(&pos.ticker) {
            let (msg_id_write, msg_buff_write) = {
                pos.update_quote(cmd).await?;
                let cmdstruct = cmd.lock().unwrap();
                let mut envstruct = cmdstruct.env.lock().unwrap();
                info!("{} position {:?}", cmdstruct.id, &pos);
                envstruct.message_buff_write.push_str(&pos.format_position(&cmdstruct)?);
                total += pos.qty * pos.quote.unwrap().price;
                ( envstruct.message_id_write, envstruct.message_buff_write.to_string() )
            };
            send_edit_msg_markdown( MsgCmd::from(cmd.clone()), msg_id_write, &msg_buff_write).await?;
        }
    }

    let (msg_id_write, msg_buff_write) = {
        let cmdstruct = cmd.lock().unwrap();
        let mut envstruct = cmdstruct.env.lock().unwrap();
        let cash = get_bank_balance(&cmdstruct)?;
        envstruct.message_buff_write.push_str(&format!("\n`{:7.2}``Cash`    `YOLO``{:.2}`\n", roundcents(cash), roundcents(total+cash)));
        ( envstruct.message_id_write, envstruct.message_buff_write.to_string() )
    };

    send_edit_msg_markdown( MsgCmd::from(cmd.clone()), msg_id_write, &msg_buff_write).await?;
    Ok("COMPLETED.")
}

// Handle: /yolo
async fn do_yolo (cmd:&Cmd) -> Bresult<&'static str> {
    let rows = {
        let cmdstruct = cmd.lock().unwrap();
        if Regex::new(r"/YOLO").unwrap().find(&cmdstruct.msg.to_uppercase()).is_none() { return Ok("SKIP"); }
        getsql!(cmdstruct.dbconn, "\
            SELECT ticker \
            FROM positions \
            WHERE positions.ticker NOT LIKE '0%' \
            AND positions.ticker NOT LIKE '1%' \
            AND positions.ticker NOT LIKE '2%' \
            AND positions.ticker NOT LIKE '3%' \
            AND positions.ticker NOT LIKE '4%' \
            AND positions.ticker NOT LIKE '5%' \
            AND positions.ticker NOT LIKE '6%' \
            AND positions.ticker NOT LIKE '7%' \
            AND positions.ticker NOT LIKE '8%' \
            AND positions.ticker NOT LIKE '9%' \
            AND positions.ticker NOT LIKE '@%' \
            GROUP BY ticker")?
    };

    let message_id = send_msg( MsgCmd::from(cmd.clone()).level(1), &"working...").await?;

    // Update all user-positioned tickers
    for row in rows {
        let ticker = row.get("ticker").unwrap();
        //working_message.push_str(ticker);
        //working_message.push_str("...");
        //send_edit_msg(cmd, message_id, &working_message).await?;
        info!("Stonk \x1b[33m{:?}", Quote::get_market_quote(cmd, ticker).await?);
    }

    /* Everyone's YOLO including non positioned YOLOers
    let sql = "\
    SELECT name, round(sum(value),2) AS yolo \
    FROM ( \
            SELECT id, qty*stonks.price AS value \
            FROM positions \
            LEFT JOIN stonks ON positions.ticker = stonks.ticker \
        UNION \
            SELECT id, balance as value FROM accounts \
        ) \
    NATURAL JOIN entitys \
    GROUP BY name \
    ORDER BY yolo DESC"; */

    // Everyone's YOLO including non positioned YOLOers
    let sql = "SELECT name, ROUND(value + balance, 2) AS yolo \
               FROM (SELECT positions.id, SUM(qty*stonks.price) AS value \
                     FROM positions \
                     LEFT JOIN stonks ON stonks.ticker = positions.ticker \
                     WHERE positions.ticker NOT LIKE '0%' \
                       AND positions.ticker NOT LIKE '1%' \
                       AND positions.ticker NOT LIKE '2%' \
                       AND positions.ticker NOT LIKE '3%' \
                       AND positions.ticker NOT LIKE '4%' \
                       AND positions.ticker NOT LIKE '5%' \
                       AND positions.ticker NOT LIKE '6%' \
                       AND positions.ticker NOT LIKE '7%' \
                       AND positions.ticker NOT LIKE '8%' \
                       AND positions.ticker NOT LIKE '9%' \
                       AND positions.ticker NOT LIKE '@%' \
                     GROUP BY id) \
               NATURAL JOIN accounts \
               NATURAL JOIN entitys \
               ORDER BY yolo DESC";

    let sql_results = {
        let cmdstruct = cmd.lock().unwrap();
        getsql!(cmdstruct.dbconn, &sql)?
    };

    // Build and send response string

    let mut msg = "*YOLOlians*".to_string();
    for row in sql_results {
        msg.push_str( &format!(" `{:.2}@{}`",
            row.get_f64("yolo")?,
            row.get("name").unwrap()) );
    }
    send_edit_msg_markdown( MsgCmd::from(cmd.clone()).level(1), message_id, &msg).await?;
    Ok("COMPLETED.")
}

// Returns quantiy and new bank balance if all good.  Otherwise an error string.
fn verify_qty (mut qty:f64, price:f64, bank_balance:f64) -> Result<(f64,f64), &'static str> {
    qty = roundqty(qty);
    let basis  = qty * price;
    let new_balance = bank_balance - basis;
    info!("\x1b[1mbuy? {} @ {} = {}  BANK {} -> {}", qty, price, basis,  bank_balance, new_balance);
    if qty < 0.0001 { return Err("Amount too low") }
    if new_balance < 0.0 { return Err("Need more cash") }
    Ok((qty, new_balance))
}

////////////////////////////////////////////////////////////////////////////////
/// Stonk Buy

#[derive(Debug)]
struct TradeBuy { qty:f64, price:f64, bank_balance: f64, hours: i64, trade: Trade }

impl TradeBuy {
    async fn new (trade:Trade) -> Bresult<Self> {
        let quote = Quote::get_market_quote(&trade.cmd, &trade.ticker).await?;

        if quote.exchange == "PNK" {
            send_msg_markdown( MsgCmd::from(trade.cmd.clone()), "`OTC / PinkSheet Verboten Stonken`").await?;
            Err("OTC / PinkSheet Verboten Stonken")?
        }
        let price = quote.price;
        let bank_balance = get_bank_balance( &trade.cmd.lock().unwrap() )?;
        let hours = quote.hours;
        let qty = match trade.amt {
            Some(amt) => if trade.is_dollars { amt/price } else { amt },
            None => roundqty(bank_balance / price) // Buy as much as possible
        };
        Ok( Self{qty, price, bank_balance, hours, trade} )
    }
}

#[derive(Debug)]
struct TradeBuyCalc { qty:f64, new_balance:f64, new_qty:f64, new_basis:f64, position:Position, new_position_p:bool, tradebuy:TradeBuy }

impl TradeBuyCalc {
    async fn compute_position (obj:TradeBuy) -> Bresult<Self> {
        let (qty, new_balance) =
            match
                verify_qty(obj.qty, obj.price, obj.bank_balance)
                .or_else( |_e| verify_qty(obj.qty-0.0001, obj.price, obj.bank_balance) )
                .or_else( |_e| verify_qty(obj.qty-0.0002, obj.price, obj.bank_balance) )
                .or_else( |_e| verify_qty(obj.qty-0.0003, obj.price, obj.bank_balance) )
                .or_else( |_e| verify_qty(obj.qty-0.0004, obj.price, obj.bank_balance) )
                .or_else( |_e| verify_qty(obj.qty-0.0005, obj.price, obj.bank_balance) ) {
                Err(e) => {  // Message user problem and log
                    send_msg_id( MsgCmd::from(obj.trade.cmd.clone()), &e).await?;
                    return Err(e.into())
                },
                Ok(r) => r
            };
        let id = {
            obj.trade.cmd.lock().unwrap().id
        };
        let position = Position::query( &obj.trade.cmd, id, &obj.trade.ticker).await?;
        let new_position_p = position.qty == 0.0;

        let qty_old = position.qty;
        let price_old = position.price;
        let price_new = position.quote.as_ref().unwrap().price;
        let (new_qty, new_basis) = {
            //let qty_old = position.qty;
            //let price_old = position.quote.unwrap().price;
            let new_basis = (qty * price_new + qty_old * price_old) / (qty + qty_old);
            let new_qty = roundqty(qty + qty_old);
            (new_qty, new_basis)
        };
        Ok(Self{qty, new_balance, new_qty, new_basis, position, new_position_p, tradebuy:obj})
    }
}

#[derive(Debug)]
struct ExecuteBuy { msg:String, tradebuycalc:TradeBuyCalc }

impl ExecuteBuy {
    fn execute (mut obj:TradeBuyCalc) -> Bresult<Self> {
        let msg = {
            let now = Instant::now().seconds();
            if obj.tradebuy.hours!=24 && !trading_hours_p( obj.tradebuy.trade.cmd.lock().unwrap().env.lock().unwrap().dst_hours_adjust, now)? {
                return Ok( ExecuteBuy {
                    msg:"Unable to trade after hours".into(),
                    tradebuycalc:obj
                } )
            }
            let cmdstruct = obj.tradebuy.trade.cmd.lock().unwrap();
            let id = cmdstruct.id;
            let ticker = &obj.tradebuy.trade.ticker;
            let price = obj.tradebuy.price;

            sql_table_order_insert(&cmdstruct.dbconn, id, ticker, obj.qty, price, now)?;
            let mut msg = format!("*Bought:*");

            if obj.new_position_p {
                getsql!( &cmdstruct.dbconn, "INSERT INTO positions VALUES (?, ?, ?, ?)", id, &**ticker, obj.new_qty, obj.new_basis)?;
            } else {
                msg += &format!("  `{:.2}``{}` *{}*_@{}_", obj.qty*price, ticker, obj.qty, price);
                info!("\x1b[1madd to existing position:  {} @ {}  ->  {} @ {}", obj.position.qty, obj.position.quote.as_ref().unwrap().price, obj.new_qty, obj.new_basis);
                getsql!( &cmdstruct.dbconn, "UPDATE positions SET qty=?, price=? WHERE id=? AND ticker=?", obj.new_qty, obj.new_basis, id, &**ticker)?;
            }
            obj.position.qty = obj.new_qty;
            obj.position.price = obj.new_basis;
            msg += &obj.position.format_position(&cmdstruct)?;

            getsql!( &cmdstruct.dbconn, "UPDATE accounts SET balance=? WHERE id=?", obj.new_balance, id)?;
            msg
        };

        Ok(Self{msg, tradebuycalc: obj})
    }
}

async fn do_trade_buy (cmd:&Cmd) -> Bresult<&'static str> {
    let trade = Trade::new(cmd);
    if trade.as_ref().map_or(true, |trade| trade.action != '+') { return Ok("SKIP") }

    let res =
        TradeBuy::new(trade.unwrap()).await
        .map(|tradebuy| TradeBuyCalc::compute_position(tradebuy))?.await
        .map(|tradebuycalc| ExecuteBuy::execute(tradebuycalc))?
        .map(|res| { info!("\x1b[1;31mResult {:#?}", &res); res })?;

    send_msg_markdown( MsgCmd::from(cmd.clone()), &res.msg).await?; // Report to group
    Ok("COMPLETED.")
}

////////////////////////////////////////////////////////////////////////////////
/// Stonk Sell

#[derive(Debug)]
struct TradeSell {
    position: Position,
    qty: f64,
    price: f64,
    bank_balance: f64,
    new_balance: f64,
    new_qty: f64,
    hours: i64,
    trade: Trade
}

impl TradeSell {
    async fn new (trade:Trade) -> Bresult<Self> {
        let id = { trade.cmd.lock().unwrap().id };
        let ticker = &trade.ticker;

        let mut positions = Position::get_position( &trade.cmd, id, ticker)?;
        if positions.is_empty() {
            send_msg_id( MsgCmd::from(trade.cmd.clone()), "You lack a valid position.").await?;
            Err("expect 1 position in table")?
        }
        let mut position = positions.pop().unwrap();
        position.update_quote(&trade.cmd).await?;

        let pos_qty = position.qty;
        let quote = &position.quote.as_ref().unwrap();
        let price = quote.price;
        let hours = quote.hours;

        let mut qty =
            roundqty(match trade.amt {
                Some(amt) => if trade.is_dollars { amt / price } else { amt }, // Convert dollars to shares maybe
                None => pos_qty // no amount set, so set to entire qty
            });
        if qty <= 0.0 {
            send_msg_id( MsgCmd::from(trade.cmd.clone()), "Quantity too low.").await?;
            Err("sell qty too low")?
        }

        let bank_balance = get_bank_balance( &trade.cmd.lock().unwrap() )?;
        let mut gain = qty*price;
        let mut new_balance = bank_balance+gain;

        info!("\x1b[1msell? {}/{} @ {} = {}  CASH {} -> {}", qty, pos_qty, price, gain,  bank_balance, new_balance);

        // If equal to the rounded position value, snap qty to exact position
        if qty != pos_qty && roundcents(gain) == roundcents(pos_qty*price) {
            qty = pos_qty;
            gain = qty*price;
            new_balance = bank_balance + gain;
            info!("\x1b[1msell? {}/{} @ {} = {}  BANK {} -> {}", qty, pos_qty, price, gain,  bank_balance, new_balance);
        }

        if pos_qty < qty {
            send_msg_id( MsgCmd::from(trade.cmd.clone()), "You can't sell more than you own.").await?;
            return Err("not enough shares to sell".into());
        }

        let new_qty = roundqty(pos_qty-qty);

        Ok( Self{position, qty, price, bank_balance, new_balance, new_qty, hours, trade} )
    }
}

#[derive(Debug)]
struct ExecuteSell {
    msg: String,
    tradesell: TradeSell
}

impl ExecuteSell {
    fn execute (mut obj:TradeSell) -> Bresult<Self> {
        let msg = {
            let now = Instant::now().seconds();
            if obj.hours!=24 && !trading_hours_p(obj.trade.cmd.lock().unwrap().env.lock().unwrap().dst_hours_adjust, now)? {
                return Ok(Self{msg:"Unable to trade after hours".into(), tradesell:obj});
            }
            let cmdstruct = obj.trade.cmd.lock().unwrap();
            let id = cmdstruct.id;
            let ticker = &obj.trade.ticker;
            let qty = obj.qty;
            let price = obj.price;
            let new_qty = obj.new_qty;
            sql_table_order_insert(&cmdstruct.dbconn, id, ticker, -qty, price, now)?;
            let mut msg = format!("*Sold:*");
            if new_qty == 0.0 {
                getsql!( &cmdstruct.dbconn, "DELETE FROM positions WHERE id=? AND ticker=?", id, &**ticker)?;
                msg += &obj.position.format_position(&cmdstruct)?;
            } else {
                msg += &format!("  `{:.2}``{}` *{}*_@{}_{}",
                    qty*price, ticker, qty, price,
                    &obj.position.format_position(&cmdstruct)?);
                getsql!( &cmdstruct.dbconn, "UPDATE positions SET qty=? WHERE id=? AND ticker=?", new_qty, id, &**ticker)?;
            }
            getsql!( &cmdstruct.dbconn, "UPDATE accounts SET balance=? WHERE id=?", obj.new_balance, id)?;
            msg
        };
        Ok(Self{msg, tradesell:obj})
    }
}

async fn do_trade_sell (cmd :&Cmd) -> Bresult<&'static str> {
    let trade = Trade::new(cmd);
    if trade.as_ref().map_or(true, |trade| trade.action != '-') { return Ok("SKIP") }

    let res =
        TradeSell::new(trade.unwrap()).await
        .map(|tradesell| ExecuteSell::execute(tradesell))??;

    info!("\x1b[1;31mResult {:#?}", res);
    send_msg_markdown( MsgCmd::from(cmd.clone()), &res.msg).await?; // Report to group
    Ok("COMPLETED.")
}


////////////////////////////////////////////////////////////////////////////////
/// General Exchange Market Place
/*
    Create an ask quote (+price) on the exchange table, lower is better.
            [[ 0.01  0.30  0.50  0.99   1.00  1.55  2.00  9.00 ]]
    Best BID price (next buyer)--^      ^--Best ASK price (next seller)
    Positive quantity                      Negative quantity
*/ 

#[derive(Debug)]
struct ExQuote { // Local Exchange Quote Structure
    id: i64,
    thing: String,
    qty: f64,
    price: f64,
    ticker: String,
    now: i64,
    cmd: Cmd
}

impl ExQuote {
    fn scan (cmd:&Cmd) -> Bresult<Option<Self>> {
        let cmdstruct = cmd.lock().unwrap();
         //                         ____ticker____  _____________qty____________________  $@  ___________price______________
        let caps = regex_to_vec(r"^(@[A-Za-z^.-_]+)([+-]([0-9]+[.]?|[0-9]*[.][0-9]{1,4}))[$@]([0-9]+[.]?|[0-9]*[.][0-9]{1,2})$", &cmdstruct.msg)?;
        if caps.is_empty() { return Ok(None) }

        let thing = caps.as_string(1)?;
        let qty   = roundqty(caps.as_f64(2)?);
        let price = caps.as_f64(4)?;
        let now   = Instant::now().seconds();
        deref_ticker(&cmdstruct.dbconn, &thing) // -> Option<String>
        .map_or( Ok(None), |ticker| // String
            Ok(Some(ExQuote {
                id: cmdstruct.id,
                thing, qty, price, ticker, now,
                cmd: cmd.clone()} )))
    }
}

#[derive(Debug)]
struct QuoteCancelMine {
    qty: f64,
    myasks: Vec<HashMap<String,String>>,
    myasksqty: f64,
    mybids: Vec<HashMap<String,String>>,
    exquote: ExQuote,
    msg: String
}

impl QuoteCancelMine {
    fn doit (exquote :ExQuote) -> Bresult<Self> {
        let id = exquote.id;
        let ticker = &exquote.ticker;
        let mut qty = exquote.qty;
        let price = exquote.price;
        let mut msg = String::new();

        let (myasks, myasksqty, mybids) = {
            let dbconn = &exquote.cmd.lock().unwrap().dbconn;

            let mut myasks = getsql!(dbconn, "SELECT * FROM exchange WHERE id=? AND ticker=? AND qty<0.0 ORDER BY price", id, &**ticker)?;
            let myasksqty = -roundqty(myasks.iter().map( |ask| ask.get_f64("qty").unwrap() ).sum::<f64>());
            let mut mybids = getsql!(dbconn, "SELECT * FROM exchange WHERE id=? AND ticker=? AND 0.0<=qty ORDER BY price DESC", id, &**ticker)?;

            if qty < 0.0 { // This is an ask exquote
                // Remove/decrement a matching bid in the current settled market
                if let Some(mybid) = mybids.iter_mut().find( |b| price == b.get_f64("price").unwrap() ) {
                    let bidqty = roundqty(mybid.get_f64("qty")?);
                    if bidqty <= -qty {
                        getsql!(dbconn, "DELETE FROM exchange WHERE id=? AND ticker=? AND qty=? AND price=?", id, &**ticker, bidqty, price)?;
                        mybid.insert("*UPDATED*".into(), "*REMOVED*".into());
                        qty = roundqty(qty + bidqty);
                        msg += &format!("\n*Removed bid:* `{}+{}@{}`", exquote.thing, bidqty, price);
                    } else {
                        let newbidqty = roundqty(bidqty+qty);
                        getsql!(dbconn, "UPDATE exchange SET qty=? WHERE id=? AND ticker=? AND qty=? AND price=?", newbidqty, id, &**ticker, bidqty, price)?;
                        mybid.insert("*UPDATED*".into(), format!("*QTY={:.}*", newbidqty));
                        qty = 0.0;
                        msg += &format!("\n*Updated bid:* `{}+{}@{}` -> `{}@{}`", exquote.thing, bidqty, price, newbidqty, price);
                    }
                }
            } else if 0.0 < qty { // This is a bid exquote
                // Remove/decrement a matching ask in the current settled market
                if let Some(myask) = myasks.iter_mut().find( |b| price == b.get_f64("price").unwrap() ) {
                    let askqty = roundqty(myask.get_f64("qty")?);
                    if -askqty <= qty {
                        getsql!(dbconn, "DELETE FROM exchange WHERE id=? AND ticker=? AND qty=? AND price=?", id, &**ticker, askqty, price)?;
                        myask.insert("*UPDATED*".into(), "*REMOVED*".into());
                        qty = roundqty(qty + askqty);
                        msg += &format!("\n*Removed ask:* `{}{}@{}`", exquote.thing, askqty, price);
                    } else {
                        let newaskqty = roundqty(askqty+qty);
                        getsql!(dbconn, "UPDATE exchange SET qty=? WHERE id=? AND ticker=? AND qty=? AND price=?", newaskqty, id, &**ticker, askqty, price)?;
                        myask.insert("*UPDATED*".into(), format!("*QTY={:.}*", newaskqty));
                        qty = 0.0;
                        msg += &format!("\n*Updated ask:* `{}{}@{}` -> `{}@{}`", exquote.thing, askqty, price, newaskqty, price);
                    }
                }
            }
            (myasks, myasksqty, mybids)
        };

        Ok(Self{qty, myasks, myasksqty, mybids, exquote, msg})
    }
}

#[derive(Debug)]
struct QuoteExecute {
    qty: f64,
    bids: Vec<HashMap<String,String>>,
    quotecancelmine: QuoteCancelMine,
    msg: String
}

impl QuoteExecute {
    async fn doit (mut obj :QuoteCancelMine) -> Bresult<Self> {
        let id = obj.exquote.id;
        let ticker = &obj.exquote.ticker;
        let price = obj.exquote.price;
        let mut qty = obj.qty;
        let now = obj.exquote.now;
        let myasks = &mut obj.myasks;
        let mybids = &mut obj.mybids;
        let asksqty = obj.myasksqty;

        let (mut bids, mut asks) = {
            let cmdstruct = obj.exquote.cmd.lock().unwrap();
            // Other bids / asks (not mine) global since it's being updated and returned for logging
            (
                getsql!( &cmdstruct.dbconn,
                    "SELECT * FROM exchange WHERE id!=? AND ticker=? AND 0.0<qty ORDER BY price DESC",
                    id, &**ticker)?,
                getsql!( &cmdstruct.dbconn,
                    "SELECT * FROM exchange WHERE id!=? AND ticker=? AND qty<0.0 ORDER BY price",
                    id, &**ticker)?
            )
        };

        let mut msg = obj.msg.to_string();
        let position = Position::query( &obj.exquote.cmd, id, ticker).await?;
        let posqty = position.qty;
        let posprice = position.price;

        if qty < 0.0 { // This is an ask exquote
            if posqty < -qty + asksqty { // does it exceed my current ask qty?
                msg += "\nYou lack that available quantity to sell.";
            } else {
                for abid in bids.iter_mut().filter( |b| price <= b.get_f64("price").unwrap() ) {
                    let aid = abid.get_i64("id")?;
                    let aqty = roundqty(abid.get_f64("qty")?);
                    let aprice = abid.get_f64("price")?;
                    let xqty = aqty.min(-qty); // quantity exchanged is the smaller of the two (could be equal)

                    let dbconn = &obj.exquote.cmd.lock().unwrap().dbconn;

                    if xqty == aqty { // bid to be entirely settled
                        getsql!(dbconn, "DELETE FROM exchange WHERE id=? AND ticker=? AND qty=? AND price=?", aid, &**ticker, aqty, aprice)?;
                        abid.insert("*UPDATED*".into(), "*REMOVED*".into());
                    } else {
                        getsql!(dbconn, "UPDATE exchange set qty=? WHERE id=? AND ticker=? AND qty=? AND price=?", aqty-xqty, aid, &**ticker, aqty, aprice)?;
                        abid.insert("*UPDATED*".into(), format!("*qty={}*", aqty-xqty));
                    }

                    // Update order table with the purchase and buy info
                    sql_table_order_insert(dbconn, id, ticker, -xqty, aprice, now)?;
                    sql_table_order_insert(dbconn, aid, ticker, xqty, aprice, now)?;

                    // Update each user's account
                    let value = xqty * aprice;
                    getsql!(dbconn, "UPDATE accounts SET balance=balance+? WHERE id=?",  value, id)?;
                    getsql!(dbconn, "UPDATE accounts SET balance=balance+? WHERE id=?", -value, aid)?;

                    msg += &format!("\n*Settled:*\n{} `{}{:+}@{}` <-> `${}` {}",
                            reference_ticker(dbconn, &id.to_string()), obj.exquote.thing, xqty, aprice,
                            value, reference_ticker(dbconn, &aid.to_string()));

                    // Update my position
                    let newposqty = roundqty(posqty - xqty);
                    if 0.0 == newposqty {
                        getsql!(dbconn, "DELETE FROM positions WHERE id=? AND ticker=? AND qty=?", id, &**ticker, posqty)?;
                    } else {
                        getsql!(dbconn, "UPDATE positions SET qty=? WHERE id=? AND ticker=? AND qty=?", newposqty, id, &**ticker, posqty)?;
                    }

                    // Update buyer's position
/**/                let aposition = Position::query( &obj.exquote.cmd, aid, ticker).await?;
                    let aposqty = aposition.qty;
                    let aposprice = aposition.price;

                    if 0.0 == aposqty {
                        getsql!(dbconn, "INSERT INTO positions values(?, ?, ?, ?)", aid, &**ticker, xqty, value)?;
                    } else {
                        let newaposqty = roundqty(aposqty+aqty);
                        let newaposcost = (aposprice + value) / (aposqty + xqty);
                        getsql!(dbconn, "UPDATE positions SET qty=?, price=? WHERE id=? AND ticker=? AND qty=?", newaposqty, newaposcost, aid, &**ticker, aposqty)?;
                    }

                    // Update stonk exquote value
/**/                let last_price = Quote::get_market_quote( &obj.exquote.cmd, ticker).await?.price;
                    getsql!(dbconn, "UPDATE stonks SET price=?, last=?, time=? WHERE ticker=?", aprice, last_price, now, &**ticker)?;

                    qty = roundqty(qty+xqty);
                    if 0.0 <= qty { break }
                } // for

                // create or increment in exchange table my ask.  This could also be the case if no bids were executed.
                if qty < 0.0 {
                    let dbconn = &obj.exquote.cmd.lock().unwrap().dbconn;
                    if let Some(myask) = myasks.iter_mut().find( |a| price == a.get_f64("price").unwrap() ) {
                        let oldqty = myask.get_f64("qty").unwrap();
                        let newqty = roundqty(oldqty + qty); // both negative, so "increased" ask qty
                        let mytime = myask.get_i64("time").unwrap();
                        getsql!(dbconn, "UPDATE exchange set qty=? WHERE id=? AND ticker=? AND price=? AND time=?", newqty, id, &**ticker, price, mytime)?;
                        myask.insert("*UPDATED*".into(), format!("qty={}", newqty));
                        msg += &format!("\n*Updated ask:* `{}{}@{}` -> `{}@{}`", obj.exquote.thing, oldqty, price, newqty, price);
                    } else {
                        getsql!(dbconn, "INSERT INTO exchange VALUES (?, ?, ?, ?, ?)", id, &**ticker, &*format!("{:.4}", qty), &*format!("{:.4}", price), now)?;
                        // Add a fake entry to local myasks vector for logging's sake
                        let mut hm = HashMap::<String,String>::new();
                        hm.insert("*UPDATED*".to_string(), format!("*INSERTED* {} {} {} {} {}", id, ticker, qty, price, now));
                        myasks.push(hm);
                        msg += &format!("\n*Created ask:* `{}{}@{}`", obj.exquote.thing, qty, price);
                    }
                }
            }
        } else if 0.0 < qty { // This is a bid exquote (want to buy someone)
            // Limited by available cash but that's up to the current best ask price and sum of ask costs.
            let basis = qty * price;
            let bank_balance = get_bank_balance( &obj.exquote.cmd.lock().unwrap() )?;
            let mybidsprice =
                mybids.iter().map( |bid|
                    bid.get_f64("qty").unwrap()
                    * bid.get_f64("price").unwrap() )
                .sum::<f64>();

            if bank_balance < mybidsprice + basis { // Verify not over spending as this order could become a bid
                msg += "Available cash lacking for this bid.";
            } else {
                for aask in asks.iter_mut().filter( |a| a.get_f64("price").unwrap() <= price ) {
                    //error!("{:?}", mybid);
                    let aid = aask.get_i64("id")?;
                    let aqty = roundqty(aask.get_f64("qty")?); // This is a negative value
                    let aprice = aask.get_f64("price")?;
                    //error!("my ask qty {}  asksqty sum {}  bid qty {}", qty, asksqty, aqty);


                    let xqty = roundqty(qty.min(-aqty)); // quantity exchanged is the smaller of the two (could be equal)

                    {
                        let cmdstruct = obj.exquote.cmd.lock().unwrap();

                        if xqty == -aqty { // ask to be entirely settled
                            getsql!( &cmdstruct.dbconn, "DELETE FROM exchange WHERE id=? AND ticker=? AND qty=? AND price=?", aid, &**ticker, aqty, aprice)?;
                            aask.insert("*UPDATED*".into(), "*REMOVED*".into());
                        } else {
                            let newqty = aqty+xqty;
                            getsql!( &cmdstruct.dbconn, "UPDATE exchange set qty=? WHERE id=? AND ticker=? AND qty=? AND price=?", newqty, aid, &**ticker, aqty, aprice)?;
                            aask.insert("*UPDATED*".into(), format!("*qty={}*", newqty));
                        }

                        //error!("new order {} {} {} {} {}", id, ticker, aqty, price, now);
                        // Update order table with the purchase and buy info
                        sql_table_order_insert(&cmdstruct.dbconn, id, ticker, xqty, aprice, now)?;
                        sql_table_order_insert(&cmdstruct.dbconn, aid, ticker, -xqty, aprice, now)?;

                        // Update each user's account
                        let value = xqty * aprice;
                        getsql!( &cmdstruct.dbconn, "UPDATE accounts SET balance=balance+? WHERE id=?", -value, id)?;
                        getsql!( &cmdstruct.dbconn, "UPDATE accounts SET balance=balance+? WHERE id=?",  value, aid)?;

                        msg += &format!("\n*Settled:*\n{} `${}` <-> `{}{:+}@{}` {}",
                                reference_ticker( &cmdstruct.dbconn, &id.to_string()), value,
                                obj.exquote.thing, xqty, aprice, reference_ticker(&cmdstruct.dbconn, &aid.to_string()));

                        // Update my position
                        if 0.0 == posqty {
                            getsql!( &cmdstruct.dbconn, "INSERT INTO positions values(?, ?, ?, ?)", id, &**ticker, xqty, value)?;
                        } else {
                            let newposqty = roundqty(posqty+xqty);
                            let newposcost = (posprice + value) / (posqty + xqty);
                            getsql!( &cmdstruct.dbconn, "UPDATE positions SET qty=?, price=? WHERE id=? AND ticker=? AND qty=?",
                                newposqty, newposcost, id, &**ticker, posqty)?;
                        }
                    }

                    {
                        // Update their position
    /**/                let aposition = Position::query( &obj.exquote.cmd, aid, ticker).await?;
                        let aposqty = aposition.qty;
                        let newaposqty = roundqty(aposqty - xqty);
                        let dbconn = &obj.exquote.cmd.lock().unwrap().dbconn;
                        if 0.0 == newaposqty {
                            getsql!(dbconn, "DELETE FROM positions WHERE id=? AND ticker=? AND qty=?", aid, &**ticker, aposqty)?;
                        } else {
                            getsql!(dbconn, "UPDATE positions SET qty=? WHERE id=? AND ticker=? AND qty=?", newaposqty, aid, &**ticker, aposqty)?;
                        }
                    }

                    // Update self-stonk exquote value
                    let last_price = Quote::get_market_quote( &obj.exquote.cmd, ticker).await?.price;
                    let dbconn = &obj.exquote.cmd.lock().unwrap().dbconn;
                    getsql!(dbconn, "UPDATE stonks SET price=?, last=?, time=?, WHERE ticker=?", aprice, last_price, now, &**ticker)?;

                    qty = roundqty(qty-xqty);
                    if qty <= 0.0 { break }
                } // for

                // create or increment in exchange table my bid.  This could also be the case if no asks were executed.
                if 0.0 < qty{
                    let dbconn = &obj.exquote.cmd.lock().unwrap().dbconn;
                    if let Some(mybid) = mybids.iter_mut().find( |b| price == b.get_f64("price").unwrap() ) {
                        let oldqty = mybid.get_f64("qty").unwrap();
                        let newqty = roundqty(oldqty + qty);
                        let mytime = mybid.get_i64("time").unwrap();
                        getsql!( dbconn, "UPDATE exchange set qty=? WHERE id=? AND ticker=? AND price=? AND time=?", newqty, id, &**ticker, price, mytime)?;
                        mybid.insert("*UPDATED*".into(), format!("qty={}", newqty));
                        msg += &format!("\n*Updated bid:* `{}+{}@{}` -> `{}@{}`", obj.exquote.thing, oldqty, price, newqty, price);
                    } else {
                        getsql!( dbconn, "INSERT INTO exchange VALUES (?, ?, ?, ?, ?)", id, &**ticker, &*format!("{:.4}",qty), &*format!("{:.4}",price), now)?;
                        // Add a fake entry to local mybids vector for logging's sake
                        let mut hm = HashMap::<String,String>::new();
                        hm.insert("*UPDATED*".to_string(), format!("*INSERTED* {} {} {} {} {}", id, ticker, qty, price, now));
                        mybids.push(hm);
                        msg += &format!("\n*Created bid:* `{}+{}@{}`", obj.exquote.thing, qty, price);
                    }
                }
            } // else
        } // if
        Ok(Self{qty, bids, quotecancelmine: obj, msg})
    }
}

async fn do_exchange_bidask (cmd :&Cmd) -> Bresult<&'static str> {

    let exquote = ExQuote::scan(cmd)?;
    let exquote = if exquote.is_none() { return Ok("SKIP") } else { exquote.unwrap() };

    let ret =
        QuoteCancelMine::doit(exquote)
        .map(|obj| QuoteExecute::doit(obj) )?.await?;
    info!("\x1b[1;31mResult {:#?}", ret);
    if 0!=ret.msg.len() {
        send_msg_markdown( MsgCmd::from(cmd.clone()),
            &ret.msg
            .replacen("_", "\\_", 1000)
            .replacen(">", "\\>", 1000)
        ).await?;
    } // Report to group
    Ok("COMPLETED.")
}

async fn do_orders (cmd:&Cmd) -> Bresult<&'static str> {
    let mut asks = String::from("");
    let mut bids = String::from("");
    let rows = {
        let cmdstruct = cmd.lock().unwrap();
        if Regex::new(r"/ORDERS").unwrap().find(&cmdstruct.msg.to_uppercase()).is_none() { return Ok("SKIP"); }
        let id = cmdstruct.id;
        for order in getsql!(&cmdstruct.dbconn, "SELECT * FROM exchange WHERE id=?", id)? {
            let ticker = order.get("ticker").unwrap();
            let stonk = reference_ticker(&cmdstruct.dbconn, ticker).replacen("_", "\\_", 10000);
            let qty = order.get_f64("qty")?;
            let price = order.get("price").unwrap();
            if qty < 0.0 {
                asks += &format!("\n{}{:+}@{}", stonk, qty, price);
            } else {
                bids += &format!("\n{}{:+}@{}", stonk, qty, price);
            }
        }

        // Include all self-stonks positions (mine and others)
        let sql = format!("
            SELECT {} AS id, id AS ticker, 0.0 AS qty, 0.0 AS price \
            FROM entitys \
            WHERE 0<id \
            AND id NOT IN (SELECT ticker FROM positions WHERE id={} GROUP BY ticker)
                UNION \
            SELECT * FROM positions WHERE id={} AND ticker IN (SELECT id FROM entitys WHERE 0<id)", id, id, id);
        getsql!(&cmdstruct.dbconn, sql)?
    };

    let mut msg = String::new();
    let mut total = 0.0;
    for pos in rows {
        if is_self_stonk(pos.get("ticker").unwrap()) {
            let mut pos = Position {
                quote: None,
                id: pos.get_i64("id")?,
                ticker: pos.get_str("ticker")?,
                qty: pos.get_f64("qty")?,
                price: pos.get_f64("price")?
            };
            pos.update_quote(cmd).await?;
            let cmdstruct = cmd.lock().unwrap();
            msg += &pos.format_position(&cmdstruct)?;
            total += pos.qty * pos.quote.unwrap().price;
        }
    }

    let cash = get_bank_balance(&cmd.lock().unwrap())?;
    msg += &format!("\n`{:7.2}``Cash`    `YOLO``{:.2}`",
        roundcents(cash),
        roundcents(total+cash));

    if 0 < msg.len() { msg += "\n" }
    if bids.len() == 0 && asks.len() == 0 {
        msg += "*NO BIDS/ASKS*"
    } else {
        if asks.len() != 0 {
            msg += "*ASKS:*";
            msg += &asks;
        } else {
            msg += "*ASKS:* none";
        }
        msg += "\n";
        if  bids.len() != 0 {
            msg += "*BIDS:*";
            msg += &bids;
        } else {
            msg += "*BIDS* none";
        }
    }

    info!("{:?}", &msg);
    send_msg_markdown( MsgCmd::from(cmd.clone()), &msg).await?;
    Ok("COMPLETED.")
}

async fn do_fmt (cmd :&Cmd) -> Bresult<&'static str> {

    let cmdstruct = cmd.lock().unwrap();

    let cap = Regex::new(r"^/fmt( ([qp?])[ ]?(.*)?)?$").unwrap().captures(&cmdstruct.msg);
    if cap.is_none() { return Ok("SKIP"); }
    let cap = cap.unwrap();

    if cap.get(1) == None {
        let res = getsql!(&cmdstruct.dbconn, r#"SELECT COALESCE(quote, "") as quote, COALESCE(position, "") as position FROM entitys LEFT JOIN formats ON entitys.id = formats.id WHERE entitys.id=?"#,
            cmdstruct.id)?;

        let fmt_quote =
            if res.is_empty() || res[0].get_str("quote")?.is_empty() {
                format!("DEFAULT {:?}", cmdstruct.fmt_quote.to_string())
            } else {
                res[0].get_str("quote")?
            };
        let fmt_position =
            if res.is_empty() || res[0].get_str("position")?.is_empty() {
                format!("DEFAULT {:?}", cmdstruct.fmt_position.to_string())
            } else {
                res[0].get_str("position")?
            };
        send_msg( MsgCmd::from(&*cmdstruct), &format!("fmt_quote {}\nfmt_position {}", fmt_quote, fmt_position)).await?;
        return Ok("COMPLETED.")
    }

    let c = match &cap[2] {
        "?" => {
            send_msg_markdown(
                MsgCmd::from(&*cmdstruct),
"` ™Bot Quote Formatting `
`%A` `Gain Arrow`
`%B` `Gain`
`%C` `Gain %`
`%D` `Gain Color`
`%E` `Ticker Symbol`
`%F` `Stock Price`
`%G` `Company Title`
`%H` `Market 'p're 'a'fter '∞'`
`%I` `Updated indicator`
`%[%nbiusq]` `% newline bold italics underline strikeout quote`
"
            ).await?;

            send_msg_markdown(
                MsgCmd::from(&*cmdstruct),
"` ™Bot Position Formatting `
`%A` `Value`
`%B` `Gain`
`%C` `Gain Arrow`
`%D` `Gain %`
`%E` `Gain Color`
`%F` `Ticker Symbol`
`%G` `Stock Price`
`%H` `Share Count`
`%I` `Cost Basis per Share`
`%J` `Day Color`
`%K` `Day Gain`
`%L` `Day Arrow`
`%M` `Day Gain %`
`%[%nbiusq]` `% newline bold italics underline strikeout quote`
"
            ).await?;
            return Ok("COMPLETED.");
        },
        "q" => "quote",
        _ => "position"
    };
    let s = &cap[3];

    info!("set format {} to {:?}", c, s);

    let res = getsql!(&cmdstruct.dbconn, "SELECT id FROM formats WHERE id=?", cmdstruct.id)?;
    if res.is_empty() {
        getsql!(&cmdstruct.dbconn, r#"INSERT INTO formats VALUES (?, "", "")"#, cmdstruct.id)?;
    }

    // notify user the change
    send_msg(
        MsgCmd::from(&*cmdstruct),
        &format!("fmt_{} {}",
            c,
            if s.is_empty() { "DEFAULT".to_string() } else { format!("{:?}", s) })).await?;

    // make change to DB
    getsql!(&cmdstruct.dbconn, format!("UPDATE formats SET {}=? WHERE id=?", c),
        &*s.replacen("\"", "\"\"", 10000),
        cmdstruct.id)?;

    Ok("COMPLETED.")
}


async fn do_rebalance (cmd :&Cmd) -> Bresult<&'static str> {

    let caps = regex_to_vec(
        //             _______float to 2 decimal places______                                    ____________float_____________
        r"^/REBALANCE( (-?[0-9]+[.]?|(-?[0-9]*[.][0-9]{1,2})))?(( [@^]?[A-Z_a-z][-.0-9=A-Z_a-z]* (([0-9]*[.][0-9]+)|([0-9]+[.]?)))+)",
        &cmd.lock().unwrap().msg.to_uppercase() )?;
    if caps.is_empty() { return Ok("SKIP") }

    let mut msg_feedback = format!("WallStreetBets Rebalancing Engine Activated...");
    let channel_id = send_msg(cmd.into(), &msg_feedback).await?;

    let percents = // HashMap of Ticker->Percent
        Regex::new(r" ([@^]?[A-Z_a-z][-.0-9=A-Z_a-z]*) (([0-9]*[.][0-9]+)|([0-9]+[.]?))")?
        .captures_iter(&caps.as_string(4)?)
        .map(|cap|
            ( cap.get(1).unwrap().as_str().to_string(),
              cap.get(2).unwrap().as_str().parse::<f64>().unwrap() / 100.0 ) )
        .collect::<HashMap<String, f64>>();

    // Refresh stonk quotes
    for ticker in percents.keys() {
        if !is_self_stonk(&ticker) {
            Quote::get_market_quote(cmd, &ticker).await.err().map_or(false, gwarn);
            // Update feedback message with ticker symbol
            msg_feedback.push_str(&format!("{} ", ticker));
            send_edit_msg(cmd.into(), channel_id, &msg_feedback).await?;
        }
    }

    let mut positions = {
        let cmdstruct = cmd.lock().unwrap();
        getsql!( &cmdstruct.dbconn, format!("
            SELECT
                positions.ticker,
                positions.qty,
                stonks.price,
                positions.qty*stonks.price AS value
            FROM positions
            LEFT JOIN stonks ON positions.ticker=stonks.ticker
            WHERE id={} AND positions.ticker IN ('{}')",
            cmdstruct.id,
            percents.keys().map(String::to_string).collect::<Vec<String>>().join("','")
        ))?
    };

    // Sum the optional offset amount and stonk values
    let mut total :f64 = caps.as_f64(2).unwrap_or(0.0);
    positions.iter().for_each(|hm| total += hm.get_f64("value").unwrap());
    info!("rebalance total {}", total);

    if 0==positions.len() {
        send_edit_msg(cmd.into(), channel_id, "no valid tickers").await?;
    } else {
        for i in 0..positions.len() {
            let ticker = positions[i].get_str("ticker")?;
            let value = positions[i].get_f64("value")?;
            let mut diff = roundfloat(percents.get(&ticker).unwrap() * total - value, 2);
            if -0.01 > diff || diff < 0.01 { diff = 0.0; } // under 1¢ diffs will be skipped
            positions[i].insert("diff".to_string(), diff.to_string()); // Add new key/val to Position HashMap
        }
        for i in 0..positions.len() {
            if positions[i].get_str("diff")?.chars().nth(0).unwrap() == '-'  {
                info!("rebalance position {:?}", positions[i]);
                let ticker = &positions[i].get_str("ticker")?;
                let diffstr = &positions[i].get_str("diff")?[1..];
                if "0" == diffstr {
                    msg_feedback.push_str(&format!("\n{} is balanced", ticker));
                    send_edit_msg(cmd.into(), channel_id, &msg_feedback).await?;
                } else {
                    let order = &format!("{}-${}", ticker, diffstr);
                    { cmd.lock().unwrap().msg = order.to_string(); }
                    glogd!(" do_trade_sell =>", do_trade_sell(cmd).await);
                }
            }
        }
        for i in 0..positions.len() {
            if positions[i].get_str("diff")?.chars().nth(0).unwrap() != '-'  {
                info!("rebalance position {:?}", positions[i]);
                let ticker = &positions[i].get_str("ticker")?;
                let mut diffstr = positions[i].get_str("diff")?;
                let bank_balance = { get_bank_balance(&cmd.lock().unwrap())?  };
                // The last buy might be so off, so skip or adjust to account value
                if bank_balance < diffstr.parse::<f64>()? {
                    if 0.01 <= bank_balance {
                        diffstr = format!("{}", (bank_balance*100.0) as i64 as f64 / 100.0);
                    } else {
                        diffstr = format!("0");
                    }
                    warn!("updated diff position {} {}", ticker, diffstr);
                }

                if "0" == diffstr {
                    msg_feedback.push_str(&format!("\n{} is balanced", ticker));
                    send_edit_msg(cmd.into(), channel_id, &msg_feedback).await?;
                } else {
                    let order = &format!("{}+${}", ticker, &diffstr);
                    { cmd.lock().unwrap().msg = order.to_string(); }
                    glogd!(" do_trade_sell =>", do_trade_buy(cmd).await);
                }
            }
        }
    }
    Ok("COMPLETED.")
}

async fn do_all (env: Env, body: &web::Bytes) -> Bresult<()> {
    let cmd :Cmd = CmdStruct::parse_cmd(env, body)?;
    info!("\x1b[33m{:?}", &cmd);
    glogd!("do_echo =>",       do_echo(&cmd).await);
    glogd!("do_help =>",       do_help(&cmd).await);
    glogd!("do_curse =>",      do_curse(&cmd).await);
    glogd!("do_repeat =>",     do_repeat(&cmd).await);
    glogd!("do_like =>",       do_like(&cmd).await);
    glogd!("do_like_info =>",  do_like_info(&cmd).await);
    glogd!("do_def =>",        do_def(&cmd).await);
    glogd!("do_sql =>",        do_sql(&cmd).await);
    glogd!("do_quotes => ",    do_quotes(&cmd).await);
    glogd!("do_portfolio =>",  do_portfolio(&cmd).await);
    glogd!("do_yolo =>",       do_yolo(&cmd).await);
    glogd!("do_trade_buy =>",  do_trade_buy(&cmd).await);
    glogd!("do_trade_sell =>", do_trade_sell(&cmd).await);
    glogd!("do_exchange_bidask =>", do_exchange_bidask(&cmd).await);
    glogd!("do_orders =>",     do_orders(&cmd).await);
    glogd!("do_fmt =>",        do_fmt(&cmd).await);
    glogd!("do_rebalance =>",  do_rebalance(&cmd).await);
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////

fn do_schema() -> Bresult<()> {
    let sql = ::sqlite::open("tmbot.sqlite")?;

    sql.execute("
        CREATE TABLE entitys (
            id INTEGER NOT NULL UNIQUE,
            name  TEXT NOT NULL);
    ").map_or_else(gwarn, ginfo);

    sql.execute("
        CREATE TABLE modes (
            id   INTEGER NOT NULL UNIQUE,
            echo INTEGER NOT NULL);
    ").map_or_else(gwarn, ginfo);

    sql.execute("
        CREATE TABLE formats (
            id    INTEGER NOT NULL UNIQUE,
            quote    TEXT NOT NULL,
            position TEXT NOT NULL);
    ").map_or_else(gwarn, ginfo);

    for l in read_to_string("tmbot/users.txt").unwrap().lines() {
        let mut v = l.split(" ");
        let id = v.next().ok_or("User DB malformed.")?;
        let name = v.next().ok_or("User DB malformed.")?.to_string();
        sql.execute( format!("INSERT INTO entitys VALUES ( {}, {:?} )", id, name)).map_or_else(gwarn, ginfo);
        if id.parse::<i64>().unwrap() < 0 {
            sql.execute( format!("INSERT INTO modes VALUES ({}, 2)", id) ).map_or_else(gwarn, ginfo);
        }
    }

    sql.execute("
        CREATE TABLE accounts (
            id    INTEGER  NOT NULL UNIQUE,
            balance FLOAT  NOT NULL);
    ").map_or_else(gwarn, ginfo);

    sql.execute("
        --DROP TABLE stonks;
        CREATE TABLE stonks (
            ticker   TEXT NOT NULL UNIQUE,
            price   FLOAT NOT NULL,
            last    FLOAT NOT NULL,
            market   TEXT NOT NULL,
            hours INTEGER NOT NULL,
            exchange TEXT NOT NULL,
            time  INTEGER NOT NULL,
            title    TEXT NOT NULL);
    ").map_or_else(gwarn, ginfo);
    //sql.execute("INSERT INTO stonks VALUES ( 'TWNK', 14.97, 1613630678, '14.97')").map_or_else(gwarn, ginfo);
    //sql.execute("INSERT INTO stonks VALUES ( 'GOOG', 2128.31, 1613630678, '2,128.31')").map_or_else(gwarn, ginfo);
    //sql.execute("UPDATE stonks SET time=1613630678 WHERE ticker='TWNK'").map_or_else(gwarn, ginfo);
    //sql.execute("UPDATE stonks SET time=1613630678 WHERE ticker='GOOG'").map_or_else(gwarn, ginfo);

    sql.execute("
        --DROP TABLE orders;
        CREATE TABLE orders (
            id   INTEGER  NOT NULL,
            ticker  TEXT  NOT NULL,
            qty    FLOAT  NOT NULL,
            price  FLOAT  NOT NULL,
            time INTEGER  NOT NULL);
    ").map_or_else(gwarn, ginfo);
    //sql.execute("INSERT INTO orders VALUES ( 241726795, 'TWNK', 500, 14.95, 1613544000 )").map_or_else(gwarn, ginfo);
    //sql.execute("INSERT INTO orders VALUES ( 241726795, 'GOOG', 0.25, 2121.90, 1613544278 )").map_or_else(gwarn, ginfo);

    sql.execute("
        CREATE TABLE positions (
            id   INTEGER NOT NULL,
            ticker  TEXT NOT NULL,
            qty    FLOAT NOT NULL,
            price  FLOAT NOT NULL);
    ").map_or_else(gwarn, ginfo);

    sql.execute("
        CREATE TABLE exchange (
            id   INTEGER NOT NULL,
            ticker  TEXT NOT NULL,
            qty    FLOAT NOT NULL,
            price  FLOAT NOT NULL,
            time INTEGER NOT NULL);
    ").map_or_else(gwarn, ginfo);

    Ok(())
}


fn log_header () {
    info!("\x1b[0;31;40m _____ __  __ ____        _   ");
    info!("\x1b[0;33;40m|_   _|  \\/  | __ )  ___ | |_™ ");
    info!("\x1b[0;32;40m  | | | |\\/| |  _ \\ / _ \\| __|");
    info!("\x1b[0;34;40m  | | | |  | | |_) | (_) | |_ ");
    info!("\x1b[0;35;40m  |_| |_|  |_|____/ \\___/ \\__|");
}

////////////////////////////////////////////////////////////////////////////////

async fn main_dispatch (req: HttpRequest, body: web::Bytes) -> HttpResponse {
    log_header();

    let env :Env = req.app_data::<web::Data<Env>>().unwrap().get_ref().clone();

    info!("\x1b[1m{:?}", env);
    info!("\x1b[35m{}", format!("{:?}", &req.connection_info()).replace(": ", ":").replace("\"", "").replace(",", ""));
    info!("\x1b[35m{}", format!("{:?}", &req).replace("\n", "").replace(r#"": ""#, ":").replace("\"", "").replace("   ", ""));
    info!("body:\x1b[35m{}", from_utf8(&body).unwrap_or(&format!("{:?}", body)));

    std::thread::spawn( move || {
        let env2 = env.clone();
        actix_web::rt::System::new("tmbot").block_on(async move {
            do_all(env2.clone(), &body).await.unwrap_or_else(|r| error!("{:?}", r));
        });

        info!("{:#?}", env);
        info!("End.");
    });

    HttpResponse::from("")
}

pub fn launch() -> Bresult<()> {
    let mut ssl_acceptor_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    ssl_acceptor_builder.set_private_key_file("key.pem", SslFiletype::PEM)?;
    ssl_acceptor_builder.set_certificate_chain_file("cert.pem")?;

    let botkey           = env::args().nth(1).ok_or("args[1] missing")?;
    let chat_id_default  = env::args().nth(2).ok_or("args[2] missing")?.parse::<i64>()?;
    let dst_hours_adjust = env::args().nth(3).ok_or("args[3] missing")?.parse::<i8>()?;

    if !true { do_schema()? } // Create DB

    // Hacks and other test code
    if !true { return fun().map_err( |e| {
        info!("{:?}", e);
        e
    } ) }

    let env :Env =  // Shared by requests handler threads.
        Arc::new(Mutex::new(EnvStruct{
            url_api:             format!("https://api.telegram.org/bot{}", &botkey),
            sqlite_filename:     format!("tmbot.sqlite"),
            chat_id_default:     chat_id_default,
            dst_hours_adjust:    dst_hours_adjust,
            quote_delay_secs:    QUOTE_DELAY_SECS,
            message_id_read:     0, // TODO edited_message mechanism is broken especially when /echo=1
            message_id_write:    0,
            message_buff_write:  String::new(),
            fmt_quote:           format!("%q%A%B %C%% %D%E@%F %G%H%I%q"),
            fmt_position:        format!("%n%q%A %C%B %D%%%q %q%E%F@%G%q %q%H@%I%q"),
        }));

    let srv =
        HttpServer::new(
            move || {
            App::new()
            .data(env.clone())
            .service(
                web::resource("*")
                .route(
                    Route::new()
                    .to(main_dispatch) ) ) } )
        .bind_openssl("0.0.0.0:8443", ssl_acceptor_builder)?;
        
    Ok(actix_web::rt::System::new("tmbot").block_on(async move {
        println!("launch() => {:?}", srv.run().await)
    }))
} // launch

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Hello)]
struct Greetings {
    z:i32,
    err:Box<dyn Error>
}

fn _fun_macros() -> Bresult<()> {
    let g = Greetings{z:42, err:"".into() };
    //Err("oh noe".into())
    //return Err(std::fmt::Error{});
    Err(std::io::Error::from_raw_os_error(0))?;
    Ok(g.hello())
}

fn fun () -> Bresult<()> {
    let conn = Connection::new(&"tmbot.sqlite")?;
    glog!(getsql!(&conn, "BEGIN TRANSACTION; INSERT INTO entitys VALUES (69, 'xxx')"));
    glog!(getsql!(&conn, "BEGIN TRANSACTION; BEGIN TRANSACTION; INSERT INTO entitys VALUES (69, 'xxx'); COMMIT;  ROLLBACK;"));
    glog!(getsql!(&conn, "ROLLBACK"));
    Ok(())
}
   
/*
  DROP   TABLE users
  CREATE TABLE users (name TEXT, age INTEGER)
  INSERT INTO  users VALUES ('Alice', 42)
  DELETE FROM  users WHERE age=42 AND name="Oskafunoioi"
  UPDATE       ukjsers SET a=1, b=2  WHERE c=3
*/