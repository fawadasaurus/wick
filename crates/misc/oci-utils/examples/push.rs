use std::path::PathBuf;
use std::str::FromStr;

use oci_distribution::client::{ImageData, ImageLayer};
use oci_distribution::manifest::OciImageManifest;
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::Reference;
use structopt::StructOpt;

#[derive(StructOpt, Default)]
struct Options {
  reference: String,

  path: PathBuf,

  media_type: String,

  #[structopt(long)]
  manifest: Option<PathBuf>,

  #[structopt(long)]
  config: Option<PathBuf>,

  #[structopt(long)]
  insecure: Vec<String>,

  #[structopt(long, env = "OCI_USERNAME")]
  username: Option<String>,

  #[structopt(long, env = "OCI_PASSWORD")]
  password: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  env_logger::init();
  let opts = Options::from_args();
  let protocol = oci_distribution::client::ClientProtocol::HttpsExcept(opts.insecure);
  let config = oci_distribution::client::ClientConfig {
    protocol,
    ..Default::default()
  };
  let mut c = oci_distribution::Client::new(config);

  let auth = match (opts.username, opts.password) {
    (Some(username), Some(password)) => RegistryAuth::Basic(username, password),
    (None, None) => RegistryAuth::Anonymous,
    _ => {
      println!("Both username and password must be supplied. Falling back to anonymous auth");
      RegistryAuth::Anonymous
    }
  };

  let bytes = std::fs::read(opts.path)?;

  let manifest: Option<OciImageManifest> = match opts.manifest {
    Some(path) => Some(serde_json::from_slice(&std::fs::read(path)?)?),
    None => None,
  };

  let configdata = match opts.config {
    Some(path) => std::fs::read(path)?,
    None => b"{}".to_vec(),
  };

  let imagedata = ImageData {
    layers: vec![ImageLayer {
      data: bytes,
      media_type: opts.media_type,
    }],
    digest: None,
  };

  let digest = c
    .push(
      &Reference::from_str(&opts.reference)?,
      &imagedata,
      &configdata,
      oci_distribution::manifest::IMAGE_CONFIG_MEDIA_TYPE,
      &auth,
      manifest,
    )
    .await?;

  println!("Image url: {}", digest.image_url);
  println!("Manifest url: {}", digest.manifest_url);
  println!("Config url: {:?}", digest.config_url);

  Ok(())
}
