use collection::Collection;

mod collection;
mod net;
mod auth;

#[tokio::main]
async fn main() {
    // let mut col = Collection::new(None);

    // col.new_subdomain("discord".to_string(), "https://discord.gg/...".to_string());

    // col.save().unwrap();
    let app = net::app();

    net::serve(app).await;
}