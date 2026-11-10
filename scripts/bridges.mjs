import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { copyFileSync, existsSync, lstatSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const lockPath = resolve(root, 'scripts/bridges.lock.json');
const lock = JSON.parse(readFileSync(lockPath, 'utf8'));
const [action = 'status', ...options] = process.argv.slice(2);
const locked = options.includes('--locked') || process.env.GITHUB_ACTIONS === 'true';
const temporary = resolve(process.env.PUDDING_BUILD_TMP || resolve(root, 'target/tmp'));
const env = { ...process.env, TEMP: temporary, TMP: temporary, TMPDIR: temporary, NODE_DISABLE_COMPILE_CACHE: '1', GIT_TERMINAL_PROMPT: '0', GCM_INTERACTIVE: 'never' };
const statePath = resolve(root, 'target/bridge/bridge-build.json');
const ids = ['bgi', 'march7th', 'zzz', 'maaend'];
if (lock.schemaVersion !== 1 || lock.bridges.length !== ids.length || new Set(lock.bridges.map(b => b.id)).size !== ids.length) throw Error('Invalid bridge lockfile');
for (const bridge of lock.bridges) {
  if (!ids.includes(bridge.id) || bridge.path !== `bridges/${bridge.id}` || !/^https:\/\/github\.com\/Pudding-Player\/Pudding-[A-Za-z0-9-]+-Bridge\.git$/.test(bridge.repository)) throw Error('Invalid bridge repository mapping');
  if ((bridge.enabled !== undefined && typeof bridge.enabled !== 'boolean') || typeof bridge.bundled !== 'boolean' || !Array.isArray(bridge.binaries) || bridge.binaries.some(name => !/^[A-Za-z0-9_.-]+$/.test(name))) throw Error('Invalid bridge payload mapping');
}
const activeBridges = lock.bridges.filter(bridge => bridge.enabled !== false);
// A CI secret is passed only through Git's process environment, never persisted in remotes.
const gitEnv = { ...env };
if (gitEnv.PUDDING_BRIDGE_READ_TOKEN) {
  const index = Number(gitEnv.GIT_CONFIG_COUNT || 0);
  gitEnv.GIT_CONFIG_COUNT = String(index + 1);
  gitEnv[`GIT_CONFIG_KEY_${index}`] = 'http.https://github.com/Pudding-Player/.extraheader';
  gitEnv[`GIT_CONFIG_VALUE_${index}`] = `AUTHORIZATION: basic ${Buffer.from(`x-access-token:${gitEnv.PUDDING_BRIDGE_READ_TOKEN}`).toString('base64')}`;
}
delete env.PUDDING_BRIDGE_READ_TOKEN;
delete gitEnv.PUDDING_BRIDGE_READ_TOKEN;

function noLinks(path) {
  let current = resolve(path);
  if (current !== root && !current.startsWith(root + sep)) throw Error(`Path outside workspace: ${path}`);
  while (current !== root) {
    if (existsSync(current) && lstatSync(current).isSymbolicLink()) throw Error(`Redirected bridge path: ${current}`);
    current = dirname(current);
  }
}
function command(binary, args, cwd = root, extra = {}) {
  mkdirSync(temporary, { recursive: true });
  const result = spawnSync(binary, args, { cwd, env: binary === 'git' ? gitEnv : env, encoding: 'utf8', windowsHide: true, maxBuffer: 96 * 1024 * 1024, ...extra });
  if (result.error) throw result.error;
  if (result.status !== 0) throw Error(`${binary} failed (${result.status}): ${(result.stderr || result.stdout || '').trim()}`);
  return (result.stdout || '').trim();
}
function git(bridge, ...args) { return command('git', args, resolve(root, bridge.path)); }
function plugin(bridge) { return resolve(root, bridge.path, 'plugins', bridge.id); }
function inspect(bridge) {
  const path = resolve(root, bridge.path);
  noLinks(path);
  if (!existsSync(resolve(path, '.git'))) return null;
  if (resolve(git(bridge, 'rev-parse', '--show-toplevel')) !== path) throw Error(`Not an independent checkout: ${bridge.path}`);
  if (git(bridge, 'remote', 'get-url', 'origin') !== bridge.repository) throw Error(`Unexpected bridge origin: ${bridge.path}`);
  return { revision: git(bridge, 'rev-parse', 'HEAD'), dirty: Boolean(git(bridge, 'status', '--porcelain')) };
}
function ensure(bridge) {
  if (!/^[a-f0-9]{40}$/.test(bridge.revision)) throw Error(`Pin a full commit for ${bridge.id} before fetching`);
  const path = resolve(root, bridge.path);
  let state = inspect(bridge);
  if (!state) {
    if (existsSync(path) && readdirSync(path).length) throw Error(`Refusing to overwrite existing source: ${bridge.path}`);
    noLinks(path);
    mkdirSync(dirname(path), { recursive: true });
    command('git', ['clone', '--no-checkout', '--filter=blob:none', bridge.repository, path]);
    git(bridge, 'fetch', '--no-tags', 'origin', bridge.revision, '--depth=1');
    git(bridge, 'checkout', '--detach', bridge.revision);
    state = inspect(bridge);
  }
  if (locked && state.dirty) throw Error(`Uncommitted bridge changes: ${bridge.path}`);
  if (state.revision !== bridge.revision) {
    if (locked || action === 'fetch') {
      if (state.dirty) throw Error(`Commit local changes before updating ${bridge.id}`);
      git(bridge, 'fetch', '--no-tags', 'origin', bridge.revision, '--depth=1');
      git(bridge, 'checkout', '--detach', bridge.revision);
    } else console.error(`${bridge.id}: using local ${state.revision}; lockfile pins ${bridge.revision}`);
  }
  return inspect(bridge);
}
function sha(path) { return createHash('sha256').update(readFileSync(path)).digest('hex'); }
function fingerprint(bridge) {
  const hash = createHash('sha256');
  const files = command('git', ['ls-files', '--cached', '--others', '--exclude-standard', '-z'], resolve(root, bridge.path)).split('\0').filter(Boolean);
  for (const name of [...new Set(files)].sort()) {
    const path = resolve(root, bridge.path, name);
    noLinks(path);
    if (!existsSync(path)) { hash.update(`${name}\0deleted\0`); continue; }
    hash.update(`${name}\0${sha(path)}\0`);
  }
  return hash.digest('hex');
}
function sourceFiles(directory, prefix) {
  noLinks(directory);
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    const path = resolve(directory, entry.name);
    noLinks(path);
    const name = `${prefix}/${entry.name}`;
    if (entry.isDirectory()) files.push(...sourceFiles(path, name));
    else if (entry.isFile() && /\.(md|json)$/.test(entry.name)) files.push({ Relative: name, Source: path });
    else throw Error(`Unexpected plugin resource: ${path}`);
  }
  return files;
}
function productFiles() {
  const files = [];
  for (const bridge of activeBridges.filter(b => b.bundled)) {
    const source = plugin(bridge);
    const manifest = resolve(source, '.pudding-plugin/plugin.json');
    noLinks(manifest);
    if (JSON.parse(readFileSync(manifest, 'utf8')).id !== bridge.id) throw Error(`Plugin ID mismatch: ${bridge.id}`);
    files.push({ Relative: `plugins/${bridge.id}/.pudding-plugin/plugin.json`, Source: manifest });
    for (const name of ['skills', 'resources']) if (existsSync(resolve(source, name))) files.push(...sourceFiles(resolve(source, name), `plugins/${bridge.id}/${name}`));
    for (const name of bridge.binaries) files.push({ Relative: bridge.id === 'bgi' ? `bridge/${name}` : `plugins/${bridge.id}/bridge/${name}`, Source: resolve(root, 'target/bridge', bridge.id === 'bgi' ? '' : bridge.id, name) });
  }
  files.push({ Relative: 'bridge/bridge.config.example.json', Source: resolve(root, 'bridges/bgi/bgi-bridge/bridge.config.example.json') });
  return files;
}
function verify() {
  const state = JSON.parse(readFileSync(statePath, 'utf8'));
  if (state.schemaVersion !== 1) throw Error('Invalid bridge build manifest');
  for (const bridge of activeBridges.filter(b => b.bundled)) {
    const current = inspect(bridge);
    const built = state.bridges.find(b => b.id === bridge.id);
    if (!built || !current || built.revision !== current.revision || built.sourceHash !== fingerprint(bridge)) throw Error(`Bridge source changed after build: ${bridge.id}`);
    if (locked && (current.dirty || current.revision !== bridge.revision)) throw Error(`Bridge does not match locked source: ${bridge.id}`);
    for (const entry of built.files) {
      const path = resolve(root, entry.path);
      noLinks(path);
      if (!existsSync(path) || sha(path) !== entry.sha256) throw Error(`Bridge binary changed after build: ${entry.path}`);
    }
    if (built.files.length !== bridge.binaries.length) throw Error(`Incomplete bridge build: ${bridge.id}`);
  }
}
try {
  if (options.some(option => option !== '--locked')) throw Error('Only --locked is supported');
  if (action === 'status') {
    for (const bridge of lock.bridges) console.log(JSON.stringify({ id: bridge.id, enabled: bridge.enabled !== false, path: bridge.path, pinned: bridge.revision, ...inspect(bridge) }));
  } else if (action === 'fetch' || action === 'ensure') {
    for (const bridge of activeBridges) { const state = ensure(bridge); console.log(`${bridge.id}: ${state.revision}${state.dirty ? ' (local changes preserved)' : ''}`); }
  } else if (action === 'lock') {
    for (const bridge of activeBridges) {
      const state = inspect(bridge);
      if (!state || state.dirty) throw Error(`Commit ${bridge.id} before locking it`);
      const remote = command('git', ['ls-remote', bridge.repository, 'refs/heads/main']).split(/\s+/)[0];
      if (remote !== state.revision) throw Error(`Push ${bridge.id} main before updating the lockfile`);
      bridge.revision = state.revision;
    }
    writeFileSync(lockPath, JSON.stringify(lock, null, 2) + '\n');
  } else if (action === 'prepare') {
    for (const bridge of activeBridges) ensure(bridge);
    for (const file of productFiles().filter(file => file.Relative.startsWith('plugins/') && !file.Relative.includes('/bridge/'))) {
      const destination = resolve(root, 'target/dev', file.Relative);
      noLinks(destination);
      mkdirSync(dirname(destination), { recursive: true });
      copyFileSync(file.Source, destination);
    }
    for (const id of ['windows-scheduler', 'web-search']) {
      const source = resolve(root, 'plugins', id);
      const files = [{ Relative: `plugins/${id}/.pudding-plugin/plugin.json`, Source: resolve(source, '.pudding-plugin/plugin.json') }, ...sourceFiles(resolve(source, 'skills'), `plugins/${id}/skills`)];
      for (const file of files) {
        const destination = resolve(root, 'target/dev', file.Relative);
        noLinks(destination);
        mkdirSync(dirname(destination), { recursive: true });
        copyFileSync(file.Source, destination);
      }
    }
    console.log('Development plugin resources prepared in target/dev/plugins.');
  } else if (action === 'files') console.log(JSON.stringify(productFiles()));
  else if (action === 'verify') { verify(); console.log('Bridge sources and compiled payloads verified.'); }
  else if (action === 'build') {
    if (process.platform !== 'win32') throw Error('Bridge production builds require Windows');
    for (const bridge of activeBridges) ensure(bridge);
    command(process.execPath, [resolve(root, 'scripts/generate-adapter-disclosure.mjs'), '--check']);
    const built = [];
    for (const bridge of activeBridges) {
      const source = resolve(root, bridge.path);
      const entry = bridge.id === 'bgi' ? 'bgi-bridge\\build.cmd' : `plugins\\${bridge.id}\\bridge\\build.cmd`;
      console.log(`Building ${bridge.id}...`);
      command(process.env.ComSpec || 'cmd.exe', ['/d', '/c', entry, ...(bridge.id === 'bgi' ? [] : ['--production'])], source, { stdio: 'inherit', env: { ...env, PUDDING_BUILD_TMP: temporary, PUDDING_BUILD_OUTPUT: resolve(root, 'target/bridge', bridge.id === 'bgi' ? '' : bridge.id) } });
      const state = inspect(bridge);
      const files = bridge.binaries.map(name => { const path = resolve(root, 'target/bridge', bridge.id === 'bgi' ? '' : bridge.id, name); noLinks(path); return { path: relative(root, path).replaceAll('\\', '/'), sha256: sha(path) }; });
      built.push({ id: bridge.id, ...state, sourceHash: fingerprint(bridge), files });
    }
    writeFileSync(statePath, JSON.stringify({ schemaVersion: 1, bridges: built }, null, 2) + '\n');
    verify();
  } else throw Error(`Unknown bridge action: ${action}`);
} catch (error) { console.error(error.message); process.exitCode = 1; }
