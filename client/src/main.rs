use anyhow::Result;
use chrono::Utc;
use ledger_messages::{
    Encoder, SbeResult, WriteBuf, message_header_codec,
    post_transaction_codec::PostTransactionEncoder,
};

fn main() -> Result<()> {
    let (msg_len, buf) = build_message()?;
    println!("{} {:?}", msg_len, &buf[..msg_len]);

    let ctx = zmq::Context::new();
    let requester = ctx.socket(zmq::REQ).expect("failed create requester");
    requester
        .connect("tcp://0.0.0.0:10000")
        .expect("error connect");
    requester.send(&buf[..msg_len], 0).expect("error send");

    let mut msq = zmq::Message::new();
    requester.recv(&mut msq, 0).unwrap();
    println!("response {}", msq.as_str().unwrap());
    Ok(())
}

fn build_message() -> SbeResult<(usize, Vec<u8>)> {
    let mut buf = vec![0; 4096];
    let mut post_transaction = PostTransactionEncoder::default();

    post_transaction = post_transaction.wrap(
        WriteBuf::new(&mut buf),
        message_header_codec::ENCODED_LENGTH,
    );
    post_transaction = post_transaction.header(0).parent()?;

    post_transaction.account_id(94);
    post_transaction.amount(10100);
    post_transaction.timestamp(Utc::now().timestamp_millis().try_into().unwrap());
    post_transaction.transaction_id(20001);

    let limit = post_transaction.get_limit();
    return Ok((limit, buf));
}
