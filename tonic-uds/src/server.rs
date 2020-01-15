#![cfg_attr(not(unix), allow(unused_imports))]

use futures::stream::TryStreamExt;
use futures::Stream;
use std::path::Path;
use std::pin::Pin;
use std::time::Duration;
#[cfg(unix)]
use tokio::net::UnixListener;
use tokio::sync::mpsc;
use tonic::{
    transport::Server,
    Request, Response, Status, Streaming,
};

pub mod pb {
    tonic::include_proto!("/grpc.examples.echo");
}
use pb::{EchoRequest, EchoResponse};

type EchoResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<EchoResponse, Status>> + Send + Sync>>;

#[derive(Default)]
pub struct EchoServer;

#[tonic::async_trait]
impl pb::echo_server::Echo for EchoServer {
    async fn unary_echo(&self, request: Request<EchoRequest>) -> EchoResult<EchoResponse> {
        println!("UnaryEcho = {:?}", request);
        let message = request.into_inner().message;
        Ok(Response::new(EchoResponse { message }))
    }

    type ServerStreamingEchoStream = mpsc::Receiver<Result<EchoResponse, Status>>;

    async fn server_streaming_echo(
        &self,
        request: Request<EchoRequest>,
    ) -> EchoResult<Self::ServerStreamingEchoStream> {
        println!("ServerStreamingEcho = {:?}", request);
        
        let (mut tx, rx) = mpsc::channel(4);
        let message = request.into_inner().message;

        tokio::spawn(async move {
            for c in message.chars() {
                let response = EchoResponse { message: c.to_string() };
                tx.send(Ok(response)).await.unwrap();

                tokio::time::delay_for(Duration::from_secs(1)).await;
            }
            println!("server_streaming_echo done sending");
        });

        Ok(Response::new(rx))
    }

    async fn client_streaming_echo(
        &self,
        _: Request<Streaming<EchoRequest>>,
    ) -> EchoResult<EchoResponse> {
        Err(Status::unimplemented("not implemented"))
    }

    type BidirectionalStreamingEchoStream = ResponseStream;

    async fn bidirectional_streaming_echo(
        &self,
        _: Request<Streaming<EchoRequest>>,
    ) -> EchoResult<Self::BidirectionalStreamingEchoStream> {
        Err(Status::unimplemented("not implemented"))
    }
}

#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "/tmp/tonic/helloworld";

    tokio::fs::create_dir_all(Path::new(path).parent().unwrap()).await?;

    let mut uds = UnixListener::bind(path)?;

    let server = EchoServer::default();

    Server::builder()
        .add_service(pb::echo_server::EchoServer::new(server))
        .serve_with_incoming(uds.incoming().map_ok(unix::UnixStream))
        .await?;

    Ok(())
}

#[cfg(unix)]
mod unix {
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    use tokio::io::{AsyncRead, AsyncWrite};
    use tonic::transport::server::Connected;

    #[derive(Debug)]
    pub struct UnixStream(pub tokio::net::UnixStream);

    impl Connected for UnixStream {}

    impl AsyncRead for UnixStream {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<std::io::Result<usize>> {
            Pin::new(&mut self.0).poll_read(cx, buf)
        }
    }

    impl AsyncWrite for UnixStream {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            Pin::new(&mut self.0).poll_write(cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Pin::new(&mut self.0).poll_flush(cx)
        }

        fn poll_shutdown(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<std::io::Result<()>> {
            Pin::new(&mut self.0).poll_shutdown(cx)
        }
    }
}

#[cfg(not(unix))]
fn main() {
    panic!("The `uds` example only works on unix systems!");
}
