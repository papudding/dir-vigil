# AGENTS.md

本文件为在本仓库中工作的 AI coding agent 提供上下文与操作约定。面向用户/部署的文档见 [README.md](./README.md)（英文）与 [README_ZH-CN.md](./README_ZH-CN.md)（中文）；两者是等价翻译，改动用户可见行为时需同步更新。

## 0. 先说最重要的：这是破坏性工具

dir-vigil 的核心功能是**不可恢复地删除目录**：倒计时到期后会 `remove_dir_all` 监控目录，随后进程 `exit(0)`。

- 任何测试、演示、联调都只能用临时目录（如 `mktemp -d`），**绝不要**指向真实数据、用户主目录或本仓库。
- 不要把 `--timeout-seconds` 调小后对未知目录运行。
- 用户可见行为（超时即删）是本项目语义，不要为了"安全"擅自改变；如需改动，先确认需求并同步 README。

## 1. 项目概览

dir-vigil 是"失能开关"式定时清理工具：

1. **启动**：解析 CLI → 读取 `DIR_VIGIL_SECRET_KEY` → 加载/生成 TOTP secret → 仅在交互终端打印二维码 → 启动 axum HTTP 服务与两个 tokio 后台任务。
2. **续命**：H5 页面先获取验证码，再输入 6 位 TOTP，调用 `POST /keepalive`（JSON body）刷新 `last_active`。
3. **删除**：`monitor_task` 每秒检查，`last_active` 超过 `timeout_seconds` 就递归删除监控目录并退出进程。
4. **告警**：`check_remain_time` 每 `checking_interval` 检查一次，剩余时间进入 `warning_seconds` 窗口后，通过 Bark / Server Chan 发送提醒。

技术栈：Rust（axum 0.7 + tokio + clap + otpauth + aes-gcm + serde）+ Vue 3 / Rsbuild（H5）+ nginx + Docker。当前版本 **0.2.0**。

## 2. 目录结构

```text
.github/                    CI（fmt/clippy/test/frontend/docker）与 release（仅 GitHub release notes）
backend/                   Rust 后端（单 bin，模块在 main.rs 声明）
  src/main.rs              入口：CLI、密钥加载、secret 初始化、路由、CORS、后台任务
  src/config.rs            clap Config、AppState、DIR_VIGIL_SECRET_KEY 解析
  src/handler.rs           HTTP handler：/status、/captcha、POST /keepalive
  src/file_store.rs        AES-256-GCM 加密保存/读取 TOTP secret（600 权限）
  src/totp.rs              TOTP secret / otpauth URI 生成与校验
  src/util.rs              监控删除、告警检查、发送告警、打印二维码、清理目录
  src/logger.rs            env_logger 初始化（固定 Info 级别）
frontend/h5/               Vue 3 + Rsbuild 响应式 H5
  index.html               HTML 模板（lang/title/theme-color/noscript）
  public/favicon.svg       站点图标（Rsbuild 自动注入）
  .env.example             本地开发环境变量模板（.env 已忽略）
  src/App.vue              页面逻辑与样式：倒计时、验证码 + 2FA、离线/到期状态、响应式卡片
  src/api/index.ts         axios 实例（响应拦截器返回 response.data）
  src/api/DirVigil.ts      接口封装 + StatusResponse 类型（keepalive 使用 POST）
  rsbuild.config.ts        自定义 HTML 模板 + 构建期注入 API_BASE_URL（带默认值）
nginx/nginx.conf           生产 nginx：静态托管 + /api/ 反代、限流、安全头、gzip、缓存、安全日志
Dockerfile                 多阶段构建：node 构建前端 + rust 构建后端 + nginx 运行
backend_startup.sh         容器入口：在 /app/logs 归档日志、环境变量翻译、启动后端
docs/screenshots/          README 使用的移动端/桌面端截图
SECURITY.md / CONTRIBUTING.md / CHANGELOG.md
```

## 3. 构建 / 运行 / 测试

### 环境要求

- Rust stable 1.95（edition 2021），`cargo`；若不在 PATH，常见位置是 `~/.cargo/bin`。
- Node.js **18.12+**（推荐 22），包管理器用 **yarn 1.x**（仓库提交了 `yarn.lock`）。
- Docker（构建/发布镜像时需要；Dockerfile 可从源码构建，不再需要预编译产物）。
- 首次构建需要联网下载 crates 与 npm 依赖。

### 后端

```bash
cd backend
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                       # 14 tests, all green
cargo build --release            # 产物 backend/target/release/dir-vigil
```

运行前必须提供 32 字节或 64 hex 的 AES 密钥；同时建议在独立运行目录启动，避免把 `user_secret.enc` / 日志落进仓库：

```bash
export DIR_VIGIL_SECRET_KEY="$(openssl rand -hex 32)"
mkdir -p /tmp/dir-vigil-run && cd /tmp/dir-vigil-run
/absolute/path/to/dir-vigil -d /tmp/vigil-watch
# 首次绑定/重新绑定：打印二维码/URI 后退出（不需要 -d）
/absolute/path/to/dir-vigil --print-secret-uri
```

CLI 参数（`cargo run -- --help` 可查）：

| 参数 | 说明 |
| --- | --- |
| `-d, --directory` | 监控目录；除 `--print-secret-uri` 外必填，启动时必须存在 |
| `-t, --timeout-seconds` | 删除前倒计时秒数，默认 86400（24h） |
| `-w, --warning-seconds` | 进入告警窗口的秒数，默认 21600（6h） |
| `-c, --checking-interval` | 告警检查周期秒数，默认 1200（20min） |
| `--alert-url` | 告警请求地址（含 token，禁止打日志） |
| `--alert-channel` | `bark` 或 `ServerChan3`，大小写敏感 |
| `--host` | HTTP 监听地址，默认 `127.0.0.1`；只有放在可信反代后才用 `0.0.0.0` |
| `-p, --port` | HTTP 端口，默认 8080 |
| `--cors-origin` | 仅本地前端开发时允许一个来源；默认关闭 CORS |
| `--comment` | `/status` 返回的备注，未设置时返回 `^-^` |
| `--print-secret-uri` | 打印 TOTP 绑定二维码/URI 后退出 |

环境变量 `DIR_VIGIL_SECRET_KEY`（32 字节原始字符串或 64 hex）是必需的。密钥变更会导致旧的 `user_secret.enc` 无法解密；从 0.1.0 升级可使用旧硬编码密钥 `0123456789abcdef0123456789abcdef` 保持兼容。

### 前端

```bash
cd frontend/h5
cp .env.example .env      # 可选；不建 .env 时使用默认值
yarn install --frozen-lockfile
yarn dev                  # Rsbuild dev server（默认 3000，自动打开浏览器）
yarn typecheck            # tsc --noEmit（检查 .ts，不检查 .vue 模板）
yarn build                # 产物 frontend/h5/dist
yarn preview
```

- `rsbuild.config.ts` 在未设置 `.env` 时回退：生产 `API_BASE_URL_PROD=/api`，开发 `API_BASE_URL_DEV=http://127.0.0.1:8080`；因此 `yarn build` 不依赖 `.env`。
- 本地前后端联调时后端默认监听 `127.0.0.1:8080`，并需加 `--cors-origin http://localhost:3000`。
- 样式是手写 CSS；移动端优先，`@media (min-width: 960px)` 双栏；页面不应出现滚动条（`index.css` 隐藏滚动条，短屏媒体查询收紧间距），移动端用 `touch-action: manipulation` 禁止 iOS 双击缩放。

### Docker 构建与运行

Dockerfile 已改为**从源码多阶段构建**，不再依赖根目录的 `dist/`、`dir-vigil`、`nginx.conf` 预编译产物：

```bash
docker build -t dir-vigil:0.2.0 .
docker run -d --name dir-vigil \
  --restart unless-stopped \
  -p 127.0.0.1:8082:80 \
  -v /path/to/protected/dir:/vigilDir \
  -v ~/dir-vigil:/app \
  -e DIR_VIGIL_SECRET_KEY="<64-hex-key>" \
  -e TIMEOUT_SECONDS=172800 \
  -e WARNING_SECONDS=43200 \
  -e COMMENT='my data' \
  dir-vigil:0.2.0
```

- `/vigilDir` 是会被删除的数据目录；`/app` 持久化 `user_secret.enc`，日志写入 `/app/logs`（不挂载则容器重建后需重新绑定 2FA）。
- 首次绑定：`docker exec -it dir-vigil dir-vigil --print-secret-uri`（输出即 2FA secret，勿记录/分享）。
- 容器环境变量映射：`DIR_VIGIL_SECRET_KEY`（必需）、`TIMEOUT_SECONDS`、`WARNING_SECONDS`、`CHECKING_INTERVAL`、`ALERT_URL`、`ALERT_CHANNEL`、`COMMENT`、`BIND_HOST`。
- 发布流程：tag `v*` 触发 `.github/workflows/release.yml`，只生成 GitHub release notes；不发布镜像、不上传二进制产物。
- 本地构建新镜像时若 Docker Hub 不可达，可先用镜像站拉取 `rust:1.95-alpine` / `nginx:1.31-alpine` / `node:22-alpine` 并 retag 成 Dockerfile 中的官方 tag。

## 4. HTTP API 契约（前后端共用，改动需同步）

| 接口 | 说明 |
| --- | --- |
| `GET /status` | 返回 JSON（`application/json`）：`{ "remainsSeconds": u64, "comment": "..." }`，无鉴权 |
| `GET /captcha` | 返回 6 位字母数字验证码（纯文本，内存保存） |
| `POST /keepalive` | JSON body：`{ "captcha": "...", "tow_fa_code": "6位TOTP" }`；成功 200 `Status updated` 并重置计时；失败 403 / 429 + 英文错误文本 |

- 参数名是历史拼写 **`tow_fa_code`**（前端 `src/api/DirVigil.ts` 与后端 `handler.rs` 共用），不要单方面"修正"。
- 后端路由没有 `/api` 前缀；生产由 nginx 剥掉 `/api` 前缀转发，前端生产 baseURL 是 `/api`。
- 验证码**一次性**：成功续命或 2FA 校验失败（含格式错误）一次后立即失效，必须重新获取；错误验证码本身最多尝试 5 次，超限返回 429 并清除。真正的鉴权依赖 TOTP。
- 错误文案由前端 `resolveErrorMessage` 映射为中文；新增错误记得同步。
- 默认关闭 CORS；只有显式 `--cors-origin <origin>` 才会返回 CORS 头（GET/POST + Content-Type）。

## 5. 关键机制与不变量

### 配置与密钥

- `AppState`（`Arc<tokio::Mutex<...>>`）持有 `captcha`、`captcha_attempts`、`last_active`、`config`、`file_encypt_key: [u8;32]` 与 secret 路径。
- `DIR_VIGIL_SECRET_KEY` 支持 32 字节原始字符串或 64 hex；解析逻辑与测试在 `config.rs`。缺失/非法直接退出非零。
- `user_secret.enc` 使用 AES-256-GCM，文件权限强制 600；换 key 需恢复旧 key 或删除文件重新绑定。
- **不要把 TOTP secret / otpauth URI / Alert URL 写进日志**。正常启动在非 TTY 下不打印二维码；绑定用 `--print-secret-uri`。告警发送日志只记录 channel。
- `cfg(test)` 中的测试密钥只用于测试，不要复用到生产。

### 计时与删除

- `last_active` 在进程启动时初始化为 `Instant::now()`，只有 `/keepalive` 成功才刷新；**进程重启会重置计时**（这是产品语义）。
- `monitor_task` 每秒持锁读取并打印剩余秒；达到阈值调用 `cleanup_directory` 后 `std::process::exit(0)`。
- `cleanup_directory` 使用 `remove_dir_all`：删除的是目录本身；目录不存在时静默跳过。
- `check_remain_time` 已使用 `saturating_sub`；进入 warning 窗口后每个周期都会发一次告警（无去重/退避）。

### 前端页面约定（`frontend/h5/src/App.vue`）

- 倒计时以 `/status` 为基准记录 deadline，本地每 250ms 按时间戳重算；每 20s 及页面重新可见时校准。
- 状态分级：≤10 分钟 critical、≤1 小时 warning、0 为到期；警告/到期状态会改变配色并隐藏表单。
- 交互链：获取验证码（自动聚焦）→ 验证码正确后自动跳转 2FA → 2FA 满 6 位才可提交 → 成功 toast + 重新拉状态；POST body 携带 `captcha` / `tow_fa_code`。
- 网络错误保留本地倒计时并显示离线 banner；首连失败显示重试页。
- 响应式布局与无滚动条约束见第 3 节。

## 6. 已知问题 / 边界

1. TOTP secret 的加密强度依赖 `DIR_VIGIL_SECRET_KEY` 的保密性；key 与 `user_secret.enc` 同时泄漏即等同于 2FA 失守。旧的 0.1.0 key 已从源码移除，但升级兼容说明里仍会提到 legacy key 字符串。
2. `monitor_task` 每秒 `\r` 打印剩余秒，非 TTY 日志仍会增长；日志在 `/app/logs`，容器脚本已做启动归档，但归档文件会累积，需要定期清理。
3. 验证码是可被自动化获取的"减速带"，不是强认证；真正的安全性来自 TOTP + 单次 2FA 机会 + nginx 限流。
4. 单用户设计：一个实例只有一组 TOTP secret，没有账号/多用户体系。
5. `/status` 无鉴权（只泄漏 remaining/comment）；如需隐藏请放在反代鉴权后。
6. Docker 运行默认以 root 运行 nginx/backend（bind mount 写入需要）；若在 Linux 上改为非 root 需确认 `/vigilDir`、`/app` 写权限。
7. 默认监听 `127.0.0.1`；从局域网直连需要显式 `--host 0.0.0.0`，并自行承担暴露风险，建议 TLS 反代。
8. 测试里的删除行为只能对临时目录执行；不要把正式环境目录当测试数据。
9. 仓库没有 `.env`（只有 `.env.example`）；`dist/`、`node_modules/`、`target/`、`*.enc`、`*.log*` 均被忽略，提交前检查 `git status`。

## 7. 代码风格与提交约定

- **Rust**：`cargo fmt`；模块在 `main.rs` 中声明；日志用 `log` 宏（Info 级）；handler 避免 `unwrap`/`panic`（已有输入解析已安全处理）。`cargo clippy --all-targets -- -D warnings` 必须为 0 警告。
- **前端**：Vue 3 `<script setup lang="ts">` + 手写 CSS + axios 封装；2 空格缩进、单引号、无分号；提交前跑 `yarn typecheck`；未接入 lint/formatter。
- **提交信息**：Conventional Commits（`feat:` / `fix:` / `docs:` / `chore:` / `refactor:` …），标题常用中文，正文列表说明改动文件。
- **版本同步**：`backend/Cargo.toml` 与 `frontend/h5/package.json` 当前均为 0.2.0；README 中镜像 tag 同步；本地 tag 有改写后的 v0.1.0 和新建的 v0.2.0（均尚未 push）。
- **文档同步**：用户可见 CLI/env/API 变化同时更新 `README.md` 与 `README_ZH-CN.md`，并在 CHANGELOG 记录。
- 不要提交构建产物、日志、`*.enc`、node_modules 或 `.env`；`docker build` 由 Dockerfile 自己完成。

## 8. 提交前自查

- [ ] `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 全绿（当前 14 tests）
- [ ] `yarn typecheck`、`yarn build` 通过（可在无 `.env` 的情况下构建）
- [ ] CI 预期覆盖：backend / frontend / docker build（见 `.github/workflows/ci.yml`）
- [ ] 未改变"超时即删除"语义和 `/keepalive` 鉴权链路；测试只用了临时目录
- [ ] 版本号、README（中英文）、`.env.example`、Docker/nginx 配置按需同步
- [ ] `git status` 没有意外产物、密钥文件、日志；`git diff --check` 通过
