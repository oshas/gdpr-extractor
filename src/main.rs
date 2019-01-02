use fantoccini::{Client, Locator};
use futures::future::Future;
use tokio;

fn get_credentials() -> Result<(String, String), std::env::VarError> {
    use std::env;
    let user = env::var("USER")?;
    let password = env::var("PASSWORD")?;
    Ok((user, password))
}

fn main() {
    let (user, password) = get_credentials().expect("Failed to parse credentials");
    let client = Client::new("http://localhost:4444");

    // Set up a sequence of steps we want the browser to take
    tokio::run(
        client
            .map_err(|e| panic!("failed to connect to WebDriver: {:?}", e))
            .and_then(|c| c.goto("https://www.facebook.com/"))
            // Login using the login form
            .and_then(|mut c| c.form(Locator::Id("login_form")))
            .and_then(move |mut form| form.set_by_name("email", &user))
            .and_then(move |mut form| form.set_by_name("pass", &password))
            .and_then(|form| form.submit())
            // Do some other stuffs
            .and_then(|mut c| c.current_url())
            .and_then(|url| {
                println!("The url is now: {}", url);
                Ok(())
            })
            .map_err(|e| {
                panic!("A WebDriver command failed: {:?}", e);
            }),
    )
}
