const API_BASE_URL = window.location.hostname === 'localhost'
  ? 'http://localhost:4000/_/x'
  : 'https://api.void.exploitx.org/_/x';

const GOD_URL = `${API_BASE_URL}/god`;

const API_ENDPOINTS = {
  authn: {
    resend: `${API_BASE_URL}/authn/resend`,
    verify: `${API_BASE_URL}/authn/verify`
  },
  payment: {
    create: `${API_BASE_URL}/pmt/create`,
    status: `${API_BASE_URL}/pmt/status`
  },
  register: `${API_BASE_URL}/register`
};

async function encryptPayload(data, encryptPath) {
  const response = await fetch(GOD_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ data, path: encryptPath })
  });
  if (!response.ok) {
    let err;
    try { err = await response.json(); } catch { throw new Error(`Encryption failed (${response.status}).`); }
    throw new Error(err.error?.msg || err.msg || 'Encryption failed.');
  }
  const result = await response.json();
  return result.data.payload;
}
