#!/usr/bin/env node
/* eslint-env node */
/**
 * sync-version.cjs — 同步版本号到各版本清单文件
 *
 * 用法：node sync-version.cjs <version>
 * 覆盖：package.json / package-lock.json / src-tauri/Cargo.toml
 *       / src-tauri/tauri.conf.json / README.md 版本徽章
 */
'use strict';

const fs = require('fs');
const path = require('path');

const VERSION = process.argv[2];
if (!VERSION) {
  console.error('Usage: node sync-version.cjs <version>');
  process.exit(1);
}

const root = path.join(__dirname, '..');
const read = (f) => fs.readFileSync(path.join(root, f), 'utf8');
const write = (f, content) => fs.writeFileSync(path.join(root, f), content);

function updateJson(file) {
  const raw = read(file);
  const data = JSON.parse(raw);
  let changed = false;
  if (data.version !== VERSION) {
    data.version = VERSION;
    changed = true;
  }
  const rootPkg = data.packages && data.packages[''];
  if (rootPkg && rootPkg.version !== VERSION) {
    rootPkg.version = VERSION;
    changed = true;
  }
  if (changed) write(file, JSON.stringify(data, null, 2) + (raw.endsWith('\n') ? '\n' : ''));
  return changed;
}

function updateTauriJson(file) {
  const raw = read(file);
  const oldVersion = JSON.parse(raw).version;
  if (oldVersion === VERSION) return false;
  const next = raw.replace(`"version": "${oldVersion}"`, `"version": "${VERSION}"`);
  if (next === raw) return false;
  write(file, next);
  return true;
}

function updateText(file, pattern, replacement) {
  const raw = read(file);
  const next = raw.replace(pattern, replacement);
  if (next === raw) return false;
  write(file, next);
  return true;
}

function main() {
  const changed = [];
  if (updateJson('package.json')) changed.push('package.json');
  if (updateJson('package-lock.json')) changed.push('package-lock.json');
  if (updateText('src-tauri/Cargo.toml', /^version = ".*"$/m, `version = "${VERSION}"`)) changed.push('src-tauri/Cargo.toml');
  if (updateTauriJson('src-tauri/tauri.conf.json')) changed.push('src-tauri/tauri.conf.json');
  const badge = VERSION.replace(/-/g, '--');
  if (updateText('README.md', /(img\.shields\.io\/badge\/version-).*(-blue\.svg)/, `$1${badge}$2`)) changed.push('README.md');
  console.log(`sync version -> ${VERSION}`);
  console.log(changed.length ? `updated: ${changed.join(', ')}` : 'no changes');
}

main();
