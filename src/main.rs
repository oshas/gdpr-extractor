#![feature(await_macro, async_await, futures_api)]

use fantoccini::{Client, Locator};
use futures::future::Future;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tokio;
use tokio::await;
use url::Url;

async fn download_data(user: String, password: String) -> Result<(), fantoccini::error::CmdError> {
    let caps = if let Value::Object(m) = json!({
        "moz:firefoxOptions": {
            "prefs": {
                "dom.webnotifications.enabled": false,
                "browser.helperApps.neverAsk.saveToDisk": "application/zip"
            }
        }
    }) {
        m
    } else {
        unreachable!()
    };

    println!("Starting WebDriver!");
    let client = await!(Client::with_capabilities("http://localhost:4444", caps))
        .expect("Failed to connect to WebDriver");
    let client = await!(client
        .goto("https://www.facebook.com/")
        // Login using the login form
        .and_then(|mut c| c.form(Locator::Id("login_form")))
        .and_then(|mut form| form.set_by_name("email", &user))
        .and_then(|mut form| form.set_by_name("pass", &password))
        .and_then(|form| form.submit()))?;
    // Do some other stuffs
    let client = await!(client
        .goto("https://www.facebook.com/dyi/?x=AdkmaGPgfNlhf-aR")
        // Change format to JSON
        .and_then(|mut c| c.find(Locator::LinkText("HTML")))
        .and_then(|e| e.click())
        .and_then(|c| c.wait_for_find(Locator::LinkText("JSON")))
        .and_then(|e| e.click())
        // Click button to create file
        .and_then(|mut c| c.find(Locator::Css("button[data-testid='dyi/sections/create']")))
        .and_then(|e| e.click()))?;

    let archives_url =
        Url::parse("https://www.facebook.com/dyi/?x=AdkmaGPgfNlhf-aR&tab=all_archives").unwrap();
    let mut client = Some(client);
    let client = loop {
        await!(tokio::timer::Delay::new(
            Instant::now() + Duration::from_secs(30)
        ))
        .unwrap();
        let c = client.take().unwrap();
        let mut c = await!(c.goto(archives_url.as_str()))?;
        if let Ok(e) = await!(c.find(Locator::Css(
            "button[data-testid='dyi/archives/download/0']",
        ))) {
            break await!(e.click())?;
        }
        client = Some(c);
    };

    let mut e = await!(client.wait_for_find(Locator::Id("ajax_password")))?;
    await!(e.send_keys(&password))?;
    await!(await!(e
        .client()
        .find(Locator::Css("button[data-testid='sec_ac_button']")))?
    .click())?;

    Ok(())
}

fn get_credentials() -> Result<(String, String), std::env::VarError> {
    use std::env;
    let user = env::var("USER")?;
    let password = env::var("PASSWORD")?;
    Ok((user, password))
}

fn main() {
    let (user, password) = get_credentials().expect("Failed to parse credentials");
    tokio::run_async(async {
        await!(download_data(user, password)).expect("A WebDriver command failed");
    });
}
