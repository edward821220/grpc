use tonic::transport::Channel;
use tonic::Request;

pub mod greet {
    tonic::include_proto!("greet");
}

use greet::greet_service_client::GreetServiceClient;
use greet::GreetRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://[::1]:50051").connect().await?;

    let mut greet_client = GreetServiceClient::new(channel.clone());

    // Unary
    let greet_request = Request::new(GreetRequest {
        first_name: "Rust".into(),
    });

    let greet_response = greet_client.greet(greet_request).await?;
    println!("Greet Response: {:?}", greet_response.into_inner().result);

    // Server streaming
    let greet_request_2 = Request::new(GreetRequest {
        first_name: "Rust2".into(),
    });

    let mut stream = greet_client
        .greet_many_res(greet_request_2)
        .await?
        .into_inner();

    while let Some(response) = stream.message().await? {
        println!("Greet Response: {:?}", response.result);
    }

    // Client streaming
    let (tx, rx) = tokio::sync::mpsc::channel::<GreetRequest>(4);
    let outbound = tokio_stream::wrappers::ReceiverStream::new(rx);

    let response_future = greet_client.long_greet(outbound);

    let names = vec!["Alice", "Bob", "Charlie", "David"];
    for name in names {
        tx.send(GreetRequest {
            first_name: name.into(),
        })
        .await
        .unwrap();
    }

    drop(tx);

    let response = response_future.await?;

    println!("LongGreet Response: {:?}", response.into_inner().result);

    // Bi-directional
    let (tx, rx) = tokio::sync::mpsc::channel::<GreetRequest>(4);
    let outbound = tokio_stream::wrappers::ReceiverStream::new(rx);
    let response_future = greet_client.greet_everyone(outbound);

    let names = vec!["Alice", "Bob", "Charlie", "David"];
    for name in names {
        tx.send(GreetRequest {
            first_name: name.into(),
        })
        .await
        .unwrap();
    }
    drop(tx);

    let mut stream = response_future.await?.into_inner();

    while let Some(response) = stream.message().await? {
        println!("Greet Response: {:?}", response.result);
    }

    Ok(())
}
