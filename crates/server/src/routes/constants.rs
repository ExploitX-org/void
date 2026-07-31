use const_format::concatcp;

pub const ROOT: &str = "/_/api";
pub const VERSION: &str = "v2";
pub const PUB_SCOPE: &str = "pub";
pub const PRIV_SCOPE: &str = "_";

pub const REGISTER: &str =
  concatcp!(ROOT, "/", VERSION, "/", PUB_SCOPE, "/register");
pub const VERIFY: &str =
  concatcp!(ROOT, "/", VERSION, "/", PUB_SCOPE, "/authn/verify");
pub const RESEND: &str =
  concatcp!(ROOT, "/", VERSION, "/", PUB_SCOPE, "/authn/resend");
pub const CREATE_PAYMENT: &str =
  concatcp!(ROOT, "/", VERSION, "/", PUB_SCOPE, "/pmt/create");
pub const PAYMENT_STATUS: &str =
  concatcp!(ROOT, "/", VERSION, "/", PUB_SCOPE, "/pmt/status");

pub const SANDBOX: &str = "/sandbox";
pub const HEALTH: &str = "/health";
