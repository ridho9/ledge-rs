use ledger_messages::{self as lm, WriteBuf, response_code::ResponseCode};

fn main() {
    let mut buf = vec![0; 4096];
    // let mut buf = WriteBuf::new([0; 1024]);

    let mut command_response = lm::command_response_codec::CommandResponseEncoder::default()
        .wrap(WriteBuf::new(&mut buf), 0);
    command_response.error_message(&"testing");
    command_response.response_code(ResponseCode::ERROR);

    let msg_len = command_response.encoded_length();
    println!("{} {:?}", msg_len, &buf[..msg_len]);
}
