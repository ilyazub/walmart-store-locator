# walmart-store-locator

Download a list of Walmart Stores.

The latest data is in [`output/walmart_stores.csv`](https://raw.githubusercontent.com/ilyazub/walmart-store-locator/master/output/walmart_stores.csv).

## Sample data

| store_id|postal_code|address|
|-|-|-|
|1158|35214|"2473 Hackworth Rd, Adamsville, AL 35214"
|4756|35007|9085 Hwy 119
|423|35007|630 Colonial Promenade Pkwy
|726|35010|"2643 Highway 280, Alexander City, AL 35010"

## Sponsorship

Sponsored by [SerpApi](https://serpapi.com)

<a href="https://serpapi.com">
  <img src="https://user-images.githubusercontent.com/282605/142473823-98830f92-b5e9-4d05-8fd1-27114da4a478.png"
       alt="Sponsored by SerpApi" width="128" height="128">
</a>

## Usage

This project requires Rust and Cargo to be installed. 

### Download this repository

```bash
git clone git@github.com:ilyazub/walmart-store-locator.git
```

### Execute using Cargo

```bash
cargo run
```

For better reliability use proxies. This project uses `reqwest` crate which automatically uses `HTTP_PROXY` environment variable.

```bash
HTTP_PROXY="..." cargo run
```

Walmart Stores will be written to [`output/walmart_stores.csv`](https://raw.githubusercontent.com/ilyazub/walmart-store-locator/master/output/walmart_stores.csv).

## TODO

- [ ] Speed up requests
- [ ] Retry on HTTP errors
- [ ] Add progress bar
- [ ] Refactor to idiomatic Rust
