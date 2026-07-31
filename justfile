default: dev

build:
  npm install
  npm run build

dev:
  npm install
  npm run build && node dev-server.js

clean:
  rm -rf dist node_modules
