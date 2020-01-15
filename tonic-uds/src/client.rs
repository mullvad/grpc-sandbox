#![cfg_attr(not(unix), allow(unused_imports))]

use std::convert::TryFrom;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;
use parity_tokio_ipc::Endpoint as IpcEndpoint;

pub mod pb {
    tonic::include_proto!("/grpc.examples.echo");
}
use pb::{echo_client::EchoClient, EchoRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // We will ignore this uri because uds do not use it
    // if your connector does use the uri it will be provided
    // as the request to the `MakeConnection`.
    let channel = Endpoint::try_from("lttp://[::]:50051")?
        .connect_with_connector(service_fn(|_: Uri| {
            let path = if cfg!(windows) {
                format!("//./pipe/helloworld")
            } else {
                format!("/tmp/helloworld")
            };

            IpcEndpoint::connect(path)
        }))
        .await?;

    let mut client = EchoClient::new(channel);

    let request = tonic::Request::new(EchoRequest {
        message: "hello".into(),
    });

    let response = client.unary_echo(request).await?;

    println!("RESPONSE={:?}", response);

    let request2 = tonic::Request::new(EchoRequest {
        message: "HELLO2".into(),
    });

    let mut stream = client.server_streaming_echo(request2).await?.into_inner();

    while let Some(message) = stream.message().await? {
        println!("NOTE = {:?}", message);
    }

    Ok(())
}
