#!/usr/bin/env node

const { execSync, spawn } = require("child_process");
const AdmZip = require("adm-zip");
const path = require("path");
const fs = require("fs");

function getEffectiveArch() {
  const platform = process.platform;
  const nodeArch = process.arch;

  if (platform === "darwin") {
    if (nodeArch === "arm64") return "arm64";

    try {
      const translated = execSync("sysctl -in sysctl.proc_translated", {
        encoding: "utf8",
      }).trim();
      if (translated === "1") return "arm64";
    } catch {
    }
    return "x64";
  }

  if (/arm/i.test(nodeArch)) return "arm64";

  if (platform === "win32") {
    const pa = process.env.PROCESSOR_ARCHITECTURE || "";
    const paw = process.env.PROCESSOR_ARCHITEW6432 || "";
    if (/arm/i.test(pa) || /arm/i.test(paw)) return "arm64";
  }

  return "x64";
}

const platform = process.platform;
const arch = getEffectiveArch();

function getPlatformDir() {
  if (platform === "linux" && arch === "x64") return "linux-x64";
  if (platform === "linux" && arch === "arm64") return "linux-arm64";
  if (platform === "win32" && arch === "x64") return "windows-x64";
  if (platform === "win32" && arch === "arm64") return "windows-arm64";
  if (platform === "darwin" && arch === "x64") return "macos-x64";
  if (platform === "darwin" && arch === "arm64") return "macos-arm64";

  console.error(`❌ Unsupported platform: ${platform}-${arch}`);
  console.error("Supported platforms:");
  console.error("  - Linux x64");
  console.error("  - Linux ARM64");
  console.error("  - Windows x64");
  console.error("  - Windows ARM64");
  console.error("  - macOS x64 (Intel)");
  console.error("  - macOS ARM64 (Apple Silicon)");
  process.exit(1);
}

function getBinaryName() {
  return platform === "win32" ? "mcp-dev-manager.exe" : "mcp-dev-manager";
}

const platformDir = getPlatformDir();
const extractDir = path.join(__dirname, "..", "dist", platformDir);

fs.mkdirSync(extractDir, { recursive: true });

function extractAndRun() {
  const binName = getBinaryName();
  const binPath = path.join(extractDir, binName);
  const zipName = "mcp-dev-manager.zip";
  const zipPath = path.join(extractDir, zipName);

  if (fs.existsSync(binPath)) fs.unlinkSync(binPath);
  
  if (!fs.existsSync(zipPath)) {
    console.error(`❌ ${zipName} not found at: ${zipPath}`);
    console.error(`Current platform: ${platform}-${arch} (${platformDir})`);
    process.exit(1);
  }

  try {
    const zip = new AdmZip(zipPath);
    zip.extractAllTo(extractDir, true);
  } catch (err) {
    console.error("❌ Failed to extract mcp-dev-manager archive:", err.message);
    if (process.env.MCP_DEV_MANAGER_DEBUG) {
      console.error(err.stack);
    }
    process.exit(1);
  }

  if (!fs.existsSync(binPath)) {
    console.error(`❌ Extracted binary not found at: ${binPath}`);
    console.error("This usually indicates a corrupt download. Please reinstall the package.");
    process.exit(1);
  }

  if (platform !== "win32") {
    try {
      fs.chmodSync(binPath, 0o755);
    } catch { }
  }

  console.log(`🚀 Launching mcp-dev-manager...`);
  
  const args = process.argv.slice(2);
  const proc = spawn(binPath, args, { stdio: "inherit" });
  
  proc.on("exit", (code) => process.exit(code || 0));
  proc.on("error", (err) => {
    console.error("❌ Failed to start mcp-dev-manager:", err.message);
    process.exit(1);
  });

  process.on("SIGINT", () => proc.kill("SIGINT"));
  process.on("SIGTERM", () => proc.kill("SIGTERM"));
}

extractAndRun();
