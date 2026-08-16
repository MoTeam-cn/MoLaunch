// 自动更新 easytier-core 嵌入式资源（联机虚拟组网）
//
// 对比 GitHub Releases 最新 stable 与 src-tauri/build_script/easytier.rs 记录的版本，
// 不一致则下载 6 个平台 zip（依赖系统 unzip）解压替换 src-tauri/resources/easytier/{os}/{arch}/，
// 并更新版本常量；由 .github/workflows/update-easytier.yml 每日调度调用。
// 仅依赖 Node 18+ 内置 fetch，无第三方包；GITHUB_TOKEN 可选（提升 API 限额）。

const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const EASYTIER_RS = "src-tauri/build_script/easytier.rs";

// 上游 release 资产 → 项目资源目录映射（Windows arm64 资产名为 arm64，与目录 aarch64 不同）
const PLATFORMS = [
  ["windows", "x86_64", "easytier-windows-x86_64"],
  ["windows", "aarch64", "easytier-windows-arm64"],
  ["linux", "x86_64", "easytier-linux-x86_64"],
  ["linux", "aarch64", "easytier-linux-aarch64"],
  ["macos", "x86_64", "easytier-macos-x86_64"],
  ["macos", "aarch64", "easytier-macos-aarch64"],
];

function setOutput(name, value) {
  if (process.env.GITHUB_OUTPUT) {
    fs.appendFileSync(process.env.GITHUB_OUTPUT, `${name}=${value}\n`);
  }
}

// 在目录树中查找目标文件名，返回第一个匹配的完整路径
function findFile(dir, target) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      const hit = findFile(full, target);
      if (hit) return hit;
    } else if (entry.name === target) {
      return full;
    }
  }
  return null;
}

async function main() {
  const headers = {
    "User-Agent": "MoLaunch-update-bot",
    Accept: "application/vnd.github+json",
  };
  if (process.env.GITHUB_TOKEN) headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  const res = await fetch("https://api.github.com/repos/EasyTier/EasyTier/releases/latest", { headers });
  if (!res.ok) {
    throw new Error(`获取 EasyTier 最新 release 失败: ${res.status} ${await res.text()}`);
  }
  const { tag_name: tag } = await res.json();
  const version = tag.replace(/^v/, "");

  const rsContent = fs.readFileSync(EASYTIER_RS, "utf8");
  const match = rsContent.match(/const EASYTIER_VERSION: &str = "([^"]+)"/);
  if (!match) throw new Error(`无法从 ${EASYTIER_RS} 解析 EASYTIER_VERSION 常量`);
  const current = match[1];

  if (current === version) {
    console.log(`easytier-core 已是最新 v${version}，无需更新`);
    setOutput("updated", "false");
    setOutput("version", version);
    return;
  }

  console.log(`发现新版本 v${current} -> v${version}，开始下载替换...`);

  const workDir = fs.mkdtempSync(path.join(os.tmpdir(), "easytier-update-"));
  try {
    for (const [platform, arch, asset] of PLATFORMS) {
      const zipName = `${asset}-v${version}.zip`;
      const zipPath = path.join(workDir, zipName);
      const unzipDir = path.join(workDir, `${platform}-${arch}`);
      const url = `https://github.com/EasyTier/EasyTier/releases/download/${tag}/${zipName}`;

      console.log(`下载 ${zipName} ...`);
      const dl = await fetch(url);
      if (!dl.ok) throw new Error(`下载 ${zipName} 失败: ${dl.status}`);
      fs.writeFileSync(zipPath, Buffer.from(await dl.arrayBuffer()));
      fs.mkdirSync(unzipDir, { recursive: true });
      execFileSync("unzip", ["-o", zipPath, "-d", unzipDir], { stdio: "inherit" });

      const coreName = platform === "windows" ? "easytier-core.exe" : "easytier-core";
      const core = findFile(unzipDir, coreName);
      if (!core) throw new Error(`${zipName} 中未找到 ${coreName}`);

      const destDir = path.join("src-tauri/resources/easytier", platform, arch);
      fs.mkdirSync(destDir, { recursive: true });
      const coreDest = path.join(destDir, coreName);
      fs.copyFileSync(core, coreDest);
      if (platform !== "windows") fs.chmodSync(coreDest, 0o755);

      if (platform === "windows") {
        for (const dll of ["Packet.dll", "wintun.dll"]) {
          const dllFile = findFile(unzipDir, dll);
          if (dllFile) {
            fs.copyFileSync(dllFile, path.join(destDir, dll));
          } else {
            console.warn(`${zipName} 中未找到 ${dll}（上游可能调整了打包结构）`);
          }
        }
      }
      console.log(`  更新 ${platform}/${arch} 完成`);
    }

    const updated = rsContent.replace(
      /const EASYTIER_VERSION: &str = "[^"]+"/,
      `const EASYTIER_VERSION: &str = "${version}"`
    );
    fs.writeFileSync(EASYTIER_RS, updated);
    console.log(`版本常量已更新: ${EASYTIER_RS} -> v${version}`);
  } finally {
    fs.rmSync(workDir, { recursive: true, force: true });
  }

  setOutput("updated", "true");
  setOutput("version", version);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
