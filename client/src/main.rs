use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use ledger_messages::{
    Encoder, ReadBuf, SbeResult, WriteBuf,
    command_response_codec::CommandResponseDecoder,
    message_header_codec::{self, MessageHeaderDecoder},
    post_transaction_codec::PostTransactionEncoder,
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long = "acc")]
    account_id: u64,

    #[arg(long = "tx")]
    transaction_id: u64,

    #[arg(long = "amt")]
    amount: i64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (msg_len, buf) = build_message(&args)?;
    let ctx = zmq::Context::new();
    let requester = ctx.socket(zmq::REQ).expect("failed create requester");
    requester
        .connect("tcp://0.0.0.0:10000")
        .expect("error connect");
    requester.send(&buf[..msg_len], 0).expect("error send");
    println!("sent post_transaction message");

    let mut msg = zmq::Message::new();
    requester.recv(&mut msg, 0)?;
    handle_response(&msg).expect("failed handling response");

    Ok(())
}

fn build_message(args: &Args) -> SbeResult<(usize, Vec<u8>)> {
    let mut buf = vec![0; 4096];
    let mut post_transaction = PostTransactionEncoder::default();

    post_transaction = post_transaction.wrap(
        WriteBuf::new(&mut buf),
        message_header_codec::ENCODED_LENGTH,
    );
    post_transaction = post_transaction.header(0).parent()?;

    post_transaction.account_id(args.account_id);
    post_transaction.transaction_id(args.transaction_id);
    post_transaction.amount(args.amount);
    post_transaction.timestamp(Utc::now().timestamp_millis().try_into().unwrap());

    let limit = post_transaction.get_limit();
    return Ok((limit, buf));
}

fn handle_response(msg: &zmq::Message) -> Result<()> {
    let buf = ReadBuf::new(msg);
    let header = MessageHeaderDecoder::default().wrap(buf, 0);
    let mut command_response = CommandResponseDecoder::default();
    command_response = command_response.header(header, 0);

    let err_coord = command_response.error_message_decoder();
    let err_bytes = command_response.error_message_slice(err_coord);

    println!(
        "received response status {} msg {}",
        command_response.response_code(),
        str::from_utf8(err_bytes)?,
    );

    Ok(())
}
