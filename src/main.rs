use scraper::{Html, Selector as ScraperSelector};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tower::{Service, ServiceExt};

// TODO: Get all states from https://www.walmart.com/store/directory
const STATES: [&str; 51] = [
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
    "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY",
    "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "DC",
    "WV", "WI", "WY",
];

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct City<'a> {
    city: &'a str,

    store_id: Option<usize>,
    store_count: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Store {
    store_id: usize,
    postal_code: String,
    address: String,
}

#[tokio::main]
async fn main() {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::ACCEPT, "application/json".parse().unwrap());

    let client = reqwest::Client::builder()
        .user_agent("Opera/9.80 (Windows NT 6.1; WOW64) Presto/2.12.388 Version/12.18")
        .default_headers(headers)
        .build()
        .unwrap();

    let mut service = tower::ServiceBuilder::new()
        .concurrency_limit(50)
        .buffer(200)
        .rate_limit(25, Duration::new(5, 0))
        .service_fn(move |req| client.execute(req));

    let mut walmart_addresses_queue = Vec::<Store>::new();

    for state in STATES {
        let state_req = reqwest::Request::new(
            reqwest::Method::GET,
            reqwest::Url::parse_with_params(
                "https://www.walmart.com/store/electrode/api/store-directory",
                &[("st", state)],
            )
            .unwrap(),
        );

        let state_res = service
            .ready()
            .await
            .unwrap()
            .call(state_req)
            .await
            .unwrap();

        let state_text = state_res.text().await.unwrap();

        let cities: Vec<City> = serde_json::from_str(&state_text).unwrap();

        println!("State JSON: {:?}", cities);

        for city in cities {
            println!(
                "Downloading Walmart stores of a city: {city:?}...",
                city = city
            );

            if city.store_id.is_some() && city.store_count.is_none() {
                let store_id = city.store_id.unwrap();

                let store_request = reqwest::Request::new(
                    reqwest::Method::GET,
                    reqwest::Url::parse(&format!(
                        "https://www.walmart.com/store/{store_id}",
                        store_id = store_id
                    ))
                    .unwrap(),
                );

                let store_res = service
                    .ready()
                    .await
                    .unwrap()
                    .call(store_request)
                    .await
                    .unwrap();

                let store_text = store_res.text().await.unwrap();

                let store_html = Html::parse_document(&store_text);

                let sel_store_zip_code = css(".store-address-postal[itemprop=postalCode]");
                let sel_store_address = css(".store-address[itemprop=address]");

                if let (Some(store_zip_code_ref), Some(store_address_ref)) = (
                    store_html.select(&sel_store_zip_code).next(),
                    store_html.select(&sel_store_address).next(),
                ) {
                    let store = Store {
                        store_id: store_id,
                        postal_code: store_zip_code_ref.text().collect(),
                        address: store_address_ref.text().collect(),
                    };

                    println!("Found store: {store:?}", store = store);

                    walmart_addresses_queue.push(store);
                }
            } else {
                let city_request = reqwest::Request::new(
                    reqwest::Method::GET,
                    reqwest::Url::parse_with_params(
                        "https://www.walmart.com/store/electrode/api/store-directory",
                        &[("st", state), ("city", city.city)],
                    )
                    .unwrap(),
                );

                let city_res = service
                    .ready()
                    .await
                    .unwrap()
                    .call(city_request)
                    .await
                    .unwrap();

                let city_text = city_res.text().await.unwrap();

                let stores: Vec<Store> = serde_json::from_str(&city_text).unwrap();

                for store in stores {
                    println!("Found store: {store:?}", store = store);

                    walmart_addresses_queue.push(store);
                }
            }
        }
    }

    write_results_to_file(&walmart_addresses_queue);
}

fn css(selector: &str) -> ScraperSelector {
    ScraperSelector::parse(selector).expect("this should never trigger")
}

fn write_results_to_file(records: &[Store]) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = "/tmp/walmart_stores.csv";
    let mut writer = csv::Writer::from_path(file_name)?;

    for record in records {
        // When writing records with Serde using structs, the header row is written automatically.
        writer.serialize(record)?;
    }

    writer.flush()?;

    println!(
        "Successfully wrote {number_of_records} to {file_name}",
        number_of_records = records.len(),
        file_name = file_name
    );

    Ok(())
}
