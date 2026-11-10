import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const scratch = resolve(root, "target/.tmp/cloudflare-deploy");
const dryRun = process.argv.includes("--dry-run");
const packageVersion = "4.147.0";
const command = process.platform === "win32" ? "npm.cmd" : "npm";
const temporary = process.env.PUDDING_BUILD_TMP || resolve(root, "target/tmp");
const env = {
  ...process.env,
  TEMP: temporary,
  TMP: temporary,
  TMPDIR: temporary,
  NODE_DISABLE_COMPILE_CACHE: "1",
  npm_config_logs_max: "0",
  npm_config_cache: resolve(root, "target/npm-cache"),
  WRANGLER_CACHE_DIR: resolve(scratch, "cache"),
  WRANGLER_LOG_PATH: resolve(scratch, "wrangler.log"),
  WRANGLER_SEND_METRICS: "false",
};
function run(args, input) {
  const result = spawnSync(
    command,
    [
      "exec",
      "--yes",
      `--package=wrangler@${packageVersion}`,
      "--",
      "wrangler",
      ...args,
      "--config",
      "target/.tmp/cloudflare-deploy/wrangler.json",
    ],
    {
      stdio: ["pipe", "inherit", "inherit"],
      input,
      cwd: root,
      env,
      shell: process.platform === "win32",
    },
  );
  if (result.status !== 0) throw Error(`Cloudflare 部署失败：${result.status}`);
}
if (!dryRun && !process.env.CLOUDFLARE_API_TOKEN)
  throw Error("缺少 Cloudflare 部署 token");
if (!dryRun && (!process.env.GA_MEASUREMENT_ID || !process.env.GA_API_SECRET))
  throw Error("缺少 GA4 配置");
if (existsSync(scratch)) throw Error(`部署暂存目录已存在，请先核对归属：${scratch}`);
mkdirSync(temporary, { recursive: true });
mkdirSync(scratch, { recursive: true });
try {
  const config = JSON.parse(readFileSync(resolve(root, "cloudflare/wrangler.jsonc"), "utf8"));
  config.main = resolve(root, "cloudflare", config.main);
  // Wrangler derives its temporary bundle directory from the config location.
  writeFileSync(resolve(scratch, "wrangler.json"), JSON.stringify(config, null, 2) + "\n");
  run(["deploy", "--keep-vars", ...(dryRun ? ["--dry-run", "--outdir", "target/.tmp/cloudflare-deploy/bundle"] : [])]);
  if (!dryRun) run(
    ["secret", "bulk"],
    JSON.stringify({
      GA_MEASUREMENT_ID: process.env.GA_MEASUREMENT_ID,
      GA_API_SECRET: process.env.GA_API_SECRET,
    }),
  );
} finally {
  const cleanup = spawnSync(process.platform === "win32" ? "powershell.exe" : "pwsh", [
    "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", resolve(root, "scripts/remove-build-output.ps1"),
    "-Path", scratch, "-Json",
  ], { cwd: root, env, encoding: "utf8", windowsHide: true });
  if (cleanup.status !== 0 || existsSync(scratch)) {
    console.error(cleanup.stderr || cleanup.stdout || "Cloudflare 部署暂存清理失败");
    process.exitCode = 1;
  }
}
