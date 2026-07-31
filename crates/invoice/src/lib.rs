use std::io::Write;
use std::path::Path;
use std::process::Command;

use chrono::Datelike;
use typst::World;
use typst::compile;
use typst::diag::{FileError, FileResult, Warned};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};
use typst_layout::PagedDocument;
use typst_pdf::{PdfOptions, pdf};

#[derive(Debug, Clone)]
pub struct InvoiceData {
  pub invoice_no: String,
  pub leader_name: String,
  pub leader_address: String,
  pub leader_email: String,
  pub date_of_issue: String,
  pub date_of_payment: String,
  pub unit_price: i32,
  pub subtotal: i32,
  pub discount_amount: i32,
  pub discount_code: Option<String>,
  pub total: i32,
  pub amount_paid: i32,
  pub balance_due: i32,
  pub cf_txn_id: Option<String>,
  pub payment_provider: String,
  pub txn_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfMethod {
  Cli,
  Crate,
}

#[derive(Debug, thiserror::Error)]
pub enum InvoiceError {
  #[error("{context}: {source}")]
  Io {
    source: std::io::Error,
    context: &'static str,
  },
  #[error("typst compilation failed: {0}")]
  Compilation(String),
}

#[allow(clippy::too_many_arguments)]
pub fn build_invoice_data(
  invoice_no: &str,
  leader_name: &str,
  leader_address: &str,
  leader_email: &str,
  date_of_issue: &str,
  date_of_payment: &str,
  unit_price: i32,
  subtotal: i32,
  discount_amount: i32,
  discount_code: Option<&str>,
  total: i32,
  amount_paid: i32,
  balance_due: i32,
  cf_txn_id: Option<&str>,
  payment_provider: &str,
  txn_id: &str,
) -> InvoiceData {
  InvoiceData {
    invoice_no: invoice_no.to_string(),
    leader_name: leader_name.to_string(),
    leader_address: leader_address.to_string(),
    leader_email: leader_email.to_string(),
    date_of_issue: date_of_issue.to_string(),
    date_of_payment: date_of_payment.to_string(),
    unit_price,
    subtotal,
    discount_amount,
    discount_code: discount_code.map(String::from),
    total,
    amount_paid,
    balance_due,
    cf_txn_id: cf_txn_id.map(String::from),
    payment_provider: payment_provider.to_string(),
    txn_id: txn_id.to_string(),
  }
}

fn escape_typst(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  for c in s.chars() {
    match c {
      '\\' => result.push_str("\\\\"),
      '#' => result.push_str("\\#"),
      '[' => result.push_str("\\["),
      ']' => result.push_str("\\]"),
      '{' => result.push_str("\\{"),
      '}' => result.push_str("\\}"),
      '$' => result.push_str("\\$"),
      '@' => result.push_str("\\@"),
      _ => result.push(c),
    }
  }
  result
}

fn substitute(template: &str, data: &InvoiceData) -> String {
  let fmt2 = |n: i32| format!("{:.2}", n as f64);
  let nos = |n: i32| n.to_string();

  let pct = if data.subtotal > 0 {
    (data.discount_amount as f64 * 100.0 / data.subtotal as f64) as i32
  } else {
    0
  };
  let discount_label = match &data.discount_code {
    Some(code) => {
      let escaped = escape_typst(code);
      format!(" \\[ Promo: {escaped} \\] (-{pct}%)")
    }
    None => format!(" (-{pct}%)"),
  };

  let leader_email_url = escape_typst(&data.leader_email);
  let leader_email_display = escape_typst(&data.leader_email);
  let leader_name = escape_typst(&data.leader_name);
  let invoice_no = escape_typst(&data.invoice_no);
  let date_of_issue = escape_typst(&data.date_of_issue);
  let date_of_payment = escape_typst(&data.date_of_payment);
  let cf_txn_id = data
    .cf_txn_id
    .as_deref()
    .map(escape_typst)
    .unwrap_or_else(|| "—".to_string());
  let payment_provider = escape_typst(&data.payment_provider);
  let txn_id = escape_typst(&data.txn_id);

  template
    .replace("{{INVOICE_NO}}", &invoice_no)
    .replace("{{INVOICE_STATUS}}", "PAID")
    .replace("{{DATE_OF_ISSUE}}", &date_of_issue)
    .replace("{{DATE_OF_PAYMENT}}", &date_of_payment)
    .replace("{{LEADER_NAME}}", &leader_name)
    .replace("{{LEADER_ADDRESS}}", &{
      let parts: Vec<&str> = data.leader_address.split(", ").collect();
      let escaped_parts: Vec<String> =
        parts.iter().map(|p| escape_typst(p)).collect();
      let n = escaped_parts.len();
      let lines = if n <= 3 {
        1
      } else if n <= 6 {
        2
      } else {
        3
      };
      let chunk_sz = n.div_ceil(lines);
      escaped_parts
        .chunks(chunk_sz)
        .map(|c| c.join(", "))
        .collect::<Vec<_>>()
        .join(" \\\n")
        + " \\\n"
    })
    .replace("{{LEADER_EMAIL}}", &leader_email_url)
    .replace("{{LEADER_EMAIL_AT}}", &leader_email_display)
    .replace("{{UNIT_PRICE}}", &fmt2(data.unit_price))
    .replace("{{SUBTOTAL_AMOUNT}}", &fmt2(data.subtotal))
    .replace("{{DISCOUNT_CODE_LABEL}}", &discount_label)
    .replace("{{DISCOUNT_AMOUNT}}", &fmt2(data.discount_amount))
    .replace("{{TOTAL_EXCL_TAX}}", &fmt2(data.total))
    .replace("{{TAX_AMOUNT}}", "0.00")
    .replace("{{TOTAL}}", &fmt2(data.total))
    .replace("{{AMOUNT_PAID}}", &fmt2(data.amount_paid))
    .replace("{{AMOUNT_PAID_NOS}}", &nos(data.amount_paid))
    .replace("{{BALANCE_DUE}}", &fmt2(data.balance_due))
    .replace("{{CF_TXN_ID}}", &cf_txn_id)
    .replace("{{PAYMENT_PROVIDER}}", &payment_provider)
    .replace("{{TXN_ID}}", &txn_id)
}

pub fn generate_invoice(
  assets_dir: &Path,
  data: &InvoiceData,
  method: PdfMethod,
) -> Result<Vec<u8>, InvoiceError> {
  match method {
    PdfMethod::Cli => match generate_via_cli(assets_dir, data) {
      Ok(pdf) => Ok(pdf),
      Err(InvoiceError::Io { source, context })
        if context == "run typst compile"
          && source.kind() == std::io::ErrorKind::NotFound =>
      {
        tracing::warn!("typst CLI not found, falling back to crate method");
        generate_via_crate(assets_dir, data)
      }
      Err(e) => Err(e),
    },
    PdfMethod::Crate => generate_via_crate(assets_dir, data),
  }
}

fn generate_via_cli(
  assets_dir: &Path,
  data: &InvoiceData,
) -> Result<Vec<u8>, InvoiceError> {
  let template = include_str!("template.typ");
  let source = substitute(template, data);

  let tmp_base = std::env::temp_dir();
  std::fs::create_dir_all(&tmp_base).map_err(|e| InvoiceError::Io {
    source: e,
    context: "create temp base dir",
  })?;
  let tmp = tempfile::tempdir_in(&tmp_base).map_err(|e| InvoiceError::Io {
    source: e,
    context: "create temp dir",
  })?;
  let typ_path = tmp.path().join("invoice.typ");
  let pdf_path = tmp.path().join("invoice.pdf");
  let logo_dst = tmp.path().join("logo.png");

  let mut f =
    std::fs::File::create(&typ_path).map_err(|e| InvoiceError::Io {
      source: e,
      context: "write invoice.typ",
    })?;
  f.write_all(source.as_bytes())
    .map_err(|e| InvoiceError::Io {
      source: e,
      context: "fill invoice.typ",
    })?;
  drop(f);

  let logo_src = assets_dir.join("png").join("logo.png");
  std::fs::copy(&logo_src, &logo_dst).map_err(|e| InvoiceError::Io {
    source: e,
    context: "copy logo.png",
  })?;

  let font_path = assets_dir.join("ttf");

  let output = Command::new("typst")
    .arg("compile")
    .arg(format!("--font-path={}", font_path.display()))
    .arg("--ignore-system-fonts")
    .arg(&typ_path)
    .arg(&pdf_path)
    .output()
    .map_err(|e| InvoiceError::Io {
      source: e,
      context: "run typst compile",
    })?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(
      &output.stderr[..output.stderr.len().min(10_240)],
    )
    .to_string();
    return Err(InvoiceError::Compilation(stderr));
  }

  let pdf = std::fs::read(&pdf_path).map_err(|e| InvoiceError::Io {
    source: e,
    context: "read invoice.pdf",
  })?;
  Ok(pdf)
}

fn load_fonts(
  assets_dir: &Path,
) -> Result<(LazyHash<FontBook>, Vec<Font>), InvoiceError> {
  let font_dir = assets_dir.join("ttf");
  let mut fonts = Vec::new();

  let entries = std::fs::read_dir(&font_dir).map_err(|e| InvoiceError::Io {
    source: e,
    context: "read ttf directory",
  })?;

  for entry in entries.flatten() {
    let path = entry.path();
    if path.is_dir() {
      let subdir = std::fs::read_dir(&path).map_err(|e| InvoiceError::Io {
        source: e,
        context: "read font subdirectory",
      })?;
      for file_entry in subdir.flatten() {
        let file_path = file_entry.path();
        if file_path
          .extension()
          .is_some_and(|ext| ext.eq_ignore_ascii_case("ttf"))
        {
          let data =
            std::fs::read(&file_path).map_err(|e| InvoiceError::Io {
              source: e,
              context: "read font file",
            })?;
          for font in Font::iter(Bytes::new(data)) {
            fonts.push(font);
          }
        }
      }
    }
  }

  if fonts.is_empty() {
    tracing::warn!("no TTF fonts loaded from {:?}", font_dir);
  }

  let book = FontBook::from_fonts(&fonts);
  Ok((LazyHash::new(book), fonts))
}

struct TypstWorld {
  library: LazyHash<Library>,
  book: LazyHash<FontBook>,
  fonts: Vec<Font>,
  main_id: FileId,
  logo_id: FileId,
  source: Source,
  logo_bytes: Bytes,
}

impl World for TypstWorld {
  fn library(&self) -> &LazyHash<Library> {
    &self.library
  }

  fn book(&self) -> &LazyHash<FontBook> {
    &self.book
  }

  fn main(&self) -> FileId {
    self.main_id
  }

  fn source(&self, id: FileId) -> FileResult<Source> {
    if id == self.main_id {
      Ok(self.source.clone())
    } else {
      Err(FileError::NotFound(std::path::PathBuf::from(
        "source not found",
      )))
    }
  }

  fn file(&self, id: FileId) -> FileResult<Bytes> {
    if id == self.logo_id {
      Ok(self.logo_bytes.clone())
    } else {
      Err(FileError::NotFound(std::path::PathBuf::from(
        "file not found",
      )))
    }
  }

  fn font(&self, index: usize) -> Option<Font> {
    self.fonts.get(index).cloned()
  }

  fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
    let now = chrono::Utc::now();
    let ist = chrono::FixedOffset::east_opt(5 * 3600 + 30 * 60)?;
    let dt = now.with_timezone(&ist);
    Datetime::from_ymd(dt.year(), dt.month() as u8, dt.day() as u8)
  }
}

fn generate_via_crate(
  assets_dir: &Path,
  data: &InvoiceData,
) -> Result<Vec<u8>, InvoiceError> {
  let template = include_str!("template.typ");
  let source_text = substitute(template, data);

  let vpath = VirtualPath::new("invoice.typ")
    .map_err(|e| InvoiceError::Compilation(e.to_string()))?;
  let logo_vpath = VirtualPath::new("logo.png")
    .map_err(|e| InvoiceError::Compilation(e.to_string()))?;

  let main_id = FileId::new(RootedPath::new(VirtualRoot::Project, vpath));
  let logo_id = FileId::new(RootedPath::new(VirtualRoot::Project, logo_vpath));

  let source = Source::new(main_id, source_text);

  let logo_path = assets_dir.join("png").join("logo.png");
  let logo_data = std::fs::read(&logo_path).map_err(|e| InvoiceError::Io {
    source: e,
    context: "read logo.png",
  })?;
  let logo_bytes = Bytes::new(logo_data);

  let (book, fonts) = load_fonts(assets_dir)?;
  let library = LazyHash::new(Library::default());

  let world = TypstWorld {
    library,
    book,
    fonts,
    main_id,
    logo_id,
    source,
    logo_bytes,
  };

  let Warned { output, warnings } = compile::<PagedDocument>(&world);

  for warn in &warnings {
    tracing::warn!("typst warning: {}", warn.message);
  }

  let doc = output.map_err(|diags| {
    let msgs: Vec<String> =
      diags.iter().map(|d| d.message.to_string()).collect();
    InvoiceError::Compilation(msgs.join("\n"))
  })?;

  let pdf_bytes = pdf(&doc, &PdfOptions::default()).map_err(|diags| {
    let msgs: Vec<String> =
      diags.iter().map(|d| d.message.to_string()).collect();
    InvoiceError::Compilation(msgs.join("\n"))
  })?;

  Ok(pdf_bytes)
}
