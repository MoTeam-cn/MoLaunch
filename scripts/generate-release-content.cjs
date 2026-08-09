#!/usr/bin/env node
/* eslint-env node */
/**
 * 生成 GitHub Release 分类内容
 * 用法：node generate-release-content.cjs <prev_tag> <repo_url> <repo> [head_sha]
 * 环境变量：GITHUB_TOKEN 调 compare API 拉取协作者头像；GITHUB_OUTPUT 存在则写入 stdout
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

const esc = (s) =>
  s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

const avatarSrc = (p) => {
  if (p.avatarUrl) return p.avatarUrl.split('?')[0];
  if (p.login) return `https://avatars.githubusercontent.com/${p.login}`;
  if (p.email) {
    const hash = crypto.createHash('md5').update(p.email.trim().toLowerCase()).digest('hex');
    return `https://www.gravatar.com/avatar/${hash}?d=identicon`;
  }
  return '';
};

const avatarImg = (src, size) => {
  const origin = encodeURIComponent(src.replace(/^https?:\/\//, ''));
  return `https://images.weserv.nl/?url=${origin}&w=${size}&h=${size}&fit=cover&mask=circle`;
};

function gitLog() {
  const range = PREV_TAG ? `${PREV_TAG}..${headRef()}` : headRef();
  const limit = PREV_TAG ? '' : ' | head -50';
  const out = run(`git log ${range} --no-merges --format=%s%x09%h%x09%H%x09%ae%x09%an${limit}`);
  return out ? out.split('\n') : [];
}

const reSubject = /^([a-z]+)(?:\(([^)]+)\))?(!?[a-z]*):\s?(.*)$/i;

const TYPE_GROUPS = [
  { types: ['feat', 'feature'], key: 'FEATURES', header: '### 新增内容' },
  { types: ['fix', 'bugfix'], key: 'FIXES', header: '### 修复' },
  { types: ['perf'], key: 'OTHERS', header: '### 性能优化' },
  { types: ['refactor'], key: 'OTHERS', header: '### 重构' },
  { types: ['test', 'tests'], key: 'OTHERS', header: '### 测试' },
  { types: ['build', 'ci'], key: 'OTHERS', header: '### 构建系统' },
  { types: ['doc', 'docs'], key: 'OTHERS', header: '### 文档' },
  { types: ['style'], key: 'OTHERS', header: '### 代码风格' },
  { types: ['chore'], key: 'OTHERS', header: '### 杂项' }
];

function classify(authors) {
  const buckets = { NOTES: [], BREAKING: [], FEATURES: [], FIXES: [], OTHERS: [] };
  const others = new Map();
  const byline = (email, name) => {
    const key = (email || '').trim().toLowerCase();
    if (!key) return '';
    const p = authors.get(key);
    if (p && p.login) return `*(commit by [@${p.login}](https://github.com/${p.login}))*`;
    return `*(commit by ${esc(name)})*`;
  };
  let breakingShas = new Set();
  try {
    const range = PREV_TAG ? `${PREV_TAG}..${headRef()}` : headRef();
    breakingShas = new Set(
      run(`git log ${range} --no-merges --grep=BREAKING -i --format=%H`)
        .split('\n').map((s) => s.trim()).filter(Boolean)
    );
  } catch (e) {
    console.warn(`[generate-release-content] BREAKING CHANGE 检测失败: ${e.message}`);
  }
  for (const line of gitLog()) {
    if (!line.trim()) continue;
    const [subject, short, full, email, authorName] = line.split('\t');
    if (subject.startsWith('note:')) {
      const note = subject.startsWith('note: ') ? subject.slice(6) : subject;
      buckets.NOTES.push(`- ${stripCi(note)}`);
      continue;
    }
    const m = subject.match(reSubject);
    const type = m ? m[1].toLowerCase() : '';
    const scope = m && m[2] ? `**${m[2]}**: ` : '';
    const cleanSubject = m ? m[4] : subject;
    const entry = `- [\`${short}\`](${REPO_URL}/commit/${full}) - ${scope}${stripCi(cleanSubject)} ${byline(email, authorName)}`;
    const bangIdx = subject.indexOf('!');
    const isBreaking = breakingShas.has(full) || (bangIdx > 0 && /![\s:]/.test(subject.slice(bangIdx, bangIdx + 2)));
    if (isBreaking) buckets.BREAKING.push(entry);
    const group = TYPE_GROUPS.find((g) => g.types.includes(type));
    if (!group) {
      if (!others.has('### 其他')) others.set('### 其他', []);
      others.get('### 其他').push(entry);
    } else if (group.key === 'FEATURES') {
      buckets.FEATURES.push(entry);
    } else if (group.key === 'FIXES') {
      buckets.FIXES.push(entry);
    } else {
      if (!others.has(group.header)) others.set(group.header, []);
      others.get(group.header).push(entry);
    }
  }
  for (const [header, entries] of others) buckets.OTHERS.push(`${header}\n${entries.join('\n')}`);
  return buckets;
}

async function fetchAuthors() {
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
  if (!authors.size) return authors;
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
  return authors;
}

function contributorsBlock(authors) {
  if (!authors.size) return '';
  const avatar = (p) => {
    const src = avatarSrc(p);
    if (!src) return '';
    const label = p.login ? `@${p.login}` : p.name;
    const img = `<img src="${avatarImg(src, 48)}" width="48" height="48" alt="${esc(label)}" />`;
    return p.login ? `<a href="https://github.com/${p.login}">${img}</a>` : img;
  };
  return [...authors.values()].map(avatar).filter(Boolean).join(' ');
}

async function main() {
  const authors = await fetchAuthors();
  const b = classify(authors);
  const blocks = {};
  blocks.NOTES = b.NOTES.join('\n');
  blocks.BREAKING = b.BREAKING.join('\n');
  blocks.FEATURES = b.FEATURES.join('\n');
  blocks.FIXES = b.FIXES.join('\n');
  blocks.OTHERS = b.OTHERS.join('\n');
  if (blocks.NOTES) blocks.NOTES = `###### 作者的话\n${blocks.NOTES}`;
  if (blocks.BREAKING) blocks.BREAKING = `### 破坏性变更\n${blocks.BREAKING}`;
  if (blocks.FEATURES) blocks.FEATURES = `### 新增内容\n${blocks.FEATURES}`;
  if (blocks.FIXES) blocks.FIXES = `### 修复\n${blocks.FIXES}`;
  const list = contributorsBlock(authors);
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
