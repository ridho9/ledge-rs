use std::time::Instant;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use hdrhistogram::Histogram;
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

    #[arg(long)]
    bench: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let ctx = zmq::Context::new();
    let requester = ctx.socket(zmq::REQ).expect("failed create requester");
    requester
        // .connect("tcp://0.0.0.0:10000")
        .connect("ipc:///tmp/ledger.sock")
        .expect("error connect");

    if args.bench {
        return run_benchmark(&requester);
    }

    let (msg_len, buf) = build_message(&args)?;
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

fn run_benchmark(requester: &zmq::Socket) -> Result<()> {
    // 1. Create a dummy message (reuse same buffer to isolate network cost)
    // We create a dummy Args just to build the buffer
    let dummy_args = Args {
        account_id: 1,
        transaction_id: 1,
        amount: 100,
        bench: true,
    };
    let (len, buf) = build_message(&dummy_args)?;

    let mut hist = Histogram::<u64>::new(3).unwrap();
    let iterations = 10_000;

    println!("warming up (1,000 requests)...");
    for _ in 0..1_000 {
        requester.send(&buf[..len], 0)?;
        let mut msg = zmq::Message::new();
        requester.recv(&mut msg, 0)?;
    }

    println!("benchmarking ({} requests)...", iterations);
    for _ in 0..iterations {
        let start = Instant::now();

        requester.send(&buf[..len], 0)?;
        let mut msg = zmq::Message::new();
        requester.recv(&mut msg, 0)?;

        let duration = start.elapsed().as_micros() as u64;
        hist.record(duration).expect("value out of range");
    }

    println!("\n--- Latency Results (microseconds) ---");
    println!("Min :  {} µs", hist.min());
    println!("Mean: {:.2} µs", hist.mean());
    println!("P50 :  {} µs", hist.value_at_quantile(0.50));
    println!("P99 :  {} µs", hist.value_at_quantile(0.99));
    println!("Max :  {} µs", hist.max());
    println!("--------------------------------------");

    Ok(())
}
