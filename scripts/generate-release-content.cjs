#!/usr/bin/env node
/* eslint-env node */
/**
 * generate-release-content.cjs — 生成 GitHub Release 分类内容
 *
 * 用法：node generate-release-content.cjs <prev_tag> <repo_url> <repo> [head_sha]
 * 环境变量：GITHUB_TOKEN 调 compare API 拉取协作者头像；GITHUB_OUTPUT 存在则写入，否则打印 stdout
 * 输出块：NOTES（作者的话）/ FEATURES / FIXES / OTHERS / CONTRIBUTORS
 */
'use strict';

const { execSync } = require('child_process');
const fs = require('fs');
const crypto = require('crypto');

const args = process.argv.slice(2);
if (args.length < 3) {
  console.error('Usage: node generate-release-content.cjs <prev_tag> <repo_url> <repo> [head_sha]');
  process.exit(1);
}
const PREV_TAG = args[0];
const REPO_URL = args[1];
const REPO = args[2];
const HEAD_SHA = args[3] || '';
const headRef = () => HEAD_SHA || 'HEAD';
const TOKEN = process.env.GITHUB_TOKEN || '';
const OUTPUT_FILE = process.env.GITHUB_OUTPUT || '';

const run = (cmd) =>
  execSync(cmd, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 }).trim();

const stripCi = (s) => (s.endsWith(' !c') ? s.slice(0, -3) : s);

function gitLog() {
  const range = PREV_TAG ? `${PREV_TAG}..${headRef()}` : headRef();
  const limit = PREV_TAG ? '' : ' | head -50';
  const out = run(`git log ${range} --no-merges --format=%s%x09%h%x09%H${limit}`);
  return out ? out.split('\n') : [];
}

function classify() {
  const buckets = { NOTES: [], FEATURES: [], FIXES: [], OTHERS: [] };
  for (const line of gitLog()) {
    if (!line.trim()) continue;
    const [subject, short, full] = line.split('\t');
    if (subject.startsWith('note:')) {
      const note = subject.startsWith('note: ') ? subject.slice(6) : subject;
      buckets.NOTES.push(`- ${stripCi(note)}`);
    } else if (subject.startsWith('feat:') || subject.startsWith('feat(')) {
      buckets.FEATURES.push(`- ${stripCi(subject)} ([${short}](${REPO_URL}/commit/${full}))`);
    } else if (subject.startsWith('fix:') || subject.startsWith('fix(')) {
      buckets.FIXES.push(`- ${stripCi(subject)} ([${short}](${REPO_URL}/commit/${full}))`);
    } else {
      buckets.OTHERS.push(`- ${stripCi(subject)} ([${short}](${REPO_URL}/commit/${full}))`);
    }
  }
  return buckets;
}

async function contributors() {
  const headSha = run(`git rev-parse ${headRef()}`);
  let baseSha;
  try {
    baseSha = PREV_TAG
      ? run(`git rev-list -n 1 ${PREV_TAG}`)
      : run(`git rev-list --max-parents=0 ${headRef()}`);
  } catch {
    baseSha = run(`git rev-list --max-parents=0 ${headRef()}`);
  }
  const range = PREV_TAG ? `${PREV_TAG}..${headRef()}` : headRef();
  const authors = new Map();
  for (const line of run(`git log ${range} --no-merges --format=%an%x09%ae`).split('\n')) {
    if (!line.trim()) continue;
    const [name, email] = line.split('\t');
    const key = (email || name).trim().toLowerCase();
    if (!authors.has(key)) {
      authors.set(key, { name: name.trim(), email: (email || '').trim(), login: '', avatarUrl: '' });
    }
  }
  if (!authors.size) return '';
  let commits = [];
  try {
    const url = `https://api.github.com/repos/${REPO}/compare/${baseSha}...${headSha}`;
    const res = await fetch(url, {
      headers: { Accept: 'application/vnd.github+json', ...(TOKEN && { Authorization: `Bearer ${TOKEN}` }) },
    });
    if (res.ok) commits = (await res.json()).commits || [];
    else console.warn(`[generate-release-content] compare API ${res.status} ${res.statusText}，无法解析 GitHub 登录名`);
  } catch (e) {
    console.warn(`[generate-release-content] compare API 调用失败，无法解析 GitHub 登录名: ${e.message}`);
  }
  for (const c of commits) {
    const gitAuthor = (c.commit || {}).author || {};
    const email = (gitAuthor.email || '').trim().toLowerCase();
    const entry = email ? authors.get(email) : null;
    if (!entry || !c.author) continue;
    if (c.author.login) entry.login = c.author.login;
    if (c.author.avatar_url) entry.avatarUrl = c.author.avatar_url;
  }
  const avatar = (p) => {
    const { name, email, login, avatarUrl } = p;
    const label = login ? `@${login}` : name;
    let src = avatarUrl;
    if (!src && login) src = `https://avatars.githubusercontent.com/${login}?s=64`;
    if (!src && email) {
      const hash = crypto.createHash('md5').update(email.toLowerCase()).digest('hex');
      src = `https://www.gravatar.com/avatar/${hash}?s=64&d=identicon`;
    }
    if (!src) return name;
    const href = login ? `https://github.com/${login}` : '#';
    const srcAttr = /\bs=\d+/.test(src) ? src : `${src}${src.includes('?') ? '&' : '?'}s=64`;
    return `<a href="${href}" title="${label}"><img src="${srcAttr}" width="48" height="48" alt="${label}" /></a>`;
  };
  return [...authors.values()].map(avatar).join(' ');
}

async function main() {
  const b = classify();
  const blocks = {};
  blocks.NOTES = b.NOTES.join('\n');
  blocks.FEATURES = b.FEATURES.join('\n');
  blocks.FIXES = b.FIXES.join('\n');
  blocks.OTHERS = b.OTHERS.join('\n');
  if (blocks.NOTES) blocks.NOTES = `###### 作者的话\n${blocks.NOTES}`;
  if (blocks.FEATURES) blocks.FEATURES = `### 新增内容\n${blocks.FEATURES}`;
  if (blocks.FIXES) blocks.FIXES = `### 修复\n${blocks.FIXES}`;
  if (blocks.OTHERS) blocks.OTHERS = `### 其他\n${blocks.OTHERS}`;
  const list = await contributors();
  blocks.CONTRIBUTORS = list
    ? `### 协作者\n\n感谢以下协作者对本阶段的贡献：\n\n${list}`
    : '';
  if (OUTPUT_FILE) {
    let buf = '';
    for (const [k, v] of Object.entries(blocks)) buf += `${k}<<EOF\n${v}\nEOF\n`;
    fs.appendFileSync(OUTPUT_FILE, buf);
  } else {
    for (const [k, v] of Object.entries(blocks)) console.log(`===== ${k} =====\n${v}`);
  }
}

main().catch((e) => {
  console.error(e.message);
  process.exit(1);
});
