#!/usr/bin/env node
/* eslint-env node */
/**
 * ci-upload.cjs — 上传 MoLaunch 安装包到 apiServer（MoSign-v2 鉴权）
 *
 * 纯 Node.js 实现，消除 shell/Node 数据传递导致的签名不一致问题：
 * - JSON.stringify() 生成 body Buffer，签名和 HTTP 请求使用同一个 Buffer
 * - 无 heredoc / curl / openssl / shell 变量展开，行为确定
 *
 * 用法：
 *   node ci-upload.cjs <version> <platform> <arch> <bundle_type> <package_path> <sig_path> <release_url> [release_notes]
 *
 * 参数：
 *   version        语义化版本号（如 0.2.0 / 0.3.1-rc1）
 *   platform       平台：windows | macos | linux
 *   arch           架构：x86_64 | aarch64 | i686 | armv7
 *   bundle_type    安装包类型：nsis | app | appimage | deb | rpm | dmg | msi | portable
 *   package_path   本地安装包路径（如 *-setup.exe / *.app.tar.gz / *.AppImage）
 *   sig_path       签名文件路径（.sig，tauri signer 输出的 base64 签名）
 *   release_url    GitHub Release tag 页面 URL（如 .../releases/tag/v0.3.0，注册版本时原样上报）
 *   release_notes  可选，更新日志（Markdown，随版本注册上报，启动器「检查更新」对话框展示）
 *
 * 环境变量：
 *   MOLAUNCH_ACTION_PUSH_KEY  MoSign-v2 签名密钥（必填）
 *   API_BASE_URL              apiServer 基础 URL（默认 https://api.molaunch.moiu.cn）
 *
 * 渠道（channel）自动推导：由 version 预发布后缀判定，并把结果收敛到服务端合法取值——
 * 服务端（api-server/src/services/updates.rs 的 VALID_CHANNELS）仅接受 stable / beta / alpha，
 * 因此 rc 归入 beta 灰度通道、canary/nightly/dev/未知 归入 alpha：
 *   无后缀→stable / -rc→beta / -beta→beta / -alpha、-dev→alpha / -canary、-nightly、未知→alpha。
 *
 * 流程：
 *   1. 读取 .sig 文件获取签名 base64
 *   2. 计算安装包大小和 SHA256
 *   3. POST /v3/ci/presign-upload 获取 S3 预签名上传 URL（安装包 + .sig，携带 sizes 供服务端判断分片）
 *   4. 上传到 S3：
 *      - 小文件（< 50MB）：单次 PUT 直传
 *      - 大文件（>= 50MB）：分片上传（按分片 PUT 直传，收集各分片 ETag）
 *   5. 若走了分片上传，POST /v3/ci/complete-upload 完成合并
 *   6. POST /v3/ci/releases 注册版本到 apiServer
 *
 * See: api-server/src/utils/mosign_v2.rs（签名协议）
 *      api-server/src/controllers/v3/ci.rs（presign_upload + create_release）
 *      api-server/src/models/updates.rs（CreateReleaseRequest 字段定义）
 */

'use strict';

const crypto = require('crypto');
const fs = require('fs');
const path = require('path');
const https = require('https');
const http = require('http');
const { URL } = require('url');

// ===== 参数解析 =====
const args = process.argv.slice(2);
if (args.length < 7) {
  console.error('Usage: node ci-upload.cjs <version> <platform> <arch> <bundle_type> <package_path> <sig_path> <release_url> [release_notes]');
  console.error('');
  console.error('  version        semver (e.g. 0.2.0)');
  console.error('  platform       windows | macos | linux');
  console.error('  arch           x86_64 | aarch64 | i686 | armv7');
  console.error('  bundle_type    nsis | app | appimage | deb | rpm | dmg | msi | portable');
  console.error('  package_path   local installer path');
  console.error('  sig_path       .sig file path (base64 signature)');
  console.error('  release_url    GitHub release URL');
  console.error('  release_notes  optional, markdown');
  process.exit(1);
}

const VERSION = args[0];
const PLATFORM = args[1];
const ARCH = args[2];
const BUNDLE_TYPE = args[3];
const PACKAGE_PATH = args[4];
const SIG_PATH = args[5];
const RELEASE_URL = args[6];
const RELEASE_NOTES = args[7] || '';

// ===== 渠道推导 =====
// 由语义化版本预发布后缀推导发布渠道，并把结果收敛到服务端合法取值（仅 stable/beta/alpha）：
//   - 无后缀           → stable（正式版）
//   - -rc             → beta（Release Candidate，归入 beta 灰度通道）
//   - -beta           → beta（内测版）
//   - -alpha / -dev   → alpha（开发版）
//   - -canary / -nightly → alpha（金丝雀/每日构建，归入 alpha 通道）
//   - 未知后缀         → alpha（防御性兜底）
function resolveChannel(version) {
  const suffix = (version.split('-')[1] || '').replace(/[\d.]+$/, '').toLowerCase();
  if (!suffix) return 'stable';
  if (suffix.startsWith('rc')) return 'beta';
  if (suffix.startsWith('beta')) return 'beta';
  if (suffix.startsWith('alpha') || suffix.startsWith('dev')) return 'alpha';
  if (suffix.startsWith('canary') || suffix.startsWith('nightly')) return 'alpha';
  return 'alpha';
}
const CHANNEL = resolveChannel(VERSION);
console.log(`渠道推导: version=${VERSION} -> channel=${CHANNEL}`);

const API_BASE_URL = process.env.API_BASE_URL || 'https://api.molaunch.moiu.cn';
const PUSH_KEY = process.env.MOLAUNCH_ACTION_PUSH_KEY;

// ===== 环境校验 =====
if (!PUSH_KEY) {
  console.error('::error::MOLAUNCH_ACTION_PUSH_KEY 环境变量未设置');
  process.exit(1);
}
if (!fs.existsSync(PACKAGE_PATH)) {
  console.error(`::error::安装包文件不存在: ${PACKAGE_PATH}`);
  process.exit(1);
}
if (!fs.existsSync(SIG_PATH)) {
  console.error(`::error::签名文件不存在: ${SIG_PATH}`);
  process.exit(1);
}

// 参数校验
if (!['windows', 'macos', 'linux'].includes(PLATFORM)) {
  console.error(`::error::platform 非法（仅 windows / macos / linux）: ${PLATFORM}`);
  process.exit(1);
}
if (!['x86_64', 'aarch64', 'i686', 'armv7'].includes(ARCH)) {
  console.error(`::error::arch 非法（仅 x86_64 / aarch64 / i686 / armv7）: ${ARCH}`);
  process.exit(1);
}
if (!['nsis', 'app', 'appimage', 'deb', 'rpm', 'dmg', 'msi', 'portable'].includes(BUNDLE_TYPE)) {
  console.error(`::error::bundle_type 非法: ${BUNDLE_TYPE}`);
  process.exit(1);
}

// ===== MoSign-v2 签名 =====
// string-to-sign = METHOD\nPATH\nTIMESTAMP\nNONCE\nBODY_SHA256_HEX
// signature = HMAC-SHA256(push_key, string_to_sign).hex()
function signRequest(method, reqPath, bodyBuffer) {
  const timestamp = Math.floor(Date.now() / 1000).toString();
  const nonce = crypto.randomBytes(16).toString('hex');
  const bodySha256 = crypto.createHash('sha256').update(bodyBuffer).digest('hex');
  const stringToSign = [method, reqPath, timestamp, nonce, bodySha256].join('\n');
  const signature = crypto.createHmac('sha256', PUSH_KEY).update(stringToSign).digest('hex');
  return { timestamp, nonce, signature };
}

// ===== HTTP 请求封装（支持 HTTPS + 自动重定向） =====
function httpRequest(targetUrl, options, body) {
  return new Promise((resolve, reject) => {
    const u = new URL(targetUrl);
    const lib = u.protocol === 'https:' ? https : http;
    const headers = Object.assign({}, options.headers || {});
    if (body) headers['Content-Length'] = Buffer.byteLength(body);

    const req = lib.request(u, { method: options.method, headers }, (res) => {
      // 处理 3xx 重定向（S3 可能返回 307 临时重定向）
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        res.resume(); // 丢弃当前响应体
        httpRequest(res.headers.location, options, body).then(resolve, reject);
        return;
      }
      const chunks = [];
      res.on('data', (chunk) => chunks.push(chunk));
      res.on('end', () => {
        resolve({ status: res.statusCode, headers: res.headers, body: Buffer.concat(chunks) });
      });
    });
    req.on('error', reject);
    if (body) req.write(body);
    req.end();
  });
}

// ===== S3 上传辅助 =====

// 单次 PUT 上传（小文件）
function uploadSingle(item, buffer, label) {
  return httpRequest(item.upload_url, { method: 'PUT', headers: {} }, buffer).then((resp) => {
    if (resp.status < 200 || resp.status >= 300) {
      const err = new Error(`${label} S3 上传失败 (HTTP ${resp.status}): ${resp.body.toString().slice(0, 500)}`);
      err.code = 'UPLOAD_FAILED';
      throw err;
    }
    return resp;
  });
}

// 分片上传（大文件）：按 part_number 顺序 PUT 各分片，收集 ETag
async function uploadMultipart(item, buffer, label) {
  const { upload_id, part_size, parts } = item.multipart;
  console.log(`${label}分片上传开始: ${parts.length} 片 x ${part_size} 字节 (upload_id=${upload_id})`);
  const uploadedParts = [];
  for (const part of parts) {
    const start = (part.part_number - 1) * part_size;
    const end = Math.min(start + part_size, buffer.length);
    const chunk = buffer.subarray(start, end);
    const resp = await httpRequest(part.upload_url, { method: 'PUT', headers: {} }, chunk);
    if (resp.status < 200 || resp.status >= 300) {
      const err = new Error(`${label} 分片 ${part.part_number} 上传失败 (HTTP ${resp.status}): ${resp.body.toString().slice(0, 500)}`);
      err.code = 'UPLOAD_FAILED';
      throw err;
    }
    const etag = resp.headers.etag;
    if (!etag) {
      const err = new Error(`${label} 分片 ${part.part_number} 响应缺少 ETag`);
      err.code = 'UPLOAD_FAILED';
      throw err;
    }
    uploadedParts.push({ part_number: part.part_number, etag });
    console.log(`  分片 ${part.part_number}/${parts.length} 完成 (${chunk.length} bytes)`);
  }
  return uploadedParts;
}

// 完成分片上传：回传 upload_id + 分片 ETag 列表，由服务端合并
async function completeMultipartUpload(item, parts) {
  const completePath = '/v3/ci/complete-upload';
  const completeBody = Buffer.from(JSON.stringify({
    upload_id: item.multipart.upload_id,
    download_key: item.download_key,
    parts,
  }));
  const completeSign = signRequest('POST', completePath, completeBody);
  console.log('完成分片上传（CompleteMultipartUpload）...');

  const resp = await httpRequest(`${API_BASE_URL}${completePath}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-MoSign-Version': 'MoSign-v2',
      'X-MoSign-Timestamp': completeSign.timestamp,
      'X-MoSign-Nonce': completeSign.nonce,
      'X-MoSign-Signature': completeSign.signature,
    },
  }, completeBody);

  let data;
  try {
    data = JSON.parse(resp.body.toString());
  } catch (e) {
    const err = new Error(`完成分片响应非 JSON（HTTP ${resp.status}）: ${resp.body.toString().slice(0, 500)}`);
    err.code = 'UPLOAD_FAILED';
    throw err;
  }

  if (data.code !== 1) {
    const err = new Error(`完成分片失败 (code=${data.code}): ${data.msg || ''}\n完整响应：${resp.body.toString()}`);
    err.code = 'UPLOAD_FAILED';
    throw err;
  }
  console.log('分片上传完成');
}

// 通用上传入口：根据服务端返回的 multipart 字段自动选择分片 / 单次 PUT
async function uploadToS3(item, buffer, label) {
  if (item.multipart && item.multipart.parts && item.multipart.parts.length > 0) {
    const parts = await uploadMultipart(item, buffer, label);
    await completeMultipartUpload(item, parts);
    return;
  }
  if (!item.upload_url) {
    const err = new Error(`${label} 预签名响应缺少 upload_url 且无分片凭证`);
    err.code = 'UPLOAD_FAILED';
    throw err;
  }
  console.log(`上传${label}: -> S3`);
  await uploadSingle(item, buffer, label);
  console.log(`${label}上传完成`);
}

// ===== 主流程 =====
async function main() {
  const PACKAGE_FILENAME = path.basename(PACKAGE_PATH);
  const SIG_FILENAME = path.basename(SIG_PATH);
  const packageBuffer = fs.readFileSync(PACKAGE_PATH);
  const sigBuffer = fs.readFileSync(SIG_PATH);

  const FILE_SIZE = packageBuffer.length;
  const FILE_HASH = `sha256:${crypto.createHash('sha256').update(packageBuffer).digest('hex')}`;
  // .sig 文件内容为标准 minisign 4 行格式（tauri signer / tauri-action 输出）。
  // 原样存储（保留换行），启动器 updater（src-tauri/updater，minisign_verify）
  // 与 tauri-plugin-updater 才能正确解析；不要去除换行/空白。
  const SIGNATURE = sigBuffer.toString('utf8');

  console.log(`::group::上传 ${PLATFORM}/${ARCH} ${BUNDLE_TYPE} (${PACKAGE_FILENAME}, ${FILE_SIZE} bytes)`);

  // ===== Step 1: 获取 S3 预签名上传 URL（安装包 + .sig 两个文件） =====
  const presignPath = '/v3/ci/presign-upload';
  const presignBody = Buffer.from(JSON.stringify({
    version: VERSION,
    platform: PLATFORM,
    filenames: [PACKAGE_FILENAME, SIG_FILENAME],
    // 各文件字节大小（与 filenames 对齐），服务端据此判断是否走分片上传
    sizes: [FILE_SIZE, sigBuffer.length],
  }));

  const presignSign = signRequest('POST', presignPath, presignBody);
  console.log('请求预签名上传 URL...');

  const presignResp = await httpRequest(`${API_BASE_URL}${presignPath}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-MoSign-Version': 'MoSign-v2',
      'X-MoSign-Timestamp': presignSign.timestamp,
      'X-MoSign-Nonce': presignSign.nonce,
      'X-MoSign-Signature': presignSign.signature,
    },
  }, presignBody);

  let presignData;
  try {
    presignData = JSON.parse(presignResp.body.toString());
  } catch (e) {
    console.error(`::error::预签名响应非 JSON（HTTP ${presignResp.status}）: ${presignResp.body.toString().slice(0, 500)}`);
    process.exit(1);
  }

  if (presignData.code !== 1) {
    console.error(`::error::预签名业务错误 (code=${presignData.code}): ${presignData.msg || ''}`);
    console.error(`完整响应：${presignResp.body.toString()}`);
    process.exit(1);
  }

  if (!presignData.data || !Array.isArray(presignData.data.uploads) || presignData.data.uploads.length === 0) {
    console.error(`::error::预签名响应格式异常: ${presignResp.body.toString()}`);
    process.exit(1);
  }

  // 构建 filename -> upload_item 映射
  const uploadMap = {};
  for (const item of presignData.data.uploads) {
    uploadMap[item.filename] = item;
  }

  const pkgUpload = uploadMap[PACKAGE_FILENAME];
  const sigUpload = uploadMap[SIG_FILENAME];
  if (!pkgUpload || !sigUpload) {
    console.error(`::error::预签名响应缺少对应文件的上传 URL`);
    console.error(`期望: ${PACKAGE_FILENAME}, ${SIG_FILENAME}`);
    console.error(`实际: ${Object.keys(uploadMap).join(', ')}`);
    process.exit(1);
  }

  console.log(`已获取预签名 URL（有效期 ${presignData.data.expires_in} 秒）`);
  console.log(`  安装包 download_key: ${pkgUpload.download_key}`);
  console.log(`  签名文件 download_key: ${sigUpload.download_key}`);

  // ===== Step 2: 上传安装包到 S3（大文件走分片上传，小文件单次 PUT） =====
  await uploadToS3(pkgUpload, packageBuffer, '安装包');

  // ===== Step 3: 上传签名文件到 S3（.sig 很小，始终单次 PUT） =====
  console.log(`上传签名文件: ${SIG_PATH} -> S3`);
  await uploadSingle(sigUpload, sigBuffer, '签名文件');
  console.log('签名文件上传完成');

  // ===== Step 4: 注册版本到 apiServer =====
  const releasePath = '/v3/ci/releases';
  const releaseBody = Buffer.from(JSON.stringify({
    version: VERSION,
    channel: CHANNEL,
    platform: PLATFORM,
    arch: ARCH,
    bundle_type: BUNDLE_TYPE,
    download_url: pkgUpload.download_key,
    signature: SIGNATURE,
    file_size: FILE_SIZE,
    file_hash: FILE_HASH,
    release_notes: RELEASE_NOTES,
    release_url: RELEASE_URL,
    rollout_pct: 100,
    force_update: false,
    min_version: '',
  }));

  const releaseSign = signRequest('POST', releasePath, releaseBody);
  console.log('注册版本到 apiServer...');

  const releaseResp = await httpRequest(`${API_BASE_URL}${releasePath}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-MoSign-Version': 'MoSign-v2',
      'X-MoSign-Timestamp': releaseSign.timestamp,
      'X-MoSign-Nonce': releaseSign.nonce,
      'X-MoSign-Signature': releaseSign.signature,
    },
  }, releaseBody);

  let releaseData;
  try {
    releaseData = JSON.parse(releaseResp.body.toString());
  } catch (e) {
    console.error(`::error::注册响应非 JSON（HTTP ${releaseResp.status}）: ${releaseResp.body.toString().slice(0, 500)}`);
    process.exit(1);
  }

  if (releaseData.code !== 1) {
    console.error(`::error::版本注册失败 (code=${releaseData.code}): ${releaseData.msg || ''}`);
    console.error(`完整响应：${releaseResp.body.toString()}`);
    process.exit(1);
  }

  console.log('::endgroup::');
  console.log(`✓ 已注册 ${PLATFORM}/${ARCH} ${BUNDLE_TYPE} v${VERSION} (id=${releaseData.data?.id})`);
}

main().catch((err) => {
  console.error(`::error::${err.message}`);
  console.error(err.stack);
  process.exit(1);
});
