const fs = require('fs');
const path = require('path');

const SRC = __dirname;
const DIST = path.join(__dirname, 'dist');

const JS_FILES = [
  'api.js',
  'register.js',
  'navbar.js',
  'sticky-scroll.js',
  'tailwind-config.js',
];

const HTML_FILES = [
  'assets/html/index.html',
  'assets/html/home.html',
  'assets/html/ctf.html',
  'assets/html/receipt.html',
  'assets/html/particle.html',
];

const PARTIAL_HTML = ['assets/html/navbar.html', 'assets/html/footer.html', 'assets/html/404.html'];

const CSS_FILES = [
  'assets/css/main.css',
  'assets/css/navbar.css',
  'assets/css/responsive.css',
];

const ASSET_DIRS = ['assets/png', 'assets/ico', 'assets/mp4'];

const OBFUSCATOR_CONFIG = {
  compact: true,
  controlFlowFlattening: true,
  controlFlowFlatteningThreshold: 0.75,
  deadCodeInjection: true,
  deadCodeInjectionThreshold: 0.2,
  debugProtection: false,
  disableConsoleOutput: true,
  identifierNamesGenerator: 'hexadecimal',
  renameGlobals: false,
  selfDefending: true,
  simplify: true,
  splitStrings: true,
  splitStringsChunkLength: 5,
  stringArray: true,
  stringArrayCallsTransform: true,
  stringArrayEncoding: ['rc4'],
  stringArrayIndexShift: true,
  stringArrayRotate: true,
  stringArrayShuffle: true,
  stringArrayWrappersCount: 2,
  stringArrayWrappersChainedCalls: true,
  stringArrayWrappersParametersMaxCount: 3,
  stringArrayWrappersType: 'function',
  stringArrayThreshold: 0.75,
  transformObjectKeys: true,
  unicodeEscapeSequence: true,
};

const INLINE_OBFUSCATOR_CONFIG = {
  ...OBFUSCATOR_CONFIG,
  controlFlowFlatteningThreshold: 0.5,
  deadCodeInjectionThreshold: 0.1,
  stringArrayThreshold: 0.5,
};

function rimraf(dir) {
  if (fs.existsSync(dir)) {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

function mkdirp(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function copyFile(src, dest) {
  mkdirp(path.dirname(dest));
  fs.copyFileSync(src, dest);
}

function copyDir(src, dest) {
  mkdirp(dest);
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyDir(srcPath, destPath);
    } else {
      copyFile(srcPath, destPath);
    }
  }
}

function obfuscateJS(code, config) {
  const JavaScriptObfuscator = require('javascript-obfuscator');
  const result = JavaScriptObfuscator.obfuscate(code, config || OBFUSCATOR_CONFIG);
  return result.getObfuscatedCode();
}

function minifyCSS(code) {
  const csso = require('csso');
  const result = csso.minify(code);
  return result.css;
}

async function minifyHTML(code) {
  const { minify } = require('html-minifier-terser');
  return minify(code, {
    collapseWhitespace: true,
    removeComments: true,
    removeRedundantAttributes: true,
    removeScriptTypeAttributes: true,
    removeStyleLinkTypeAttributes: true,
    minifyCSS: true,
    minifyJS: false,
  });
}

function extractInlineScripts(html) {
  const scripts = [];
  const regex = /<script(?![^>]*\b(?:type\s*=\s*["'](?:importmap|module)["']|src\s*=))[^>]*>([\s\S]*?)<\/script>/gi;
  let match;
  while ((match = regex.exec(html)) !== null) {
    scripts.push({
      full: match[0],
      content: match[1],
      index: match.index,
    });
  }
  return scripts;
}

async function build() {
  console.log('VOID-CLT OBFUSCATION BUILD');
  console.time('Build time');

  console.log('\n[1/6] Cleaning dist/...');
  rimraf(DIST);
  mkdirp(DIST);

  console.log('[2/6] Copying static assets...');
  for (const dir of ASSET_DIRS) {
    const src = path.join(SRC, dir);
    if (fs.existsSync(src)) {
      copyDir(src, path.join(DIST, dir));
      console.log('  ' + dir);
    }
  }

  console.log('\n[3/6] Minifying CSS...');
  for (const cssPath of CSS_FILES) {
    const src = path.join(SRC, cssPath);
    const dest = path.join(DIST, cssPath);
    if (fs.existsSync(src)) {
      const code = fs.readFileSync(src, 'utf8');
      const minified = minifyCSS(code);
      mkdirp(path.dirname(dest));
      fs.writeFileSync(dest, minified);
      const savings = ((1 - minified.length / code.length) * 100).toFixed(1);
      console.log('  ' + cssPath + ' (-' + savings + '%)');
    }
  }

  console.log('\n[4/6] Obfuscating JS files...');
  const jsFileMap = {};

  JS_FILES.forEach((file, i) => {
    const src = path.join(SRC, 'assets/js', file);
    const obfName = '_0x' + (i + 1).toString(16).padStart(2, '0') + '.js';
    const dest = path.join(DIST, 'assets/js', obfName);

    if (fs.existsSync(src)) {
      const code = fs.readFileSync(src, 'utf8');
      try {
        const obfuscated = obfuscateJS(code);
        mkdirp(path.dirname(dest));
        fs.writeFileSync(dest, obfuscated);
        console.log('  ' + file + ' -> ' + obfName + ' (' + obfuscated.length + ' bytes)');
      } catch (err) {
        console.error('  FAIL ' + file + ': ' + err.message);
        mkdirp(path.dirname(dest));
        fs.copyFileSync(src, dest);
      }
    }

    jsFileMap[file] = 'assets/js/' + obfName;
  });

  console.log('\n[5/6] Processing HTML files...');

  for (const htmlFile of HTML_FILES) {
    const srcPath = path.join(SRC, htmlFile);
    const destPath = path.join(DIST, htmlFile);

    if (!fs.existsSync(srcPath)) continue;

    let html = fs.readFileSync(srcPath, 'utf8');

    for (const [orig, obf] of Object.entries(jsFileMap)) {
      const escaped = orig.replace('.', '\\.');
      html = html.replace(
        new RegExp('src=["\']/?assets/js/' + escaped + '["\']', 'g'),
        'src="/' + obf + '"'
      );
    }

    const scripts = extractInlineScripts(html);
    if (scripts.length > 0) {
      console.log('  ' + htmlFile + ': ' + scripts.length + ' inline script(s) (skipped)');
    }

    try {
      html = await minifyHTML(html);
    } catch (err) {
      console.warn('  WARN HTML minify for ' + htmlFile + ': ' + err.message);
    }

    fs.mkdirSync(path.dirname(destPath), { recursive: true });
    fs.writeFileSync(destPath, html);
    console.log('  ' + htmlFile);
  }

  console.log('\n[6/6] Copying partial HTML files...');
  for (const htmlFile of PARTIAL_HTML) {
    const srcPath = path.join(SRC, htmlFile);
    const destPath = path.join(DIST, htmlFile);

    if (!fs.existsSync(srcPath)) continue;

    let html = fs.readFileSync(srcPath, 'utf8');
    try {
      html = await minifyHTML(html);
    } catch {}
    fs.mkdirSync(path.dirname(destPath), { recursive: true });
    fs.writeFileSync(destPath, html);
    console.log('  ' + htmlFile);
  }

  console.timeEnd('Build time');
  console.log('\nBuild complete! Output in dist/');
}

build().catch(err => {
  console.error('Build failed:', err);
  process.exit(1);
});
