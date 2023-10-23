use anyhow::Result;
use chrono::Utc;
use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions, QueueDeleteOptions},
    protocol::basic::AMQPProperties,
    types::{AMQPValue, FieldTable, ShortString},
    Connection, ConnectionProperties,
};

#[tokio::main]
async fn main() -> Result<()> {
    let queue_name = "demo";
    let connection_string = "amqp://guest:guest@127.0.0.1:5672".to_string();
    let connection =
        Connection::connect(&connection_string, ConnectionProperties::default()).await?;

    let channel = connection.create_channel().await?;

    let _ = channel
        .queue_delete(queue_name, QueueDeleteOptions::default())
        .await;

    let mut queue_args = FieldTable::default();
    queue_args.insert(
        ShortString::from("x-queue-type"),
        AMQPValue::LongString("stream".into()),
    );

    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable: true,
                auto_delete: false,
                ..Default::default()
            },
            queue_args,
        )
        .await?;

    loop {
        let data = lipsum::lipsum(20);
        let data = data.as_bytes();
        let uuid = uuid::Uuid::new_v4();
        let timestamp = Utc::now().timestamp_millis() as u64;
        let transaction_id = format!("transaction_{}", uuid);
        let mut headers = FieldTable::default();
        headers.insert(
            ShortString::from("x-stream-transaction-id"),
            AMQPValue::LongString(transaction_id.clone().into()),
        );

        channel
            .basic_publish(
                "",
                queue_name,
                BasicPublishOptions::default(),
                data,
                AMQPProperties::default()
                    .with_headers(headers.clone())
                    .with_timestamp(timestamp),
            )
            .await?;
        println!(
            "Published message with transaction id: {}, timestamp: {} and data: {}",
            transaction_id.clone(),
            timestamp,
            String::from_utf8_lossy(data)
        );
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}