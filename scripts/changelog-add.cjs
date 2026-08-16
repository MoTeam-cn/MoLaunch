#!/usr/bin/env node
/* eslint-env node */
/**
 * changelog-add.cjs — 向 CHANGELOG 指定区块植入条目（自动归并重复分组 / 创建缺失类型）
 * 用法：node changelog-add.cjs --section <区块> --type <类型> --entry <条目> [--file <path>] [--dry-run]
 */
'use strict';

const fs = require('fs');
const path = require('path');

const STD_ORDER = ['Added', 'Changed', 'Fixed', 'Removed'];
const TYPE_ALIAS = {
  added: 'Added', add: 'Added', new: 'Added',
  changed: 'Changed', change: 'Changed',
  fixed: 'Fixed', fix: 'Fixed',
  removed: 'Removed', remove: 'Removed',
};

function parseArgs(argv) {
  const opts = { file: path.join(__dirname, '..', 'CHANGELOG.md'), section: '', type: '', entries: [], dryRun: false };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    const next = () => argv[++i];
    switch (a) {
      case '--section': case '-s': opts.section = next(); break;
      case '--type': case '-t': opts.type = next(); break;
      case '--entry': case '-e': opts.entries.push(next()); break;
      case '--file': case '-f': opts.file = path.resolve(next()); break;
      case '--dry-run': opts.dryRun = true; break;
      case '--help': case '-h': return null;
      default: console.error(`未知参数: ${a}`); process.exit(1);
    }
  }
  return opts;
}

function normalizeType(type) {
  const key = type.toLowerCase().trim();
  return TYPE_ALIAS[key] || type.trim();
}

function findSection(lines, section) {
  const norm = section.replace(/^\[|\]$/g, '').trim();
  for (let i = 0; i < lines.length; i++) {
    const m = lines[i].match(/^##\s+\[([^\]]+)\]/);
    if (m && m[1] === norm) {
      let end = i + 1;
      while (end < lines.length && !/^##\s/.test(lines[end])) end++;
      return { start: i, end };
    }
  }
  return null;
}

function listSections(lines) {
  const names = [];
  for (const line of lines) {
    const m = line.match(/^##\s+\[([^\]]+)\]/);
    if (m) names.push(`[${m[1]}]`);
  }
  return names.join('、');
}

function parseGroups(blockLines) {
  const groups = [];
  let cur = null;
  for (const line of blockLines) {
    const m = line.match(/^###\s+(.+)$/);
    if (m) {
      cur = { title: m[1].trim(), entries: [] };
      groups.push(cur);
    } else if (cur && line.trim() !== '') {
      cur.entries.push(line);
    }
  }
  return groups;
}

function mergeGroups(groups) {
  const byTitle = new Map();
  for (const g of groups) {
    if (!byTitle.has(g.title)) {
      byTitle.set(g.title, { title: g.title, entries: [] });
    }
    byTitle.get(g.title).entries.push(...g.entries);
  }
  return [...byTitle.values()];
}

function insertPos(result, type) {
  const stdIdx = STD_ORDER.indexOf(type);
  if (stdIdx === -1) return result.length;
  for (let i = 0; i < result.length; i++) {
    const gi = STD_ORDER.indexOf(result[i].title);
    if (gi !== -1 && gi > stdIdx) return i;
  }
  return result.length;
}

function renderBlock(sectionLine, groups) {
  const out = [sectionLine, ''];
  for (const g of groups) {
    out.push(`### ${g.title}`, '');
    for (const e of g.entries) out.push(e, '');
  }
  while (out[out.length - 1] === '') out.pop();
  out.push('');
  return out;
}

function main() {
  const opts = parseArgs(process.argv.slice(2));
  if (!opts) {
    console.log('用法: node changelog-add.cjs --section <区块> --type <类型> --entry <条目> [--file <path>] [--dry-run]');
    console.log('  --section 区块名，如 Unreleased / 0.3.6-rc3（可带方括号）');
    console.log('  --type    分组标题，如 added/changed/fixed/removed（英文别名自动标准化，其他原样）');
    console.log('  --entry   条目文本，可多次传入；自动补 "- " 前缀');
    console.log('  --file    目标文件（默认项目根 CHANGELOG.md）');
    console.log('  --dry-run 仅打印结果不写文件');
    return;
  }
  if (!opts.section || !opts.type || opts.entries.length === 0) {
    console.error('缺少必填参数：--section / --type / --entry');
    process.exit(1);
  }
  if (!fs.existsSync(opts.file)) {
    console.error(`文件不存在: ${opts.file}`);
    process.exit(1);
  }
  const raw = fs.readFileSync(opts.file, 'utf8');
  const eol = raw.includes('\r\n') ? '\r\n' : '\n';
  const lines = raw.split(/\r?\n/);

  const sec = findSection(lines, opts.section);
  if (!sec) {
    console.error(`未找到区块 [${opts.section}]，现有区块：${listSections(lines)}`);
    process.exit(1);
  }
  const type = normalizeType(opts.type);
  const groups = parseGroups(lines.slice(sec.start + 1, sec.end));
  const merged = mergeGroups(groups);
  let target = merged.find((g) => g.title === type);
  if (!target) {
    target = { title: type, entries: [] };
    merged.splice(insertPos(merged, type), 0, target);
  }
  for (const e of opts.entries) target.entries.push(e.startsWith('- ') ? e : `- ${e}`);

  const newBlock = renderBlock(lines[sec.start], merged);
  const output = [...lines.slice(0, sec.start), ...newBlock, ...lines.slice(sec.end)].join(eol);

  if (opts.dryRun) {
    console.log(output);
    return;
  }
  fs.writeFileSync(opts.file, output);
  console.log(`[${opts.section}] ### ${type} +${opts.entries.length} 条`);
}

main();
