# rtoolkit

rtoolkit 是一个用 Rust 编写的本地工具箱，提供命令行和 Web 工作台两种使用方式。

当前已支持：

- 中国大陆身份证测试数据生成
- TCP 端口扫描
- JSON 格式化、压缩和排序
- 本地 Web 页面调用工具

> 生成的身份证信息仅用于测试、开发和演示，不代表真实身份信息，请勿用于非法用途。

## 安装与构建

从源码构建：

```bash
git clone https://github.com/zhengpanone/rtoolkit.git
cd rtoolkit
cargo build --release
```

本地开发运行：

```bash
cargo run -- --help
```

安装到本机 Cargo bin 路径：

```bash
cargo install --path .
```

## 命令总览

```bash
rtoolkit --help
```

可用命令：

```text
idgen      生成中国身份证号
port-scan  端口扫描
jsonfmt    JSON 格式化
web        启动本地 Web 工作台
```

使用 `cargo run` 时，需要把参数放在 `--` 后面：

```bash
cargo run -- idgen -n 3
```

## 身份证生成

生成 1 条随机数据：

```bash
rtoolkit idgen
```

生成多条数据：

```bash
rtoolkit idgen -n 5
```

指定地区代码：

```bash
rtoolkit idgen --region 110101
```

指定出生日期：

```bash
rtoolkit idgen --birth 1990-05-20
```

指定性别：

```bash
rtoolkit idgen --gender male
rtoolkit idgen --gender female
```

指定随机生日范围：

```bash
rtoolkit idgen --min-birth 1980-01-01 --max-birth 2000-12-31
```

完整示例：

```bash
rtoolkit idgen -n 3 --region 310101 --birth 1995-08-15 --gender female
```

日期格式支持：

- `19900520`
- `1990-05-20`

## 端口扫描

扫描本机 80 端口：

```bash
rtoolkit port-scan
```

扫描指定目标：

```bash
rtoolkit port-scan --target 127.0.0.1 --port 80
```

扫描端口范围：

```bash
rtoolkit port-scan --target 127.0.0.1 --port 80-100
```

调整并发和超时时间：

```bash
rtoolkit port-scan --target 127.0.0.1 --port 1-1024 --concurrency 200 --timeout 500
```

JSON 输出：

```bash
rtoolkit port-scan --target 127.0.0.1 --port 80-100 --output json
```

为避免误操作，单次端口扫描最多允许 4096 个端口。

## JSON 格式化

默认会尽量保留对象 key 的输入顺序；需要稳定排序时使用 `--sort`。

从 stdin 读取并格式化输出：

```bash
echo '{"b":1,"a":[2,3]}' | rtoolkit jsonfmt
```

直接传入 JSON 文本：

```bash
rtoolkit jsonfmt -i '{"b":2,"a":1}'
rtoolkit jsonfmt '{"b":2,"a":1}' -o output.json
```

读取 JSON 文件并输出到终端：

```bash
rtoolkit jsonfmt -i input.json
```

读取 JSON 文件并写入另一个文件：

```bash
rtoolkit jsonfmt -i input.json -o output.json
```

指定缩进宽度：

```bash
rtoolkit jsonfmt -i input.json --indent 4
```

按对象 key 排序：

```bash
rtoolkit jsonfmt -i input.json --sort -o output.json
```

压缩为单行 JSON：

```bash
rtoolkit jsonfmt -i input.json --compact -o output.json
```

命令别名 `json-fmt` 也可使用。

## Web 工作台

启动本地 Web 页面：

```bash
rtoolkit web
```

默认监听：

```text
http://127.0.0.1:8080
```

指定端口：

```bash
rtoolkit web --port 18080
```

指定监听地址：

```bash
rtoolkit web --host 0.0.0.0 --port 8080
```

Web 前端文件位于：

```text
static/index.html
static/idgen.html
static/port-scan.html
static/styles.css
static/idgen.js
static/port-scan.js
```

后端服务入口位于：

```text
src/web.rs
```

`src/web.rs` 只负责 HTTP 服务、API 路由和静态资源响应。前端页面拆分为 HTML、CSS 和 JS，访问 `/idgen` 进入身份证生成页面，访问 `/port-scan` 进入端口扫描页面。

## Web API

### 健康检查

```http
GET /api/health
```

响应示例：

```json
{
  "status": "ok"
}
```

### 生成身份证数据

```http
POST /api/idgen
Content-Type: application/json
```

请求示例：

```json
{
  "count": 2,
  "region": "110101",
  "birth": "1990-05-20",
  "gender": "female"
}
```

字段说明：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `count` | number | 生成数量，范围 1-200 |
| `region` | string | 可选，6 位地区代码 |
| `birth` | string | 可选，固定出生日期 |
| `min_birth` | string | 可选，随机生日最小值 |
| `max_birth` | string | 可选，随机生日最大值 |
| `gender` | string | `any`、`male`、`female` |

响应示例：

```json
{
  "records": [
    {
      "name": "张三",
      "id_number": "110101199005200024",
      "region": "110101",
      "birthday": "1990-05-20",
      "gender": "female",
      "address": "北京市市辖区东城区"
    }
  ]
}
```

### 端口扫描

```http
POST /api/portscan
Content-Type: application/json
```

请求示例：

```json
{
  "target": "127.0.0.1",
  "port": "80-100",
  "concurrency": 100,
  "timeout_ms": 1000
}
```

字段说明：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `target` | string | 目标主机，默认 `127.0.0.1` |
| `port` | string | 端口或端口范围，例如 `80`、`80-100` |
| `concurrency` | number | 并发数，范围 1-1000 |
| `timeout_ms` | number | 连接超时时间，范围 50-10000 |

响应示例：

```json
{
  "target": "127.0.0.1",
  "port_range": "80-100",
  "concurrency": 100,
  "timeout_ms": 1000,
  "total": 21,
  "open_count": 1,
  "closed_count": 20,
  "open_ports": [80],
  "ports": [
    {
      "port": 80,
      "open": true
    }
  ]
}
```

## 项目结构

```text
rtoolkit/
├── Cargo.toml
├── README.md
├── data/
│   ├── provinces.csv
│   ├── cities.csv
│   ├── areas.csv
│   └── streets.csv
├── static/
│   ├── index.html
│   ├── idgen.html
│   ├── port-scan.html
│   ├── styles.css
│   ├── idgen.js
│   └── port-scan.js
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── web.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── idgen.rs
│   │   ├── jsonfmt.rs
│   │   └── portscan.rs
│   └── utils/
│       ├── mod.rs
│       └── areas.rs
└── tests/
    └── fake.rs
```

## 开发

运行测试：

```bash
cargo test
```

调试构建：

```bash
cargo build
```

运行 Web 工作台：

```bash
cargo run -- web --port 8080
```

运行命令示例：

```bash
cargo run -- idgen -n 3
cargo run -- port-scan --target 127.0.0.1 --port 80-100
cargo run -- jsonfmt input.json -o output.json
```

## 技术栈

- Rust 2021
- clap：命令行参数解析
- tokio / futures：异步端口扫描
- chrono：日期处理
- rand / fake：测试数据生成
- serde / serde_json：JSON 序列化与格式化
- Vue.js：Web 工作台前端页面

## 许可证

本项目采用双许可证：

- MIT
- Apache-2.0
