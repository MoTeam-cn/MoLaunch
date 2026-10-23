#!/usr/bin/env node
/* eslint-env node */
/**
 * generate-update-manifest.cjs — 生成 GitHub 更新源清单（latest.json）
 *
 * 用法: node generate-update-manifest.cjs <version> <repo> <artifacts_dir> <out_file> [notes_file]
 *   version        semver（不带 v 前缀，如 0.4.0 / 0.4.1-beta.1）
 *   repo           GitHub 仓库全名（如 MoTeam-cn/MoLaunch）
 *   artifacts_dir  release job 下载下来的产物根目录（含 windows/ macos/ linux/）
 *   out_file       输出清单路径
 *   notes_file     可选，更新说明（markdown）文件路径
 * 环境变量: FORCE_UPDATE=true 时把清单标记为强制更新
 * 标准输出: 仅打印通道名（stable / beta / alpha），供 workflow 读取
 */

'use strict';

const fs = require('fs');
const path = require('path');

const [, , VERSION, REPO, ARTIFACTS_DIR, OUT_FILE, NOTES_FILE] = process.argv;

if (!VERSION || !REPO || !ARTIFACTS_DIR || !OUT_FILE) {
  console.error('Usage: node generate-update-manifest.cjs <version> <repo> <artifacts_dir> <out_file> [notes_file]');
  process.exit(1);
}

/** 版本后缀 → 通道（与 ci-upload.cjs / 启动器 channel_name 保持一致：rc 与 beta 同属 beta） */
function resolveChannel(version) {
  const suffix = (version.split('-')[1] || '').replace(/[\d.]+$/, '').toLowerCase();
  if (!suffix) return 'stable';
  if (suffix.startsWith('rc')) return 'beta';
  if (suffix.startsWith('beta')) return 'beta';
  if (suffix.startsWith('alpha') || suffix.startsWith('dev')) return 'alpha';
  if (suffix.startsWith('canary') || suffix.startsWith('nightly')) return 'alpha';
  return 'alpha';
}

/** 目录内按后缀找文件（可排除指定后缀） */
function findOne(dir, predicate) {
  let entries;
  try {
    entries = fs.readdirSync(dir);
  } catch {
    return null;
  }
  const hit = entries.find((name) => predicate(name) && fs.statSync(path.join(dir, name)).isFile());
  return hit ? path.join(dir, hit) : null;
}

/** 读取签名内容（缺失返回空串并告警，不阻断发版） */
function readSignature(file) {
  if (!file) return '';
  const sigPath = file + '.sig';
  if (!fs.existsSync(sigPath)) {
    console.error('[manifest] 警告：缺少签名文件 ' + sigPath + '，清单中不写入 signature');
    return '';
  }
  return fs.readFileSync(sigPath, 'utf8').trim();
}

/** 由文件名推断架构（缺省按各平台默认） */
function inferArch(file, fallback) {
  const lower = file.toLowerCase();
  if (lower.includes('aarch64') || lower.includes('arm64')) return 'aarch64';
  if (lower.includes('x86_64') || lower.includes('x64')) return 'x86_64';
  return fallback;
}

/** 组装单个平台的下载信息 */
function buildPlatform(file, os, fallbackArch) {
  if (!file) return null;
  const name = path.basename(file);
  const arch = inferArch(name, fallbackArch);
  const key = os === 'macos' ? 'darwin-' + arch : os + '-' + arch;
  const signature = readSignature(file);
  const entry = {
    url: 'https://github.com/' + REPO + '/releases/download/v' + VERSION + '/' + encodeURIComponent(name),
  };
  if (signature) entry.signature = signature;
  return [key, entry];
}

const CHANNEL = resolveChannel(VERSION);

const candidates = [
  buildPlatform(
    findOne(path.join(ARTIFACTS_DIR, 'windows'), (n) => n.endsWith('.exe') && !n.endsWith('-setup.exe')),
    'windows',
    'x86_64',
  ),
  buildPlatform(
    findOne(path.join(ARTIFACTS_DIR, 'macos'), (n) => n.endsWith('.app.tar.gz')),
    'macos',
    'aarch64',
  ),
  buildPlatform(
    findOne(path.join(ARTIFACTS_DIR, 'linux'), (n) => n.endsWith('.AppImage')),
    'linux',
    'x86_64',
  ),
].filter(Boolean);

if (candidates.length === 0) {
  console.error('[manifest] 未在 ' + ARTIFACTS_DIR + ' 下找到任何可分发产物，终止生成清单');
  process.exit(1);
}

const platforms = Object.fromEntries(candidates);

const notes = NOTES_FILE && fs.existsSync(NOTES_FILE) ? fs.readFileSync(NOTES_FILE, 'utf8').trim() : '';

const manifest = {
  schema: 1,
  generated_at: new Date().toISOString(),
  channels: {
    [CHANNEL]: {
      version: VERSION,
      notes,
      force_update: process.env.FORCE_UPDATE === 'true',
      platforms,
    },
  },
};

fs.mkdirSync(path.dirname(OUT_FILE), { recursive: true });
fs.writeFileSync(OUT_FILE, JSON.stringify(manifest, null, 2) + '\n', 'utf8');

console.error(
  '[manifest] 已生成 ' + OUT_FILE + '（channel=' + CHANNEL + '，平台 ' + Object.keys(platforms).join(', ') + '）',
);
process.stdout.write(CHANNEL);
