use std::time::Duration;
use tower::{Service, ServiceExt};

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
        .concurrency_limit(10)
        .buffer(100)
        .rate_limit(10, Duration::new(10, 0))
        .service_fn(move |req| client.execute(req));

    let req = reqwest::Request::new(reqwest::Method::GET, reqwest::Url::parse("https://www.walmart.com/store/electrode/api/store-directory?st=CA").unwrap());

    let res = service.ready().await.unwrap().call(req).await.unwrap();

    println!("res: {:?}", res.text().await.unwrap());
}
