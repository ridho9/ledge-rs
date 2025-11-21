use anyhow::Result;
use ledger_messages::{
    Encoder, ReadBuf, SbeResult, WriteBuf,
    command_response_codec::CommandResponseEncoder,
    message_header_codec::{self, MessageHeaderDecoder},
    post_transaction_codec::PostTransactionDecoder,
    response_code::ResponseCode,
};

fn main() -> Result<()> {
    let ctx = zmq::Context::new();
    let responder = ctx.socket(zmq::REP).unwrap();
    responder.bind("tcp://*:10000").expect("err connect");

    let mut resp_buf = vec![0; 4096];
    let mut msg = zmq::Message::new();

    loop {
        responder.recv(&mut msg, 0).expect("failed recv");

        handle_request(&msg);

        let msg_len = build_response(&mut resp_buf)?;
        responder.send(&resp_buf[..msg_len], 0)?;

        println!("=========")
    }
}

fn handle_request(msg: &zmq::Message) {
    let buf = ReadBuf::new(msg);
    let header = MessageHeaderDecoder::default().wrap(buf, 0);
    let mut post_transaction = PostTransactionDecoder::default();

    post_transaction = post_transaction.header(header, 0);
    println!("received post_transaction");
    println!("\taccount id {}", post_transaction.account_id());
    println!("\tamount {}", post_transaction.amount());
    println!("\ttimestamp {}", post_transaction.timestamp());
    println!("\ttx id {}", post_transaction.transaction_id());
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
