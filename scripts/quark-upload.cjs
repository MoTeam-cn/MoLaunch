#!/usr/bin/env node
/* eslint-env node */
/**
 * quark-upload.cjs — 把发布产物上传到夸克网盘（经 api-server 中转签名）
 *
 * 用法: node quark-upload.cjs <dir> [--dry-run]
 *   dir        待上传目录（目录内普通文件逐个上传，不递归）
 *   --dry-run  只列出将要上传的文件，不发任何请求
 *
 * 环境变量:
 *   MOLAUNCH_ACTION_PUSH_KEY     MoSign-v2 HMAC 密钥（必填）
 *   MOLAUNCH_ACTION_PUSH_SERVER  api-server 地址（必填，如 https://api.example.com）
 *
 * 账号与 token 全在服务端，CI 只负责直传字节：
 *   begin（取 proof 采样范围）→ prepare（换预签名直传 URL）→ PUT 分片 → finish（合并）。
 */

'use strict';

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const args = process.argv.slice(2);
const DIR = args.find((a) => !a.startsWith('--'));
const DRY_RUN = args.includes('--dry-run');
const PUSH_KEY = process.env.MOLAUNCH_ACTION_PUSH_KEY || '';
const SERVER = (process.env.MOLAUNCH_ACTION_PUSH_SERVER || '').replace(/\/+$/, '');

if (!DIR) {
  console.error('用法: node quark-upload.cjs <dir> [--dry-run]');
  process.exit(1);
}

function fail(msg) {
  console.error('::error::' + msg);
  process.exit(1);
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** MoSign-v2 签名：string-to-sign = METHOD\\nPATH\\nTIMESTAMP\\nNONCE\\nBODY_SHA256_HEX */
function signHeaders(method, reqPath, bodyBuffer) {
  const timestamp = Math.floor(Date.now() / 1000).toString();
  const nonce = crypto.randomBytes(16).toString('hex');
  const bodySha256 = crypto.createHash('sha256').update(bodyBuffer).digest('hex');
  const stringToSign = [method, reqPath, timestamp, nonce, bodySha256].join('\n');
  const signature = crypto.createHmac('sha256', PUSH_KEY).update(stringToSign).digest('hex');
  return {
    'X-MoSign-Version': 'MoSign-v2',
    'X-MoSign-Timestamp': timestamp,
    'X-MoSign-Nonce': nonce,
    'X-MoSign-Signature': signature,
  };
}

/** 调 api-server（失败重试 3 次，指数退避） */
async function apiPost(reqPath, payload, label) {
  const body = Buffer.from(JSON.stringify(payload));
  for (let attempt = 1; ; attempt++) {
    try {
      const resp = await fetch(SERVER + reqPath, {
        method: 'POST',
        headers: Object.assign({ 'Content-Type': 'application/json' }, signHeaders('POST', reqPath, body)),
        body,
      });
      const text = await resp.text();
      let json;
      try {
        json = JSON.parse(text);
      } catch {
        throw new Error(label + ' 返回非 JSON（HTTP ' + resp.status + '）：' + text.slice(0, 200));
      }
      if (!resp.ok) throw new Error(label + ' 失败（HTTP ' + resp.status + '）：' + (json.msg || text.slice(0, 200)));
      if (json.code !== 1) throw new Error(label + ' 业务失败（code=' + json.code + '）：' + (json.msg || ''));
      return json.data;
    } catch (err) {
      if (attempt >= 3) throw err;
      const delay = Math.min(1000 * 2 ** (attempt - 1), 8000);
      console.log('::warning::' + label + ' 失败（' + err.message + '），' + delay + 'ms 后第 ' + attempt + ' 次重试...');
      await sleep(delay);
    }
  }
}

/** 直传单个分片，返回 ETag */
async function putPart(part, chunk, label) {
  const headers = {};
  for (const h of part.headers) headers[h.name] = h.value;
  for (let attempt = 1; ; attempt++) {
    try {
      const resp = await fetch(part.url, { method: 'PUT', headers, body: chunk });
      if (resp.ok) return resp.headers.get('etag') || '';
      const text = await resp.text();
      if (resp.status < 500 || attempt >= 3) {
        throw new Error(label + ' 分片 ' + part.part_number + ' 上传失败（HTTP ' + resp.status + '）：' + text.slice(0, 200));
      }
    } catch (err) {
      if (attempt >= 3) throw err;
      const delay = Math.min(1000 * 2 ** (attempt - 1), 8000);
      console.log('::warning::' + label + ' 分片 ' + part.part_number + ' 失败，' + delay + 'ms 后重试...');
      await sleep(delay);
    }
  }
}

function guessMime(file) {
  const ext = path.extname(file).toLowerCase();
  if (ext === '.zip') return 'application/zip';
  if (ext === '.gz') return 'application/gzip';
  if (ext === '.exe') return 'application/octet-stream';
  return 'application/octet-stream';
}

async function uploadFile(file) {
  const name = path.basename(file);
  const data = fs.readFileSync(file);
  const size = data.length;
  console.log('上传 ' + name + '（' + (size / 1048576).toFixed(1) + ' MB）');

  const begin = await apiPost('/v3/ci/quark/begin', { filename: name, size }, '开会话');
  const codes = (begin.ranges || []).map((r) => data.subarray(r.start, r.end).toString('base64'));
  const md5 = crypto.createHash('md5').update(data).digest('hex');
  const sha1 = crypto.createHash('sha1').update(data).digest('hex');

  const prep = await apiPost(
    '/v3/ci/quark/prepare',
    { session_id: begin.session_id, codes, md5, sha1, mime: guessMime(file) },
    '换直传地址',
  );

  if (prep.finish) {
    console.log('  秒传命中，跳过分片上传（fid=' + (prep.fid || '-') + '）');
    return;
  }

  const etags = [];
  const parts = prep.parts || [];
  let offset = 0;
  for (const part of parts) {
    const chunk = data.subarray(offset, offset + part.part_size);
    offset += part.part_size;
    const etag = await putPart(part, chunk, name);
    etags.push({ part_number: part.part_number, part_size: part.part_size, etag });
    process.stdout.write('\\r  分片 ' + part.part_number + '/' + parts.length + ' 完成');
  }
  if (parts.length > 0) process.stdout.write('\\n');

  const done = await apiPost(
    '/v3/ci/quark/finish',
    { session_id: begin.session_id, task_id: prep.task_id, parts: etags },
    '合并分片',
  );
  console.log('  ✓ 已上传 ' + name + '（fid=' + (done.fid || '-') + '）');
}

async function main() {
  if (!fs.existsSync(DIR)) fail('目录不存在: ' + DIR);
  const files = fs
    .readdirSync(DIR)
    .filter((n) => fs.statSync(path.join(DIR, n)).isFile())
    .map((n) => path.join(DIR, n))
    .sort();
  if (files.length === 0) fail('目录内没有可上传文件: ' + DIR);

  console.log('待上传 ' + files.length + ' 个文件');
  for (const f of files) console.log('  · ' + path.basename(f) + '（' + (fs.statSync(f).size / 1048576).toFixed(1) + ' MB）');

  if (DRY_RUN) {
    console.log('（--dry-run：不发请求）');
    return;
  }
  if (!PUSH_KEY) fail('缺少 MOLAUNCH_ACTION_PUSH_KEY');
  if (!SERVER) fail('缺少 MOLAUNCH_ACTION_PUSH_SERVER');

  for (const f of files) await uploadFile(f);
  console.log('全部上传完成');
}

main().catch((e) => fail('上传中断：' + (e && e.message ? e.message : String(e))));
