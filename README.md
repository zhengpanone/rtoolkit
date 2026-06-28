# rtoolkit

rtoolkit 是一个用 Rust 编写的本地工具箱，提供命令行和 Web 工作台两种使用方式。

当前已支持：

- 中国大陆身份证测试数据生成
- TCP 端口扫描
- JSON 格式化、压缩和排序
- PDF 信息查看、拆分和合并
- 图片格式转换、颜色调整、滤镜和水印
- 本地 Web 页面调用工具

## Feature

核心 feature：

- `idgen`：生成中国大陆身份证测试数据
- `port-scan`：扫描 TCP 端口
- `jsonfmt`：格式化、压缩、排序 JSON
- `pdf`：查看 PDF 信息、拆分和合并
- `imgtool`：图片格式转换、颜色调整、滤镜和水印
- `web`：本地 Web 工作台统一入口

可继续扩展的实用 CLI：

- `hash`：计算文件哈希
- `base64`：编码和解码文本或文件
- `file-info`：查看文件大小、类型、时间信息
- `du`：统计目录占用
- `rename`：批量重命名
- `url`：URL 编码、解码和 query 解析
- `uuid`：生成 UUID 或 ULID
- `csvfmt`：CSV 和 JSON 互转
- `yamlfmt`：YAML 格式化和互转
- `tomlfmt`：TOML 格式化
- `regex`：正则测试和替换
- `diff`：文本或 JSON 差异对比
- `http`：轻量 HTTP 请求
- `dns`：DNS 查询
- `ping`：TCP ping 或 HTTP ping
- `whois`：域名 whois 查询
- `ip`：本机 IP、CIDR 解析
- `jwt`：解析 JWT
- `password`：生成随机密码
- `otp`：TOTP/HOTP 生成或校验
- `encrypt`：本地文件加解密
- `cert`：查看证书信息和过期时间
- `image`：图片压缩、缩放和格式转换 ✅ 已实现
- `qrcode`：生成和解析二维码
- `pdf text`：提取 PDF 文本
- `pdf images`：提取 PDF 图片
- `pdf rotate`：旋转 PDF 页面
- `pdf encrypt/decrypt`：PDF 加密和解密

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
pdf        PDF 处理工具
imgtool    图片处理工具
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

## PDF 处理

查看 PDF 基本信息：

```bash
rtoolkit pdf info input.pdf
```

按页拆分 PDF：

```bash
rtoolkit pdf split input.pdf -o pages
```

指定拆分后的文件名前缀：

```bash
rtoolkit pdf split input.pdf -o pages --prefix report
```

合并多个 PDF：

```bash
rtoolkit pdf merge a.pdf b.pdf c.pdf -o merged.pdf
```

当前 PDF 命令使用纯 Rust 依赖处理文件，不需要额外安装系统命令。基础功能适合未加密或普通结构的 PDF；复杂表单、签名、加密 PDF 建议先另存为普通 PDF 后再处理。

## 图片处理

图片处理工具支持格式转换、颜色调整、滤镜和水印。

### 格式转换

```bash
rtoolkit imgtool basic convert input.jpg output.png
```

支持的输入/输出格式：JPEG、PNG、GIF、BMP、WebP、TIFF。

### 颜色调整

调整亮度：

```bash
rtoolkit imgtool color brightness input.jpg output.jpg -v 1.5
```

调整对比度：

```bash
rtoolkit imgtool color contrast input.jpg output.jpg -v 1.2
```

### 滤镜

```bash
rtoolkit imgtool filter grayscale input.jpg output.jpg
```

### 水印

添加文字水印：

```bash
rtoolkit imgtool watermark text input.jpg "Hello" output.jpg
```

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
static/jsonfmt.html
static/styles.css
static/idgen.js
static/port-scan.js
static/jsonfmt.js
```

后端服务入口位于：

```text
src/web.rs
```

`src/web.rs` 只负责 HTTP 服务、API 路由和静态资源响应。前端页面拆分为 HTML、CSS 和 JS，访问 `/idgen` 进入身份证生成页面，访问 `/port-scan` 进入端口扫描页面，访问 `/jsonfmt` 进入 JSON 格式化页面。

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
| `count` | number | 生成数量，范围 1-10000000 |
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
│   ├── jsonfmt.html
│   ├── styles.css
│   ├── idgen.js
│   ├── port-scan.js
│   └── jsonfmt.js
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── web.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── idgen.rs
│   │   ├── jsonfmt.rs
│   │   ├── pdf.rs
│   │   ├── portscan.rs
│   │   └── imagetool/
│   │       ├── mod.rs
│   │       ├── basic/
│   │       │   ├── mod.rs
│   │       │   └── convert.rs
│   │       ├── color/
│   │       │   └── mod.rs
│   │       ├── filter/
│   │       │   └── mod.rs
│   │       └── watermark/
│   │           └── mod.rs
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
cargo run -- pdf info input.pdf
```

## 技术栈

- Rust 2021
- clap：命令行参数解析
- tokio / futures：异步端口扫描
- chrono：日期处理
- rand / fake：测试数据生成
- serde / serde_json：JSON 序列化与格式化
- lopdf：PDF 读取、拆分和合并
- image：图片格式转换与处理
- Vue.js：Web 工作台前端页面

## 许可证

本项目采用双许可证：

- MIT
- Apache-2.0
