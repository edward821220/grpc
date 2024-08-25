use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

pub mod greet {
    tonic::include_proto!("greet");
}

use greet::greet_service_server::{GreetService, GreetServiceServer};
use greet::{GreetRequest, GreetResponse};

#[derive(Default, Clone)]
pub struct MyService;

#[tonic::async_trait]
impl GreetService for MyService {
    async fn greet(
        &self,
        request: Request<GreetRequest>,
    ) -> Result<Response<GreetResponse>, Status> {
        let reply = greet::GreetResponse {
            result: format!("Hello, {}!", request.into_inner().first_name),
        };

        Ok(Response::new(reply))
    }

    type greetManyResStream = ReceiverStream<Result<GreetResponse, Status>>;

    async fn greet_many_res(
        &self,
        request: Request<GreetRequest>,
    ) -> Result<Response<Self::greetManyResStream>, Status> {
        let name = request.into_inner().first_name;
        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            for i in 0..5 {
                let reply = greet::GreetResponse {
                    result: format!("Hello, {} ({})", name, i),
                };
                tx.send(Ok(reply)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
    async fn long_greet(
        &self,
        request: Request<tonic::Streaming<GreetRequest>>,
    ) -> Result<Response<GreetResponse>, Status> {
        let mut stream = request.into_inner();
        let mut result = String::new();
        while let Some(request) = stream.message().await? {
            result += format!(" {}", &request.first_name).as_str();
        }

        Ok(Response::new(greet::GreetResponse { result }))
    }

    type greetEveryoneStream = ReceiverStream<Result<GreetResponse, Status>>;

    async fn greet_everyone(
        &self,
        request: Request<tonic::Streaming<GreetRequest>>,
    ) -> Result<Response<Self::greetEveryoneStream>, Status> {
        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            let mut stream = request.into_inner();
            while let Some(request) = stream.message().await.unwrap() {
                let reply = greet::GreetResponse {
                    result: format!("Hello, {}!", request.first_name),
                };
                tx.send(Ok(reply)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let service = MyService;

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(GreetServiceServer::new(service.clone()))
        .serve(addr)
        .await?;

    Ok(())
}
