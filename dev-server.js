const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = process.env.PORT || 5000;
const HOST = process.env.HOST || '0.0.0.0';
const DIST = path.join(__dirname, 'dist');
const HTML_DIR = path.join(DIST, 'assets/html');

const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'application/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon',
  '.mp4': 'video/mp4',
  '.woff2': 'font/woff2',
};

http.createServer((req, res) => {
  const url = new URL(req.url, 'http://0.0.0.0:' + PORT);
  const pathname = url.pathname;

  if (pathname === '/payment/status') {
    const receiptPath = path.join(HTML_DIR, 'receipt.html');
    return fs.readFile(receiptPath, (err, data) => {
      if (err) {
        res.writeHead(302, { 'Location': '/receipt' + url.search });
        res.end();
        return;
      }
      res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
      res.end(data);
    });
  }

  let filePath;
  if (pathname === '/') {
    filePath = path.join(HTML_DIR, 'index.html');
  } else if (pathname.startsWith('/assets/')) {
    filePath = path.join(DIST, pathname);
  } else {
    filePath = path.join(HTML_DIR, pathname);
    if (!path.extname(filePath)) {
      const htmlPath = filePath + '.html';
      if (fs.existsSync(htmlPath)) filePath = htmlPath;
    }
  }

  const ext = path.extname(filePath);
  const contentType = MIME_TYPES[ext] || 'application/octet-stream';

  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404, { 'Content-Type': 'text/html' });
      res.end('<h1>404 Not Found</h1>');
      return;
    }
    res.writeHead(200, { 'Content-Type': contentType });
    res.end(data);
  });
}).listen(PORT, HOST, () => {
  console.log('Dev server -> port ' + PORT + ' (serving dist/)');
});
