//! Main entry

use wnrake::{
    client::Client,
    parser::*,
    proxy::{Api, Proxy},
};

async fn solver() {
    // Configure proxy
    let proxy = Proxy::with_api("http://localhost:9000", Api::new("http://localhost:8000"));
    println!("{:?}", proxy.status().await);

    // Configure client
    let mut client = Client::with_proxy("http://localhost:8191/v1", &proxy);
    client.create_session().await.unwrap();
    println!("{}", client);

    // Get URL
    //let url = "https://ranobes.top/novels/1206822-diary-of-a-dead-wizard.html";
    let url = "https://ranobes.top/diary-of-a-dead-wizard-1206822/2846931.html";
    let parser = WnParser::try_from(url).unwrap();
    //match parser.get_book_info(&mut client, url).await {
    match parser.get_chapter(&mut client, url).await {
        Ok(res) => {
            match parser.parse_chapter(&res) {
                Ok(chapter) => println!("{}", chapter.html),
                Err(e) => eprintln!("{:?}", e),
            }
            //match RanobesParser::get_chapterlist(&mut client, &res).await {
            //    Ok(chapters) => println!("{:?}", chapters),
            //    Err(e) => eprintln!("{:?}", e),
            //}
        }
        Err(e) => eprintln!("{:?}", e),
    }

    // Cleanup client
    client.destroy_session().await.unwrap();
}

#[tokio::main]
async fn dispatcher() -> i32 {
    solver().await;
    return 0;
}

fn main() {
    std::process::exit(dispatcher())
}
