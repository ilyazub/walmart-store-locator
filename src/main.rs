use futures::stream::{self, StreamExt};
use scraper::{Html, Selector as ScraperSelector};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use tower::{Service, ServiceExt};

// TODO: Get all states from https://www.walmart.com/store/directory
const STATES: [&str; 51] = [
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
    "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY",
    "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "DC",
    "WV", "WI", "WY",
];

const OUTPUT_FILE_NAME: &str = "output/walmart_stores.csv";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct City {
    city: String,

    store_id: Option<usize>,
    store_count: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InnerCity {
    state: String,
    city: String,

    store_id: Option<usize>,
    store_count: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Store {
    store_id: usize,
    postal_code: String,
    address: String,
}

type StoreLocatorError = Box<dyn std::error::Error>;

#[tokio::main(worker_threads = 10)]
async fn main() -> Result<(), StoreLocatorError> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::ACCEPT, "application/json".parse()?);

    let client = reqwest::Client::builder()
        .user_agent("Opera/9.80 (Windows NT 6.1; WOW64) Presto/2.12.388 Version/12.18")
        .default_headers(headers)
        .build()?;

    // let mut service = tower::ServiceBuilder::new()
    //     .concurrency_limit(200)
    //     // .buffer(200)
    //     .rate_limit(50, Duration::new(5, 0))
    //     .service_fn(move |req| client.execute(req));

    // let mut walmart_addresses = Vec::<Store>::new();

    let walmart_addresses: Vec<Vec<Option<Store>>> = tokio_stream::iter(STATES)
        .map(|state| {
            let client = &client;
            async move {
                let state_res = client.get(
                    reqwest::Url::parse_with_params(
                        "https://www.walmart.com/store/electrode/api/store-directory",
                        &[("st", state)],
                    )
                    .unwrap(),
                );

                let state_text = state_res.send().await.unwrap().text().await.unwrap();

                let cities: Vec<City> = serde_json::from_str(&state_text).unwrap();
                let inner_cities: Vec<InnerCity> = cities
                    .iter()
                    .map(|city| InnerCity {
                        state: state.to_string(),
                        city: city.city.clone(),
                        store_id: city.store_id,
                        store_count: city.store_count,
                    })
                    .collect();

                println!("State: {state}", state = state);
                println!("State JSON: {:?}", cities);

                inner_cities
            }
        })
        .buffer_unordered(10)
        .flat_map(tokio_stream::iter)
        .map(|city| {
            let client = &client;
            async move {
                println!(
                    "Downloading Walmart stores of a city: {city:?}...",
                    city = city
                );

                if let (Some(store_id), None) = (city.store_id, city.store_count) {
                    let store_request = client.get(
                        reqwest::Url::parse(&format!(
                            "https://www.walmart.com/store/{store_id}",
                            store_id = store_id
                        ))
                        .unwrap(),
                    );

                    let store_text = store_request.send().await.unwrap().text().await.unwrap();

                    let store_html = Html::parse_document(&store_text);

                    let sel_store_zip_code = css(".store-address-postal[itemprop=postalCode]");
                    let sel_store_address = css(".store-address[itemprop=address]");

                    if let (Some(store_zip_code_ref), Some(store_address_ref)) = (
                        store_html.select(&sel_store_zip_code).next(),
                        store_html.select(&sel_store_address).next(),
                    ) {
                        let store = Store {
                            store_id,
                            postal_code: store_zip_code_ref.text().collect(),
                            address: store_address_ref.text().collect(),
                        };

                        println!("Found store: {store:?}", store = store);

                        vec![Some(store)]
                    } else {
                        vec![None]
                    }
                } else {
                    let city_request = client.get(
                        reqwest::Url::parse_with_params(
                            "https://www.walmart.com/store/electrode/api/store-directory",
                            &[("st", city.state), ("city", city.city)],
                        )
                        .unwrap(),
                    );

                    let city_text = city_request.send().await.unwrap().text().await.unwrap();

                    let stores: Vec<Store> = serde_json::from_str(&city_text).unwrap();

                    stores
                        .into_iter()
                        .map(|store| {
                            println!("Found store: {store:?}", store = store);

                            Some(store)
                        })
                        .collect::<Vec<Option<Store>>>()
                }
            }
        })
        .buffer_unordered(10)
        .collect::<Vec<_>>()
        .await;

    let walmart_addresses: Vec<Store> = walmart_addresses
        .into_iter()
        .flatten()
        .filter_map(|s| s)
        .collect();

    write_results_to_file(&walmart_addresses)?;

    Ok(())
}

fn css(selector: &str) -> ScraperSelector {
    ScraperSelector::parse(selector).expect("this should never trigger")
}

fn write_results_to_file(records: &[Store]) -> Result<(), StoreLocatorError> {
    let mut writer = csv::Writer::from_path(OUTPUT_FILE_NAME)?;

    for record in records {
        writer.serialize(record)?;
    }

    writer.flush()?;

    println!(
        "Successfully wrote {number_of_records} to {file_name}",
        number_of_records = records.len(),
        file_name = OUTPUT_FILE_NAME
    );

    Ok(())
}
