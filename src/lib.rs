use futures_util::{SinkExt, StreamExt};

use sea_orm::{ActiveModelTrait, Database, Set};
use serde::Deserialize;
use std::str;
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
mod database;

use database::trade;

#[derive(Deserialize, Debug)]
struct Data {
    p: f64,
    s: String,
    t: f64,
    v: f64,
}

#[derive(Deserialize, Debug)]
struct WSMessage {
    data: Vec<Data>,
    #[serde(rename = "type")]
    type_: String,
}

pub async fn run(
    connect_addr: String,
    database_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::connect(database_url).await?;
    let mut url = url::Url::parse(&connect_addr)?;

    let (ws_stream, _) = connect_async(&url).await?;
    let (mut write, read) = ws_stream.split();

    url.set_query(None);
    println!("Connected to {}", url);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            write
                .send(Message::Text(
                    r#"{"type":"subscribe","symbol":"BINANCE:BTCUSDT"}"#.to_owned(),
                ))
                .await
                .unwrap_or_default();
            write
                .send(Message::Text(
                    r#"{"type":"subscribe","symbol":"IC MARKETS:1"}"#.to_owned(),
                ))
                .await
                .unwrap_or_default();
        }
    });

    read.for_each(|message| async {
        let data = if let Ok(message) = message {
            message.into_data()
        } else if let Err(e) = message {
            tokio::io::stdout()
                .write_all(format!("Error while decoding message: {}", e).as_bytes())
                .await
                .unwrap_or_default();
            return;
        } else {
            return;
        };
        match str::from_utf8(&data) {
            Ok(wsmes) => {
                if let Ok(wsmes) = serde_json::from_str::<WSMessage>(wsmes) {
                    if wsmes.type_ != "trade" {
                        tokio::io::stdout()
                            .write_all(format!("Invalid Json type {}", wsmes.type_).as_bytes())
                            .await
                            .unwrap_or_default();
                        return;
                    }
                    for data in wsmes.data.iter() {
                        let post = trade::ActiveModel {
                            price: Set(data.p),
                            security: Set(data.s.clone()),
                            timestamp: Set(data.t / 1000.),
                            value: Set(data.v),
                            ..Default::default()
                        };
                        post.save(&db).await.unwrap_or_default();
                    }
                } else {
                    tokio::io::stdout()
                        .write_all(format!("Invalid UTF-8 sequence: {}", wsmes).as_bytes())
                        .await
                        .unwrap_or_default();
                }
            }
            Err(e) => {
                tokio::io::stdout()
                    .write_all(format!("Invalid UTF-8 sequence: {}", e).as_bytes())
                    .await
                    .unwrap_or_default();
            }
        };
    })
    .await;

    Ok(())
}
