pub fn sha256_hex(input: &[u8]) -> String {
  use sha2::Digest;
  let mut hasher = sha2::Sha256::new();
  hasher.update(input);
  hex::encode(hasher.finalize())
}

/// HMAC-SHA256 returning raw bytes.
pub fn hmac_sha256_raw(key: &[u8], data: &[u8]) -> Vec<u8> {
  use hmac::{Hmac, Mac};
  use sha2::Sha256;
  type HmacSha256 = Hmac<Sha256>;

  #[allow(clippy::expect_used)]
  let mut mac =
    HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
  mac.update(data);
  mac.finalize().into_bytes().to_vec()
}
