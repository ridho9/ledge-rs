use std::time::Duration;

use anyhow::Result;
use ledger_messages::{
    Encoder, ReadBuf, SbeResult, WriteBuf,
    command_response_codec::CommandResponseEncoder,
    message_header_codec::{self, MessageHeaderDecoder},
    post_transaction_codec::PostTransactionDecoder,
    response_code::ResponseCode,
    transaction_posted_codec::TransactionPostedEncoder,
};
use rdkafka::producer::{FutureProducer, FutureRecord};

#[tokio::main]
async fn main() -> Result<()> {
    // ========================== setup kafka publisher
    let producer: FutureProducer = rdkafka::ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("failed creating producer");

    // ========================== run zmq responder
    let ctx = zmq::Context::new();
    let responder = ctx.socket(zmq::REP).unwrap();
    responder
        // .bind("tcp://0.0.0.0:10000")
        .bind("ipc:///tmp/ledger.sock")
        .expect("err connect");

    let mut resp_buf = vec![0; 4096];
    let mut msg = zmq::Message::new();

    loop {
        match responder.recv(&mut msg, 0) {
            Ok(_) => {
                handle_request(producer.clone(), &msg);

                let msg_len = build_response(&mut resp_buf)?;
                responder.send(&resp_buf[..msg_len], 0)?;
            }
            Err(err) => {
                eprintln!("error receiving message: {}", err.message())
            }
        }
    }
}

fn build_response(buf: &mut [u8]) -> SbeResult<usize> {
    let mut command_response = CommandResponseEncoder::default();

    command_response =
        command_response.wrap(WriteBuf::new(buf), message_header_codec::ENCODED_LENGTH);
    command_response = command_response.header(0).parent()?;

    command_response.response_code(ResponseCode::OK);
    command_response.error_message("");

    let limit = command_response.get_limit();
    return Ok(limit);
}

fn handle_request(producer: FutureProducer, msg: &zmq::Message) {
    let buf = ReadBuf::new(msg);
    let header = MessageHeaderDecoder::default().wrap(buf, 0);
    let mut post_transaction = PostTransactionDecoder::default();

    post_transaction = post_transaction.header(header, 0);
    let tx_id = post_transaction.transaction_id();
    let account_id = post_transaction.account_id();
    let amount = post_transaction.amount();
    let timestamp = post_transaction.timestamp();

    println!("received post_transaction");
    println!("\ttx id {}", tx_id);
    println!("\taccount id {}", account_id);
    println!("\tamount {}", amount);
    println!("\ttimestamp {}", timestamp);

    let (tx_post_len, tx_post) = build_transaction_posted(tx_id, account_id, amount, timestamp);

    tokio::spawn(async move {
        let produce_status = producer
            .send(
                FutureRecord::to("transactions")
                    .key(&tx_id.to_le_bytes())
                    .payload(&tx_post[..tx_post_len]),
                Duration::from_secs(0),
            )
            .await;

        match produce_status {
            Ok(_) => (),
            Err((err, err_msg)) => eprintln!("failed kafka publish: {} {:?}", err, err_msg),
        }
    });
}

fn build_transaction_posted(
    tx_id: u64,
    account_id: u64,
    amount: i64,
    timestamp: u64,
) -> (usize, [u8; 96]) {
    let mut buf = [0u8; 96];
    let mut encoder = TransactionPostedEncoder::default();
    encoder = encoder.wrap(
        WriteBuf::new(&mut buf),
        message_header_codec::ENCODED_LENGTH,
    );
    encoder = encoder.header(0).parent().unwrap();
    encoder.transaction_id(tx_id);
    encoder.account_id(account_id);
    encoder.amount(amount);
    encoder.timestamp(timestamp);
    let limit = encoder.get_limit();
    (limit, buf)
}
