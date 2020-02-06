use async_std::net::TcpStream;
use async_std::stream::StreamExt;
use colmeia_dat1::*;
use std::net::SocketAddr;

fn address() -> SocketAddr {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let input = args
        .first()
        .expect("must have dat server:port name as argument");
    input.parse().expect("invalid ip:port as input")
}

fn name() -> String {
    let args: Vec<String> = std::env::args().skip(2).collect();
    args.first().expect("must have dat name as argument").into()
}

// TODO: send to a folder
// use std::path::PathBuf;
// fn folder() -> PathBuf {
//     let args: Vec<String> = std::env::args().skip(3).collect();
//     args.first().expect("must have folder as argument").into()
// }

fn main() {
    env_logger::init();

    let address = address();
    let key = name();
    // let path = name();
    async_std::task::block_on(async {
        let tcp_stream = TcpStream::connect(address)
            .await
            .expect("could not open address");
        let client_initialization = new_client(&key, tcp_stream).await;
        let public_key = client_initialization.dat_key().public_key().clone();
        let client = handshake(client_initialization)
            .await
            .expect("could not handshake");

        let observer = PeeredHyperdrive::new(public_key);
        let mut service = DatService::new(client, observer);

        while let Some(message) = service.next().await {
            if let DatMessage::Feed(message) = message.parse().unwrap() {
                eprintln!(
                    "Received message {:?}",
                    hex::encode(message.get_discoveryKey())
                );
            }
        }
    });
}