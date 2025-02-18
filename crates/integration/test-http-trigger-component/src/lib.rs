use std::io::{self, BufRead, BufReader};

use futures::TryStreamExt;
use wasmrs_guest::StreamExt;
mod generated;
use generated as wick;
// mod wick {
//   wick_component::wick_import!();
// }
use wick::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct Request {
  message: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Response {
  output_message: String,
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl OpHttpHandler for Component {
  async fn http_handler(
    mut request: WickStream<HttpRequest>,
    body: WickStream<bytes::Bytes>,
    mut outputs: OpHttpHandlerOutputs,
  ) -> Result<()> {
    if let (Some(Ok(request))) = (request.next().await) {
      println!("{:#?}", request);
    }

    let body: bytes::BytesMut = body.try_collect().await?;
    let res_body = if body.is_empty() {
      bytes::Bytes::new()
    } else {
      let body = String::from_utf8(body.into())?;
      let req: Request = serde_json::from_str(&body)?;
      let new: String = req.message.chars().rev().collect();
      serde_json::to_string(&Response { output_message: new })?.into()
    };
    let mut res = HttpResponse {
      version: HttpVersion::Http11,
      status: StatusCode::Ok,
      headers: std::collections::HashMap::new(),
    };
    res
      .headers
      .insert("Content-Type".to_owned(), vec!["application/json".to_owned()]);
    res
      .headers
      .insert("Access-Control-Allow-Origin".to_owned(), vec!["*".to_owned()]);
    res
      .headers
      .insert("Content-Length".to_owned(), vec![res_body.len().to_string()]);

    println!("SENDING RESPONSE BODY {:#?}", res_body);
    outputs.response.send(&res);
    outputs.body.send(&res_body);
    outputs.response.done();
    outputs.body.done();
    Ok(())
  }
}
