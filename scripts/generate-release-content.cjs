#!/usr/bin/env node
/* eslint-env node */
/**
 * generate-release-content.cjs — 生成 GitHub Release 分类内容
 *
 * 用法：node generate-release-content.cjs <prev_tag> <repo_url> <repo>
 * 环境变量：GITHUB_TOKEN 调 compare API 拉取头像；GITHUB_OUTPUT 存在则写入，否则打印 stdout
 * 输出块：NOTES（作者的话）/ FEATURES / FIXES / OTHERS / CONTRIBUTORS
 */
'use strict';

const { execSync } = require('child_process');
const crypto = require('crypto');
const fs = require('fs');

const args = process.argv.slice(2);
if (args.length < 3) {
  console.error('Usage: node generate-release-content.cjs <prev_tag> <repo_url> <repo>');
  process.exit(1);
}
const PREV_TAG = args[0];
const REPO_URL = args[1];
const REPO = args[2];
const TOKEN = process.env.GITHUB_TOKEN || '';
const OUTPUT_FILE = process.env.GITHUB_OUTPUT || '';

const run = (cmd) =>
  execSync(cmd, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 }).trim();

const stripCi = (s) => (s.endsWith(' !c') ? s.slice(0, -3) : s);

function gitLog() {
  const range = PREV_TAG ? `${PREV_TAG}..HEAD` : 'HEAD';
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

function shortlog() {
  const range = PREV_TAG ? `${PREV_TAG}..HEAD` : 'HEAD';
  const limit = PREV_TAG ? '' : ' | head -20';
  return run(`git shortlog -sn --no-merges ${range}${limit}`);
}

async function contributors() {
  if (!shortlog()) return '';
  const headSha = run('git rev-parse HEAD');
  let baseSha;
  if (PREV_TAG) {
    try {
      baseSha = run(`git rev-parse ${PREV_TAG}`);
    } catch {
      baseSha = run('git rev-list --max-parents=0 HEAD');
    }
  } else {
    baseSha = run('git rev-list --max-parents=0 HEAD');
  }
  let commits = [];
  try {
    const url = `https://api.github.com/repos/${REPO}/compare/${baseSha}...${headSha}`;
    const res = await fetch(url, {
      headers: { Accept: 'application/vnd.github+json', ...(TOKEN && { Authorization: `Bearer ${TOKEN}` }) },
    });
    if (res.ok) commits = (await res.json()).commits || [];
  } catch {
    // API 不可用时回退到 shortlog 文本列表
  }
  if (!commits.length) {
    return shortlog()
      .split('\n')
      .map((l) => `- ${l}`)
      .join('\n');
  }
  const seen = new Map();
  for (const c of commits) {
    const gitAuthor = (c.commit || {}).author || {};
    const email = (gitAuthor.email || '').trim().toLowerCase();
    const name = gitAuthor.name || 'anonymous';
    const ghAuthor = c.author || {};
    const login = ghAuthor.login || (!email ? name : '');
    if (seen.has(login)) continue;
    let avatar = ghAuthor.avatar_url || '';
    if (!avatar && email) {
      const digest = crypto.createHash('md5').update(email).digest('hex');
      avatar = `https://www.gravatar.com/avatar/${digest}?d=identicon&s=96`;
    }
    seen.set(login, { avatar, name, login });
  }
  return [...seen.values()]
    .map(({ avatar, name, login }) => {
      const img = `<img src="${avatar}" width="40" height="40" style="border-radius:50%" alt="${name}" />`;
      return login.startsWith('@') || login === name ? img : `[${img}](https://github.com/${login})`;
    })
    .join('\n\n');
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
