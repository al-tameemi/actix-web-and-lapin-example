use actix::dev::channel;
use lapin::{
    options::*,
    types::FieldTable,
    Connection,
    ConnectionProperties, 
    Channel, 
    Queue,
};
use log::info;
use std::{sync::{Arc, Mutex}, collections::HashMap, error::Error};
use futures_lite::stream::StreamExt;

pub struct Source {
    data: Arc<Mutex<HashMap<String, String>>>,
    conn: Connection,
    channel: Option<Channel>,
}

impl Source {
    pub async fn new(data: Arc<Mutex<HashMap<String, String>>>) -> Result<Source, lapin::Error> {
        let address = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
        let conn = Connection::connect(&address, ConnectionProperties::default())
            .await
            .expect("Connection Error");

         info!("{:?}", conn.status().state());

        Ok(Source{
            data,
            conn,
            channel: None,
        })
    }

    pub async fn add_queue(&mut self, routing_key: &str, queue_name: &str, exchange: &str) -> Result<(), lapin::Error> {
        self.channel = Some(self.conn.create_channel().await.expect("failed to create channel"));
        let queue = self.channel.as_ref().unwrap().queue_declare(queue_name, QueueDeclareOptions::default(), FieldTable::default());
        Ok(())
        // Ok(self.channel.as_ref().unwrap().queue_bind(queue_name, exchange, 
        //     routing_key, QueueBindOptions::default(), FieldTable::default()).await?)
    }

    pub async fn get_data(self) {
        let mut consumer = self.channel.unwrap()
            .basic_consume(
                "hello",
                "my_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("basic_consume");

        while let Some(delivery) = consumer.next().await {
            if let Ok(delivery) = delivery {
                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("basic_ack");
            }
        }
    }
}