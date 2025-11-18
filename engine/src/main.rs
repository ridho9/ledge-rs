use ledger_messages::{
    ReadBuf, message_header_codec::MessageHeaderDecoder,
    post_transaction_codec::PostTransactionDecoder,
};

fn main() {
    let ctx = zmq::Context::new();
    let responder = ctx.socket(zmq::REP).unwrap();
    responder.bind("tcp://*:10000").expect("err connect");

    let mut msg = zmq::Message::new();
    loop {
        responder.recv(&mut msg, 0).expect("failed recv");

        let mut post_transaction = PostTransactionDecoder::default();
        let buf = ReadBuf::new(&msg);
        let header = MessageHeaderDecoder::default().wrap(buf, 0);

        post_transaction = post_transaction.header(header, 0);
        println!("account id {}", post_transaction.account_id());
        println!("amount {}", post_transaction.amount());
        println!("timestamp {}", post_transaction.timestamp());
        println!("tx id {}", post_transaction.transaction_id());
        println!("");

        responder.send("recevied", 0).unwrap();
    }
}
