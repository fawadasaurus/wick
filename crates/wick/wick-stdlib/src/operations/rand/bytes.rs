use futures::StreamExt;
use seeded_random::Random;
use wick_packet::{fan_out, Observer, Packet, PacketStream};

use crate::request_response;

request_response!(job, minijob => {
  inputs: {
    length => u32,
    seed => u64,
  },
  output: "output",
});

pub(crate) fn minijob(length: u32, seed: u64) -> Result<Vec<u8>, wick_packet::Error> {
  let rng = Random::from_seed(seeded_random::Seed::unsafe_new(seed));
  let string = rng.bytes(length as _);
  Ok(string)
}
