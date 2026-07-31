use tracing;

pub const LOGO_PNG: &[u8] =
  include_bytes!("../../../assets/png/logo-email.png");
pub const MASCOT_PNG: &[u8] = include_bytes!("../../../assets/png/mascot.png");

fn html_escape(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#39;")
}

pub fn render_otp_email(otp: &str, ttl_secs: i64) -> String {
  let expires_in = if ttl_secs >= 60 {
    let mins = ttl_secs / 60;
    if mins == 1 {
      "1 minute".into()
    } else {
      format!("{mins} minutes")
    }
  } else {
    format!("{ttl_secs} seconds")
  };

  let digits: String = otp
    .chars()
    .map(|ch| {
      format!(
        r#"<td style="padding:3px;"><div style="width:50px;height:64px;border:2px solid #9EFF00;border-radius:8px;text-align:center;line-height:64px;font-size:30px;font-weight:bold;color:#9EFF00;">{ch}</div></td>"#
      )
    })
    .collect::<Vec<_>>()
    .join("\n");

  format!(
    r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Into The Void 2.0 - OTP Verification</title>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;700&display=swap" rel="stylesheet">
</head>
<body style="margin:0;padding:0;background:#030303;font-family:Arial,Helvetica,sans-serif;color:#ffffff;">

<div style="display:none; max-height:0px; max-width:0px; opacity:0; overflow:hidden; mso-hide:all; font-size:1px; line-height:1px;">
Secure your spot at the event. Enter this code to verify your registration.
</div>

<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background:#030303;">
<tr>
<td align="center" style="padding:40px 0;">

<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="max-width:600px;background:#080808;border:1px solid #9EFF00;border-radius:20px;">

<tr>
<td align="center" style="padding:30px;border-bottom:1px solid rgba(158,255,0,.25);">
<img src="cid:logo@exploitx" width="140" alt="ExploitX" style="display:block;margin:0 auto;">
<div style="color:#9EFF00;font-size:14px;letter-spacing:4px;margin-top:10px;font-weight:bold;">INTO THE VOID 2.0</div>
</td>
</tr>

<tr>
<td align="center" style="padding:30px 30px 10px 30px;">
<img src="cid:mascot@exploitx" width="500" alt="Mascot" style="display:block;margin:0 auto;max-width:100%;height:auto;">
<h1 style="color:#9EFF00;margin:25px 0 15px;font-size:32px;letter-spacing:1px;">VERIFY YOUR REGISTRATION</h1>
<p style="color:#e0e0e0;line-height:1.6;font-size:15px;margin:0;">
Hello Explorer,<br><br>
Thank you for registering for <b>Into The Void 2.0</b>.<br>To complete your application and secure your spot at the event, please use the secure One-Time Password (OTP) below:
</p>
</td>
</tr>

<tr>
<td style="padding:20px 30px 30px 30px;">

<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="border:1px solid rgba(158,255,0,.3);border-radius:16px;background:#0b0b0b;">
<tr>
<td align="center" style="padding:30px 25px;">

<div style="color:#9EFF00;margin-bottom:20px;font-size:14px;letter-spacing:2px;font-weight:bold;">YOUR OTP CODE</div>

<table role="presentation" cellpadding="0" cellspacing="0" style="margin:0 auto;">
<tr>
{digits}
</tr>
</table>

<div style="color:#bbbbbb;margin-top:25px;font-size:14px;line-height:1.5;">
This code is valid for the next <b>{expires_in}</b>.<br>
<span style="color:#888888;font-size:13px;">For security reasons, do not share this verification code with anyone.</span>
</div>

<hr style="border:0;border-top:1px solid rgba(158,255,0,.15);margin:25px 0 20px 0;">

<div style="color:#666666;font-size:12px;line-height:1.6;">
&copy; 2026 ExploitX<br>
<span style="color:#9EFF00;opacity:0.7;">Into The Void 2.0 &bull; Explore &bull; Exploit &bull; Excel</span>
</div>

</td>
</tr>
</table>

</td>
</tr>

</table>

</td>
</tr>
</table>

</body>
</html>"#,
    digits = digits,
    expires_in = expires_in
  )
}

pub fn render_invoice_email(
  leader_name: &str,
  invoice_no: &str,
  has_pdf: bool,
  team_name: &str,
  reg_type: &str,
  amount: i32,
) -> String {
  let reg_type_display = match reg_type {
    "couple" => "Couple",
    "single" => "Single",
    other => {
      tracing::warn!("unknown registration type: {other}");
      "Single"
    }
  };
  let invoice_line = if has_pdf {
    "<br>Your official invoice is attached to this transmission."
  } else {
    ""
  };

  format!(
    r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Registration Confirmed - Into The Void 2.0</title>
</head>
<body style="margin:0;padding:0;background:#030303;font-family:Arial,Helvetica,sans-serif;color:#ffffff;">

<div style="display:none; max-height:0px; max-width:0px; opacity:0; overflow:hidden; mso-hide:all; font-size:1px; line-height:1px;">
Your spot at Into The Void 2.0 is secured! View your registration details and invoice inside.
</div>

<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background:#030303;">
<tr>
<td align="center" style="padding:40px 0;">

<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="max-width:600px;background:#080808;border:1px solid #9EFF00;border-radius:20px;">

<tr>
<td align="center" style="padding:30px;border-bottom:1px solid rgba(158,255,0,.25);">
<img src="cid:logo@exploitx" width="140" alt="ExploitX" style="display:block;margin:0 auto;">
<div style="color:#9EFF00;font-size:14px;letter-spacing:4px;margin-top:10px;font-weight:bold;">INTO THE VOID 2.0</div>
</td>
</tr>

<tr>
<td align="center" style="padding:30px 30px 10px 30px;">
<img src="cid:mascot@exploitx" width="500" alt="Mascot" style="display:block;margin:0 auto;max-width:100%;height:auto;">
<h1 style="color:#9EFF00;margin:25px 0 15px;font-size:32px;letter-spacing:1px;text-align:center;">REGISTRATION CONFIRMED</h1>
<p style="color:#e0e0e0;line-height:1.6;font-size:15px;margin:0;text-align:center;">
Hello {leader_name},<br><br>
Welcome to the grid. Your registration for <b>Into The Void 2.0</b> has been successfully processed, and your clearance is confirmed.{invoice_line}
</p>
</td>
</tr>

<tr>
<td style="padding:20px 30px 30px 30px;">

<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="border:1px solid rgba(158,255,0,.3);border-radius:16px;background:#0b0b0b;">
<tr>
<td style="padding:30px 25px;">

<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="font-size:15px;line-height:1.8;color:#d0d0d0;">
<tr>
<td style="padding:6px 0;"><span style="color:#9EFF00;font-weight:bold;">Team:</span></td>
<td align="right" style="color:#ffffff;">{team_name}</td>
</tr>
<tr>
<td style="padding:6px 0;"><span style="color:#9EFF00;font-weight:bold;">Registration:</span></td>
<td align="right" style="color:#ffffff;">{reg_type_display}</td>
</tr>
<tr>
<td style="padding:6px 0;"><span style="color:#9EFF00;font-weight:bold;">Amount Paid:</span></td>
<td align="right" style="color:#ffffff;">₹{amount}</td>
</tr>
<tr>
<td style="padding:6px 0;"><span style="color:#9EFF00;font-weight:bold;">Invoice No:</span></td>
<td align="right" style="color:#ffffff;font-family:monospace;">{invoice_no}</td>
</tr>
<tr>
<td style="padding:6px 0;"><span style="color:#9EFF00;font-weight:bold;">Event Date:</span></td>
<td align="right" style="color:#ffffff;">July 30 & July 31, 2026</td>
</tr>
</table>

<div style="color:#bbbbbb;margin-top:30px;font-size:14px;text-align:center;line-height:1.5;">
Thank you for stepping forward. We look forward to seeing you inside the experience.
</div>

<hr style="border:0;border-top:1px solid rgba(158,255,0,.15);margin:25px 0 20px 0;">

<div style="color:#666666;font-size:12px;line-height:1.6;text-align:center;">
&copy; 2026 ExploitX<br>
<span style="color:#9EFF00;opacity:0.7;">Into The Void 2.0 &bull; Explore &bull; Exploit &bull; Excel</span>
</div>

</td>
</tr>
</table>

</td>
</tr>

</table>

</td>
</tr>
</table>

</body>
</html>"#,
    leader_name = html_escape(leader_name),
    team_name = html_escape(team_name),
    reg_type_display = reg_type_display,
    amount = amount,
    invoice_no = html_escape(invoice_no),
    invoice_line = invoice_line
  )
}
