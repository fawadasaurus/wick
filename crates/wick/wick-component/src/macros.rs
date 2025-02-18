#[macro_export]
macro_rules! wick_import {
  () => {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
  };
}

#[macro_export]
macro_rules! payloads {
    ($(($port:expr, $value:expr)),*) => {
      {
        let mut msgs = std::vec::Vec::new();
        let mut ports = std::collections::HashSet::new();
        $(
          ports.insert($port.to_owned());
          let md = wasmrs::Metadata::new_extra(0, $crate::WickMetadata::new($port));
          msgs.push(wasmrs::Payload::new_data(Some(md), Some(serialize(&output).unwrap().into())));
        )*
        for port in ports {
          let md = wasmrs::Metadata::new_extra(0, $crate::WickMetadata::new_done($port));
          msgs.push(wasmrs::Payload::new_data(Some(md), None));
        }
        msgs
      }
    };
}

#[macro_export]
macro_rules! payload_stream {
  ($(($port:expr, $value:expr)),*) => {{
    use $crate::wasmrs_rx::Observer;

    let packets = $crate::packet::packets!($(($port, $value)),*);
    let (tx,rx) = $crate::wasmrs_rx::FluxChannel::new_parts();
    for p in packets {
      tx.send(p).unwrap();
    }
    rx
  }};
}

#[macro_export]
macro_rules! handle_port {
  // ($packet:ident, $tx:ident, $subtx:ident, $port:expr, WickStream<$ty:ty> ) => {{
  //   use $crate::wasmrs_rx::Observer;
  //   if $packet.extra.is_done() {
  //     $tx.complete();
  //   } else {
  //     let packet: Result<$ty, _> = $packet.deserialize().map_err(|e| e.into());
  //     let _ = $tx.send_result(packet);
  //   }
  // }};
  (raw: true, $packet:ident, $tx:ident, $port:expr, $ty:ty) => {{
    use $crate::wasmrs_rx::Observer;
    if $packet.is_done() {
      $tx.complete();
    } else {
      // let packet: Result<$ty, _> = $packet.deserialize().map_err(|e| e.into());
      let _ = $tx.send($packet);
    }
  }};
  (raw: false, $packet:ident, $tx:ident, $port:expr, $ty:ty) => {{
    use $crate::wasmrs_rx::Observer;
    if $packet.is_done() {
      $tx.complete();
    } else {
      let packet: Result<$ty, _> = $packet.deserialize().map_err(|e| e.into());
      let _ = $tx.send_result(packet);
    }
  }};
}

#[macro_export]
macro_rules! payload_fan_out {
    ($stream:expr, raw:$raw:tt, [ $(($port:expr, $($ty:tt)+)),* $(,)? ]) => {
      {
          $crate::paste::paste! {
            $(
              #[allow(unused_parens)]
              let ([<$port:snake _tx>],[<$port:snake _rx>]) = $crate::wasmrs_rx::FluxChannel::new_parts();
            )*
          }
        $crate::runtime::spawn(async move {

          use $crate::StreamExt;
          while let Some(Ok( payload)) = $stream.next().await {
            let packet: $crate::packet::Packet = payload.into();
            match packet.port() {
              $(
                $port=> {
                  let tx = &$crate::paste::paste! {[<$port:snake _tx>]};
                  $crate::handle_port!(raw: $raw, packet, tx, $port, $($ty)*)
                },
              )*
              _ => panic!("Unexpected port: {}", packet.port())
            }
          }
        });
        $crate::paste::paste! {($([<$port:snake _rx>]),*)}
        }
    };
}

#[cfg(test)]
mod test {
  use anyhow::Result;
  use futures::StreamExt;
  use wasmrs::PayloadError;
  use wasmrs_rx::FluxReceiver;
  use wick_packet::Packet;

  #[tokio::test]
  async fn test_basic() -> Result<()> {
    let mut stream: FluxReceiver<Packet, PayloadError> =
      payload_stream!(("foo", 1), ("bar", 2), ("foo", 3), ("bar", 4), ("foo", 5), ("bar", 6));
    let (mut foo_rx, mut bar_rx): (FluxReceiver<i32, anyhow::Error>, FluxReceiver<i32, anyhow::Error>) =
      payload_fan_out!(stream, raw: false, [("foo", i32), ("bar", i32)]);
    assert_eq!(foo_rx.next().await.unwrap().unwrap(), 1);
    assert_eq!(bar_rx.next().await.unwrap().unwrap(), 2);
    assert_eq!(foo_rx.next().await.unwrap().unwrap(), 3);
    assert_eq!(bar_rx.next().await.unwrap().unwrap(), 4);
    assert_eq!(foo_rx.next().await.unwrap().unwrap(), 5);
    assert_eq!(bar_rx.next().await.unwrap().unwrap(), 6);

    Ok(())
  }
}
