use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use ledger_messages::{
    message_header_codec, post_transaction_codec::PostTransactionEncoder, Encoder, WriteBuf,
};

fn benchmark_sbe_encoding(c: &mut Criterion) {
    // 1. Pre-allocate buffer (simulate stack allocation)
    let mut buffer = [0u8; 128];

    c.bench_function("sbe_encode_post_transaction", |b| {
        // b.iter runs this loop millions of times to get statistically significant data
        b.iter(|| {
            // 2. Wrap buffer
            let mut encoder = PostTransactionEncoder::default();
            encoder = encoder.wrap(
                WriteBuf::new(black_box(&mut buffer)),
                message_header_codec::ENCODED_LENGTH,
            );

            // 3. Write Header
            encoder = encoder.header(0).parent().unwrap();

            // 4. Write Body
            encoder.transaction_id(black_box(1001));
            encoder.account_id(black_box(50));
            encoder.amount(black_box(999));
            encoder.timestamp(black_box(123456789));

            // black_box prevents the compiler from optimizing this away entirely
            black_box(encoder.get_limit());
        })
    });
}

criterion_group!(benches, benchmark_sbe_encoding);
criterion_main!(benches);
