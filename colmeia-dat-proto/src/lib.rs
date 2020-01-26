use async_std::net::TcpStream;
use async_std::stream::StreamExt;
use colmeia_dat_core::DatUrlResolution;
use protobuf::Message;
use rand::Rng;
use simple_message_channels::{Message as ChannelMessage, Reader, Writer};

use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

mod cipher;
mod schema;
mod socket;
use crate::cipher::Cipher;
use crate::schema as proto;

#[non_exhaustive]
#[derive(Debug)]
pub enum DatMessage {
    Feed(proto::Feed),
    Handshake(proto::Handshake),
    Info(proto::Info),
    Have(proto::Have),
    Unhave(proto::Unhave),
    Want(proto::Want),
    Unwant(proto::Unwant),
    Request(proto::Request),
    Cancel(proto::Cancel),
    Data(proto::Data),
}

type ParseResult = Result<DatMessage, protobuf::ProtobufError>;

pub trait MessageExt {
    fn parse(&self) -> ParseResult;
}

impl MessageExt for ChannelMessage {
    fn parse(&self) -> ParseResult {
        match self.typ {
            0 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Feed),
            1 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Handshake),
            2 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Info),
            3 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Have),
            4 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Unhave),
            5 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Want),
            6 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Unwant),
            7 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Request),
            8 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Cancel),
            9 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Data),
            _ => panic!("Uknonw message"), // TODO proper error handling
        }
    }
}

pub struct Client {
    reader: Reader<socket::CloneableStream>,
    writer: Writer<socket::CloneableStream>,
}

impl Client {
    pub fn reader(&mut self) -> &mut Reader<socket::CloneableStream> {
        &mut self.reader
    }

    pub fn writer(&mut self) -> &mut Writer<socket::CloneableStream> {
        &mut self.writer
    }
}

pub struct ClientInitialization {
    bare_reader: Reader<socket::CloneableStream>,
    bare_writer: Writer<socket::CloneableStream>,
    upgradable_reader: socket::CloneableStream,
    upgradable_writer: socket::CloneableStream,
    dat_key: colmeia_dat_core::HashUrl,
}

pub async fn new_client(key: &str, address: SocketAddr) -> ClientInitialization {
    let dat_key = colmeia_dat_core::parse(&key).expect("invalid dat argument");

    let dat_key = match dat_key {
        DatUrlResolution::HashUrl(result) => result,
        _ => panic!("invalid hash key"),
    };

    let socket = Arc::new(
        TcpStream::connect(address)
            .await
            .expect("could not open socket"),
    );

    let reader_cipher = Arc::new(RwLock::new(Cipher::new(
        dat_key.public_key().as_bytes().to_vec(),
    )));
    let reader_socket = socket::CloneableStream {
        socket: socket.clone(),
        cipher: reader_cipher,
        buffer: None,
    };
    let upgradable_reader = reader_socket.clone();
    let bare_reader = Reader::new(reader_socket);

    let writer_cipher = Arc::new(RwLock::new(Cipher::new(
        dat_key.public_key().as_bytes().to_vec(),
    )));
    let writer_socket = socket::CloneableStream {
        socket,
        cipher: writer_cipher,
        buffer: None,
    };
    let upgradable_writer = writer_socket.clone();
    let bare_writer = Writer::new(writer_socket);

    ClientInitialization {
        bare_reader,
        bare_writer,
        upgradable_reader,
        upgradable_writer,
        dat_key,
    }
}

pub async fn handshake(mut init: ClientInitialization) -> Option<Client> {
    let received_handshake = init.bare_reader.next().await?.ok()?;
    let parsed_handshake = received_handshake.parse().ok()?;
    let mut payload = match parsed_handshake {
        DatMessage::Feed(payload) => payload,
        _ => return None,
    };

    log::debug!("Dat feed received {:?}", payload);
    if payload.get_discoveryKey() != init.dat_key.discovery_key() && !payload.has_nonce() {
        return None;
    }
    log::debug!("Feed received, upgrading read socket");
    init.upgradable_reader.upgrade(payload.get_nonce());
    let reader = Reader::new(init.upgradable_reader);

    let nonce: [u8; 24] = rand::thread_rng().gen();
    let nonce = nonce.to_vec();
    payload.set_nonce(nonce);
    init.bare_writer
        .send(ChannelMessage::new(
            0,
            0,
            payload.write_to_bytes().expect("invalid feed message"),
        ))
        .await
        .ok()?;

    log::debug!("Feed sent, upgrading write socket");
    init.upgradable_writer.upgrade(payload.get_nonce());
    let mut writer = Writer::new(init.upgradable_writer);

    let mut handshake = proto::Handshake::new();
    let id: [u8; 32] = rand::thread_rng().gen();
    let id = id.to_vec();

    handshake.set_id(id);
    handshake.set_live(true);
    handshake.set_ack(false);
    log::debug!("Dat handshake to send {:?}", &handshake);

    writer
        .send(ChannelMessage::new(
            0,
            1,
            handshake
                .write_to_bytes()
                .expect("invalid handshake message"),
        ))
        .await
        .ok()?;

    log::debug!("Handshake finished");

    Some(Client { reader, writer })
}
