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
 *   version        语义化版本号（如 0.2.0）
 *   platform       平台：windows | macos | linux
 *   arch           架构：x86_64 | aarch64 | i686 | armv7
 *   bundle_type    安装包类型：nsis | app | appimage | deb | rpm | dmg | msi | portable
 *   package_path   本地安装包路径（如 *-setup.exe / *.app.tar.gz / *.AppImage）
 *   sig_path       签名文件路径（.sig，tauri signer 输出的 base64 签名）
 *   release_url    GitHub Release 页面 URL
 *   release_notes  可选，更新日志（Markdown）
 *
 * 环境变量：
 *   MOLAUNCH_ACTION_PUSH_KEY  MoSign-v2 签名密钥（必填）
 *   API_BASE_URL              apiServer 基础 URL（默认 https://api.molaunch.moiu.cn）
 *
 * 流程：
 *   1. 读取 .sig 文件获取签名 base64
 *   2. 计算安装包大小和 SHA256
 *   3. POST /v3/ci/presign-upload 获取 S3 PUT presigned URL（安装包 + .sig）
 *   4. PUT 直传安装包和 .sig 到 S3
 *   5. POST /v3/ci/releases 注册版本到 apiServer
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

// ===== 主流程 =====
async function main() {
  const PACKAGE_FILENAME = path.basename(PACKAGE_PATH);
  const SIG_FILENAME = path.basename(SIG_PATH);
  const packageBuffer = fs.readFileSync(PACKAGE_PATH);
  const sigBuffer = fs.readFileSync(SIG_PATH);

  const FILE_SIZE = packageBuffer.length;
  const FILE_HASH = `sha256:${crypto.createHash('sha256').update(packageBuffer).digest('hex')}`;
  // .sig 文件内容即为 base64 签名字符串（tauri signer 输出格式），去除换行和空白
  const SIGNATURE_B64 = sigBuffer.toString('utf8').replace(/[\r\n\s]/g, '');

  console.log(`::group::上传 ${PLATFORM}/${ARCH} ${BUNDLE_TYPE} (${PACKAGE_FILENAME}, ${FILE_SIZE} bytes)`);

  // ===== Step 1: 获取 S3 预签名 PUT URL（安装包 + .sig 两个文件） =====
  const presignPath = '/v3/ci/presign-upload';
  const presignBody = Buffer.from(JSON.stringify({
    version: VERSION,
    platform: PLATFORM,
    filenames: [PACKAGE_FILENAME, SIG_FILENAME],
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

  // ===== Step 2: 上传安装包到 S3 =====
  console.log(`上传安装包: ${PACKAGE_PATH} -> S3`);
  const pkgResp = await httpRequest(pkgUpload.upload_url, {
    method: 'PUT',
    headers: {},
  }, packageBuffer);

  if (pkgResp.status < 200 || pkgResp.status >= 300) {
    console.error(`::error::安装包 S3 上传失败 (HTTP ${pkgResp.status})`);
    console.error(pkgResp.body.toString().slice(0, 500));
    process.exit(1);
  }
  console.log('安装包上传完成');

  // ===== Step 3: 上传签名文件到 S3 =====
  console.log(`上传签名文件: ${SIG_PATH} -> S3`);
  const sigResp = await httpRequest(sigUpload.upload_url, {
    method: 'PUT',
    headers: {},
  }, sigBuffer);

  if (sigResp.status < 200 || sigResp.status >= 300) {
    console.error(`::error::签名文件 S3 上传失败 (HTTP ${sigResp.status})`);
    console.error(sigResp.body.toString().slice(0, 500));
    process.exit(1);
  }
  console.log('签名文件上传完成');

  // ===== Step 4: 注册版本到 apiServer =====
  const releasePath = '/v3/ci/releases';
  const releaseBody = Buffer.from(JSON.stringify({
    version: VERSION,
    channel: 'stable',
    platform: PLATFORM,
    arch: ARCH,
    bundle_type: BUNDLE_TYPE,
    download_url: pkgUpload.download_key,
    signature: SIGNATURE_B64,
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
