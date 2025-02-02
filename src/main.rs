/*
 * Author: P K Navin Shrinivas 
 * Email: karupal2002@gmail.com 
 * Website: https://navinshrinivas.com
 *
 * This code is entirely contained in a single file.
 * This script refreshes data from JIO api every 5 minutes, parses it 
 * sorts it and presents it in a HTML page.
 *
 * I have no frontend for this, this backed itself served the frontend as a simple HTML.
 */


use chrono;
use log::info;
use reqwest;
use simple_logger::SimpleLogger;
use std::sync::{mpsc::channel, Arc, Mutex};
use timer;
use tokio::io;
use warp;
use warp::Filter;
mod content;

#[derive(PartialEq, Eq)]
struct PlanInfo {
    name: String,               //Done
    price: String,              //Done
    price_per_day: String,      //Done
    price_per_gb: String,       //Done
    data_at_high_speed: String, //Done
    validity: String,           //Done
    description: String,        //Done
    call_available: bool,       //Done
    category: String,           //Done
    total_data: String,
}
impl std::fmt::Display for PlanInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Category: {} | Name: {} | Price: {} | Price per day: {} | Price per GB: {} | Data at high speed: {} | Total Data : {} | Validity: {} | Description: {:.45} | Call Available: {} <br/> <br/>", self.category, self.name, self.price, self.price_per_day, self.price_per_gb, self.data_at_high_speed, self.total_data, self.validity, self.description, self.call_available)
    }
}

impl std::fmt::Debug for PlanInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Category: {} | Name: {} | Price: {} | Price per day: {} | Price per GB: {} | Data at high speed: {} | Total Data : {} | Validity: {} | Description: {:.45} | Call Available: {} <br/> <br/>", self.category, self.name, self.price, self.price_per_day, self.price_per_gb, self.data_at_high_speed, self.total_data, self.validity, self.description, self.call_available)
    }
}

impl Default for PlanInfo {
    fn default() -> Self {
        return PlanInfo {
            name: String::new(),
            price: String::new(),
            price_per_day: String::new(),
            price_per_gb: String::new(),
            data_at_high_speed: String::new(),
            validity: String::new(),
            description: String::new(),
            call_available: false,
            category: String::new(),
            total_data: String::new(),
        };
    }
}

#[derive(Debug)]
enum CheapJioErr {
    ParseError,
    GetError,
    PlanCategoriesMissing,
    SubCategoriesMissing,
    PlanInfoMissing,
    CategoryMissing,
    DescriptionMissing,
}

impl Ord for PlanInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        //First sort by price_per_day then price_per_gb and then price and then by validity
        //then if cal is available
        //Else simply by name
        if self.call_available != other.call_available {
            //compare as bool, true is greater
            return self.call_available.cmp(&other.call_available).reverse();
        }
        if self.price_per_day != other.price_per_day {
            //compare as f64 values
            return self
                .price_per_day
                .parse::<f64>()
                .unwrap()
                .partial_cmp(&other.price_per_day.parse::<f64>().unwrap())
                .unwrap();
        }
        if self.price_per_gb != other.price_per_gb {
            //compare as f64 values
            return self
                .price_per_gb
                .parse::<f64>()
                .unwrap()
                .partial_cmp(&other.price_per_gb.parse::<f64>().unwrap())
                .unwrap();
        }
        if self.price != other.price {
            //compare as f64 values
            return self
                .price
                .parse::<f64>()
                .unwrap()
                .partial_cmp(&other.price.parse::<f64>().unwrap())
                .unwrap();
        }
        if self.validity != other.validity {
            //compare as i32
            return self
                .validity
                .parse::<i32>()
                .unwrap()
                .partial_cmp(&other.validity.parse::<i32>().unwrap())
                .unwrap();
        }

        return self.name.cmp(&other.name);
    }
}

impl PartialOrd for PlanInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

const JIO_API : &str = "https://www.jio.com/api/jio-mdmdata-service/mdmdata/recharge/plans?productType=MOBILITY&billingType=1";

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut logger = SimpleLogger::new();
    logger = logger.with_level(log::LevelFilter::Info); //Setting default
    logger = logger.env();
    logger.init().unwrap();
    //sleep for 5 seconds
    // tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    info!("Application Starting...");

    let (tx, rx) = channel();
    tx.send(0).unwrap(); //Initial load
    let timer = timer::MessageTimer::new(tx);
    let _g = timer.schedule_repeating(chrono::Duration::minutes(5), 0 as i32);
    // let parsed_data = refresh_jio_data().await; //Initial loading
    let parsed_data_ctx: Arc<Mutex<Result<Vec<PlanInfo>, CheapJioErr>>> =
        Arc::new(Mutex::new(Ok(Vec::new())));
    let routes = warp::path("plans")
        .and(with_parsed_data(parsed_data_ctx.clone()))
        .map(
            |parsed_data: Arc<Mutex<Result<Vec<PlanInfo>, CheapJioErr>>>| {
                let p_data = parsed_data.clone();
                let parsed_data_inner = p_data.lock().unwrap();
                let response = match parsed_data_inner.as_ref(){
                    Ok(plan_info_list) => plan_info_list.iter().fold(
                        String::new(),
                        |acc, plan_info| acc + &format!("{:#?}", plan_info),
                    ),
                    Err(e) => format!("Something is fishy, contact me through mailto:karupal2002@gmail.com, with the following : {:#?}", e),
                };
                let total_res = format!("{}{}{}", content::HEADER, response, content::FOOTER);
                return warp::reply::html(total_res);
            },
        );
    let warp_server = warp::serve(routes).run(([0,0,0,0], 3030));
    let fetch_job = tokio::spawn(async move {
        fetch_jio_plans_loop(parsed_data_ctx.clone(), rx).await;
    });
    let _ = tokio::join!(warp_server, fetch_job);

    Ok(())
}

async fn fetch_jio_plans_loop(
    parsed_data_ctx: Arc<Mutex<Result<Vec<PlanInfo>, CheapJioErr>>>,
    rx: std::sync::mpsc::Receiver<i32>,
) {
    loop {
        let _ = rx.recv();
        let mut parsed_data = refresh_jio_data().await;
        if parsed_data.is_ok() {
            info!("Data fetched and parsed successfully");
            let mut parsed_vec = parsed_data.unwrap();
            log::debug!("{:#?}", parsed_vec);
            parsed_vec.sort();
            parsed_data = Ok(parsed_vec);
        }
        let mut parsed_data_inner = parsed_data_ctx.lock().unwrap();
        *parsed_data_inner = parsed_data;
    }
}

async fn refresh_jio_data() -> Result<Vec<PlanInfo>, CheapJioErr> {
    info!("Refreshing JIO plans data...Happens every 5 minutes");
    let resp = match reqwest::get(JIO_API).await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(value) => value,
            Err(e) => {
                log::error!("{:#?}", e);
                return Err(CheapJioErr::ParseError); //Initial JSON parse error
            }
        },
        Err(e) => {
            log::error!("{:#?}", e);
            return Err(CheapJioErr::GetError);
        }
    };
    match parse_jio_data(resp) {
        Ok(plan_info_list) => {
            return Ok(plan_info_list);
        }
        Err(e) => {
            log::error!("{:#?}", e);
            return Err(e);
        }
    }
}

fn parse_jio_data(value: serde_json::Value) -> Result<Vec<PlanInfo>, CheapJioErr> {
    let plan_category = match value["planCategories"].as_array() {
        Some(arr) => arr,
        None => return Err(CheapJioErr::PlanCategoriesMissing),
    };
    let mut plan_info_list = Vec::new();
    for i in plan_category.iter() {
        let category = match i["type"].as_str() {
            Some(s) => s.to_string(),
            None => return Err(CheapJioErr::CategoryMissing),
        };
        let sub_category = match i["subCategories"].as_array() {
            Some(arr) => arr,
            None => return Err(CheapJioErr::SubCategoriesMissing),
        };
        for j in sub_category.iter() {
            let plans = match j["plans"].as_array() {
                Some(arr) => arr,
                None => return Err(CheapJioErr::PlanInfoMissing),
            };
            for k in plans.iter() {
                let mut plan_info = PlanInfo::default();
                plan_info.category = category.clone();
                let misc_details = match k["misc"]["details"].as_array() {
                    Some(arr) => arr,
                    None => return Err(CheapJioErr::ParseError),
                };
                let name = match k["planName"].as_str() {
                    Some(s) => s.to_string(),
                    None => return Err(CheapJioErr::ParseError),
                };
                if *name
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .get(0)
                    .unwrap()
                    == "JP"
                    || *name
                        .split_whitespace()
                        .collect::<Vec<&str>>()
                        .get(0)
                        .unwrap()
                        == "JB"
                {
                    //Avoiding JioPhone and JioBharat plans
                    continue;
                } else {
                    plan_info.name = name;
                }
                plan_info.price = match k["amount"].as_str() {
                    Some(s) => s.to_string(),
                    None => return Err(CheapJioErr::ParseError),
                };
                plan_info.validity = get_misc_details_from_data_map(
                    &misc_details,
                    "Pack validity".to_string(),
                    Some(true),
                );
                let total_data = get_misc_details_from_data_map(
                    &misc_details,
                    "Total data".to_string(),
                    Some(true),
                );
                //Some plans exist with 0 data, avoiding them too.
                if total_data == "".to_string() {
                    continue;
                } else {
                    plan_info.total_data = total_data.clone();
                    plan_info.price_per_gb = (plan_info.price.parse::<f64>().unwrap()
                        / total_data.parse::<f64>().unwrap())
                    .to_string();
                }
                plan_info.price_per_day = (plan_info.price.parse::<f64>().unwrap()
                    / plan_info.validity.parse::<f64>().unwrap())
                .to_string();
                plan_info.description = match k["description"].as_str() {
                    Some(s) => s.to_string(),
                    None => return Err(CheapJioErr::DescriptionMissing),
                };
                plan_info.data_at_high_speed = get_misc_details_from_data_map(
                    &misc_details,
                    "Data at high speed*".to_string(),
                    Some(false),
                );
                plan_info.call_available = match get_misc_details_from_data_map(
                    &misc_details,
                    "Voice".to_string(),
                    Some(false),
                )
                .as_str()
                {
                    "" => false,
                    _ => true,
                };

                plan_info_list.push(plan_info);
            }
        }
    }
    return Ok(plan_info_list);
}

fn get_misc_details_from_data_map(
    misc_details: &Vec<serde_json::Value>,
    key: String,
    unsplit: Option<bool>,
) -> String {
    for i in misc_details.iter() {
        if i["header"].as_str().unwrap() == key {
            let value = i["value"].as_str().unwrap().to_string();
            if unsplit.is_some_and(|x| x == true) {
                return value
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .get(0)
                    .unwrap()
                    .to_string();
            }
            return value;
        }
    }
    return String::new();
}

fn with_parsed_data(
    parsed_data: Arc<Mutex<Result<Vec<PlanInfo>, CheapJioErr>>>,
) -> impl Filter<
    Extract = (Arc<Mutex<Result<Vec<PlanInfo>, CheapJioErr>>>,),
    Error = std::convert::Infallible,
> + Clone {
    warp::any().map(move || parsed_data.clone())
}
