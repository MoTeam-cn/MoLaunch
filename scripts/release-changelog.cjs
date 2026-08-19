#!/usr/bin/env node
/* eslint-env node */
/**
 * release-changelog.cjs — 发布前整理 CHANGELOG：[Unreleased] 合并重复二级标题并落版本号
 *
 * 用法：node release-changelog.cjs <version>
 * 示例：node release-changelog.cjs 0.3.7-rc1
 * 效果：将 [Unreleased] 下重复的 ### Added / Fixed / Changed 分组合并到首次出现位置，
 *       标题替换为 `## [<version>] - <今天日期>`，其余版本块不动。
 */
'use strict';

const fs = require('fs');
const path = require('path');

const VERSION = process.argv[2];
if (!VERSION) {
  console.error('Usage: node release-changelog.cjs <version>');
  process.exit(1);
}

const file = path.join(__dirname, '..', 'CHANGELOG.md');
const raw = fs.readFileSync(file, 'utf8');

// 定位 [Unreleased] 块（到下一个 `## [` 版本标题为止）
const start = raw.indexOf('## [Unreleased]');
if (start === -1) {
  console.error('CHANGELOG.md 中未找到 [Unreleased] 块');
  process.exit(1);
}
const next = raw.indexOf('\n## [', start + 1);
const end = next === -1 ? raw.length : next;
const block = raw.slice(start, end);

// 解析二级标题分组，重复标题合并到首次出现位置（Map 保持插入顺序）
const groups = new Map();
let current = null;
for (const line of block.split('\n')) {
  const m = line.match(/^### (.+)$/);
  if (m) {
    current = m[1];
    if (!groups.has(current)) groups.set(current, []);
    continue;
  }
  if (current && line.trim()) {
    groups.get(current).push(line);
  }
}

// 生成新块：版本标题 + 合并后的分组
const date = new Date().toISOString().slice(0, 10);
const out = [`## [${VERSION}] - ${date}`];
for (const [title, items] of groups) {
  out.push('', `### ${title}`, '');
  items.forEach((item, i) => {
    out.push(item);
    if (i < items.length - 1) out.push('');
  });
}
out.push('');

fs.writeFileSync(file, raw.slice(0, start) + out.join('\n') + raw.slice(end));
console.log(`CHANGELOG [Unreleased] -> [${VERSION}] - ${date}`);
console.log(`merged groups: ${[...groups.keys()].join(', ')}`);