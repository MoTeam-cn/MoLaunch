#!/usr/bin/env node
/* eslint-env node */
/**
 * quark-upload.cjs — 把发布产物上传到夸克网盘
 *
 * 用法: node quark-upload.cjs <dir> [--folder <fid>] [--check] [--dry-run]
 *   dir         待上传目录（目录内的普通文件逐个上传，不递归）
 *   --folder    目标目录 fid（夸克网盘网页地址 list#/list/all/<fid> 里的那段；缺省 0 = 根目录）
 *   --check     只校验 Cookie 是否有效，不上传
 *   --dry-run   只打印将要上传的文件，不发任何请求
 *
 * 环境变量: QUARK_COOKIE（必填，浏览器登录 pan.quark.cn 后整段复制 Cookie）
 *
 * 说明：夸克网盘没有开放 API，这里用的是网页端私有接口（pre → auth → PUT 分片 → commit）。
 * 接口随时可能变；Cookie 也会过期，失效时本脚本会明确报错而不是静默失败。
 */

'use strict';

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const API = 'https://drive-pc.quark.cn/1/clouddrive';
const REFERER = 'https://pan.quark.cn';
const USER_AGENT =
  'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36';
const OSS_UA = 'aliyun-sdk-js/6.6.1 Chrome 98.0.4758.80 on Windows 10 64-bit';

const args = process.argv.slice(2);
const DIR = args.find((a) => !a.startsWith('--'));
const FLAG = (name) => args.includes('--' + name);
const OPT = (name, fallback) => {
  const i = args.indexOf('--' + name);
  return i >= 0 && args[i + 1] ? args[i + 1] : fallback;
};

if (!DIR) {
  console.error('用法: node quark-upload.cjs <dir> [--folder <fid>] [--check] [--dry-run]');
  process.exit(1);
}

const FOLDER = OPT('folder', process.env.QUARK_FID || '0');
const DRY_RUN = FLAG('dry-run');
const CHECK_ONLY = FLAG('check');
const COOKIE = process.env.QUARK_COOKIE || '';

/** Cookie 会在响应里轮换（__puus / __pus），必须合并回去，否则中途掉登录 */
let cookie = COOKIE;

function fail(msg) {
  console.error('::error::' + msg);
  process.exit(1);
}

function mergeCookies(headers) {
  const list = typeof headers.getSetCookie === 'function' ? headers.getSetCookie() : [];
  for (const raw of list) {
    const pair = raw.split(';')[0];
    const eq = pair.indexOf('=');
    if (eq <= 0) continue;
    const name = pair.slice(0, eq).trim();
    const value = pair.slice(eq + 1).trim();
    if (name !== '__puus' && name !== '__pus') continue;
    const re = new RegExp('(^|;\\s*)' + name + '=[^;]*');
    cookie = re.test(cookie) ? cookie.replace(re, '$1' + name + '=' + value) : cookie + '; ' + name + '=' + value;
  }
}

async function api(pathname, body, method = 'POST') {
  const url = API + pathname + (pathname.includes('?') ? '&' : '?') + 'pr=ucpro&fr=pc';
  const resp = await fetch(url, {
    method,
    headers: {
      Cookie: cookie,
      Accept: 'application/json, text/plain, */*',
      'Content-Type': 'application/json',
      Referer: REFERER,
      'User-Agent': USER_AGENT,
    },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  mergeCookies(resp.headers);
  const text = await resp.text();
  let json;
  try {
    json = JSON.parse(text);
  } catch {
    throw new Error('接口返回非 JSON（HTTP ' + resp.status + '）：' + text.slice(0, 200));
  }
  if (!resp.ok || (json.code !== 0 && json.status !== 200)) {
    throw new Error('接口失败 ' + pathname + '（HTTP ' + resp.status + '）：' + (json.message || text.slice(0, 200)));
  }
  return json;
}

/** 校验 Cookie：能读到 config 就说明登录态有效 */
async function checkCookie() {
  try {
    await api('/config', undefined, 'GET');
    console.log('Cookie 有效');
    return true;
  } catch (e) {
    fail('夸克网盘 Cookie 失效（' + e.message + '）。请重新登录 pan.quark.cn 复制 Cookie 并更新 secret QUARK_COOKIE');
  }
}

function guessMime(file) {
  const ext = path.extname(file).toLowerCase();
  if (ext === '.zip') return 'application/zip';
  if (ext === '.exe') return 'application/octet-stream';
  if (ext === '.gz') return 'application/gzip';
  return 'application/octet-stream';
}

/** 单文件上传：pre → 逐分片 auth+PUT → commit */
async function uploadFile(file) {
  const name = path.basename(file);
  const data = fs.readFileSync(file);
  const size = data.length;
  const mime = guessMime(file);

  const pre = await api('/file/upload/pre', {
    ccp_hash_update: true,
    dir_name: '',
    file_name: name,
    format_type: mime,
    l_created_at: Date.now(),
    l_updated_at: Date.now(),
    pdir_fid: FOLDER,
    size,
  });
  const d = pre.data || {};
  const partSize = (pre.metadata && pre.metadata.part_size) || size;
  const uploadId = d.upload_id;
  const objKey = d.obj_key;
  const bucket = d.bucket;
  const ossHost = 'https://' + bucket + '.' + String(d.upload_url || '').replace(/^https:\/\//, '');
  const ossUrl = ossHost + '/' + objKey;

  console.log('  pre: task=' + d.task_id + ' 分片=' + partSize + 'B 共 ' + Math.ceil(size / partSize) + ' 片');

  const etags = [];
  const total = Math.max(1, Math.ceil(size / partSize));
  for (let i = 0; i < total; i++) {
    const partNumber = i + 1;
    const chunk = data.subarray(i * partSize, Math.min(size, (i + 1) * partSize));
    const timeStr = new Date().toUTCString();
    const authMeta =
      'PUT\n\n' +
      mime +
      '\n' +
      timeStr +
      '\n' +
      'x-oss-date:' + timeStr + '\n' +
      'x-oss-user-agent:' + OSS_UA + '\n' +
      '/' + bucket + '/' + objKey + '?partNumber=' + partNumber + '&uploadId=' + uploadId;

    const auth = await api('/file/upload/auth', {
      auth_info: d.auth_info,
      auth_meta: authMeta,
      task_id: d.task_id,
    });

    const putResp = await fetch(ossUrl + '?partNumber=' + partNumber + '&uploadId=' + uploadId, {
      method: 'PUT',
      headers: {
        Authorization: auth.data.auth_key,
        'Content-Type': mime,
        Referer: REFERER,
        'x-oss-date': timeStr,
        'x-oss-user-agent': OSS_UA,
      },
      body: chunk,
    });
    if (!putResp.ok) {
      throw new Error('分片 ' + partNumber + ' 上传失败（HTTP ' + putResp.status + '）：' + (await putResp.text()).slice(0, 200));
    }
    etags.push(putResp.headers.get('etag') || '');
    process.stdout.write('\r  分片 ' + partNumber + '/' + total + ' 完成');
  }
  if (total > 0) process.stdout.write('\n');

  let xml = '<?xml version="1.0" encoding="UTF-8"?>\n<CompleteMultipartUpload>\n';
  etags.forEach((etag, i) => {
    xml += '<Part>\n<PartNumber>' + (i + 1) + '</PartNumber>\n<ETag>' + etag + '</ETag>\n</Part>\n';
  });
  xml += '</CompleteMultipartUpload>';

  const contentMd5 = crypto.createHash('md5').update(xml).digest('base64');
  const callbackBase64 = Buffer.from(JSON.stringify(d.callback || {})).toString('base64');
  const commitTime = new Date().toUTCString();
  const commitAuthMeta =
    'POST\n' +
    contentMd5 +
    '\napplication/xml\n' +
    commitTime +
    '\nx-oss-callback:' + callbackBase64 +
    '\nx-oss-date:' + commitTime +
    '\nx-oss-user-agent:' + OSS_UA +
    '\n/' + bucket + '/' + objKey + '?uploadId=' + uploadId;

  const commitAuth = await api('/file/upload/auth', {
    auth_info: d.auth_info,
    auth_meta: commitAuthMeta,
    task_id: d.task_id,
  });

  const commitResp = await fetch(ossUrl + '?uploadId=' + uploadId, {
    method: 'POST',
    headers: {
      Authorization: commitAuth.data.auth_key,
      'Content-MD5': contentMd5,
      'Content-Type': 'application/xml',
      Referer: REFERER,
      'x-oss-callback': callbackBase64,
      'x-oss-date': commitTime,
      'x-oss-user-agent': OSS_UA,
    },
    body: xml,
  });
  if (!commitResp.ok) {
    throw new Error('合并分片失败（HTTP ' + commitResp.status + '）：' + (await commitResp.text()).slice(0, 300));
  }
  console.log('  ✓ 已上传 ' + name + '（' + (size / 1048576).toFixed(1) + ' MB，fid=' + (d.fid || '-') + '）');
}

async function main() {
  if (!fs.existsSync(DIR)) fail('目录不存在: ' + DIR);
  const files = fs
    .readdirSync(DIR)
    .filter((n) => fs.statSync(path.join(DIR, n)).isFile())
    .map((n) => path.join(DIR, n))
    .sort();

  if (files.length === 0) fail('目录内没有可上传文件: ' + DIR);

  console.log('待上传 ' + files.length + ' 个文件 → 夸克网盘目录 fid=' + FOLDER);
  for (const f of files) console.log('  · ' + path.basename(f) + ' (' + (fs.statSync(f).size / 1048576).toFixed(1) + ' MB)');

  if (DRY_RUN) {
    console.log('（--dry-run：不发请求）');
    return;
  }
  if (!COOKIE) fail('缺少环境变量 QUARK_COOKIE（浏览器登录 pan.quark.cn 后复制整段 Cookie）');

  await checkCookie();
  if (CHECK_ONLY) return;

  for (const f of files) {
    console.log('上传 ' + path.basename(f));
    await uploadFile(f);
  }
  console.log('全部上传完成');
}

main().catch((e) => fail('上传中断：' + (e && e.message ? e.message : String(e))));
