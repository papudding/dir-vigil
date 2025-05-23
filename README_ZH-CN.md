# dir-vigil
[English](README.md) | 中文说明

是一款基于时间触发的文件夹自动删除工具，其核心机制类似于“[失能开关](https://baike.baidu.com/item/%E5%A4%B1%E8%83%BD%E5%BC%80%E5%85%B3/22694753)”。
> 直白一点，就是你💀了之后帮你销毁数据的。

<img width="250" alt="phone1" src="https://github.com/user-attachments/assets/3fdd8fc7-5336-4730-ab5c-22fddab106e6" />
<img width="255" alt="phone3" src="https://github.com/user-attachments/assets/51ad0e68-2d54-4bbd-91d0-26981cd86ae1" />

## 1. 核心流程
该工具通过以下流程工作：

### 1.1 初始化配置：
- 用户指定监控目录路径
- 设置文件保留周期（过期时限）

### 1.2 运行机制：
- 系统会持续监测预设的过期时限
- 若到达时限未收到重置信号，将自动清空目标文件夹
- 用户需在时限到期前主动触发重置操作来延续保护周期

### 1.3 使用要求：
为确保受保护文件夹的持续存在，用户必须定期（建议间隔小于设置的过期时限）执行重置操作。这种设计既保证了临时文件的自动清理，又为需要长期保留的文件提供了可控的保护机制。

## 2. 免责声明⚠️
### 2.1 数据安全提示：
本软件（dir-vigil）提供的功能涉及**文件自动删除**，请谨慎操作。软件开发者**不对因使用本工具导致的任何数据丢失负责**，包括但不限于：

- 因误操作、配置错误或未及时重置过期时间导致的文件删除
- 系统异常、权限问题或其他不可控因素造成的数据损失
### 2.2 使用建议：

- 在关键目录上使用前，请确保**重要数据已备份**
- 合理设置过期时间，避免因疏忽导致文件被自动清理
- 定期检查软件运行状态，确保重置机制正常工作

**继续使用即表示您已充分了解并接受相关风险。**

## 3.docker安装
从relaese获取镜像打包文件（注意宿主机环境），导入docker
```bash
docker load -i dir-vigil-linuxamd64.tar
```
运行镜像
```bash
docker run -d -p <your_pot>:80 --name dir-vigil -v <your_dir>:/vigilDir dir-vigil:0.1.0
```
## 4. 裸机安装
获取源码:
```bash
git clone https://github.com/papudding/dir-vigil.git
```
### 前端
```bash
cd frontend/h5
yarn install
yarn build
```
> 省略nginx安装

将前端编译后生产的`frontend/h5/dist`目录下的文件复制到nginx的`html`目录下
参考`nginx/nginx.conf`配置nginx的config

### 后端
> 省略rust环境安装

```bash
cd backend
cargo build --release
```
产生的可执行文件在`backend/target/release`目录下

启动：
```bash
./dir-vigil -d <the dir path that you wanna vigilance>
```
## 5. 使用方法
1. 启动docker容器或裸机程序后端
2. 使用2FA验证器扫描终端打印的二维码获取2FA代码（推荐Microsoft Authenticator）
<img width="450" alt="cmd1" src="https://github.com/user-attachments/assets/e07f7b0d-f54d-4030-b0b8-b8f22430da33" />

3. 定期访问前端页面进行重置操作

## 6. 提醒配置
<img width="220" alt="phone2" src="https://github.com/user-attachments/assets/56d63664-5f83-4a83-a0be-5e14eb6a4bfe" />

### docker
支持bark和server酱两种提醒方式
```bash
docker run -d -p <your_pot>:80 --name dir-vigil \
-v <your_dir>:/vigilDir \
-e ALERT_URL=<your_request_url> \
-e ALERT_CHANNEL=bark \
dir-vigil:0.1.0
```
server酱：`-e ALERT_CHANNEL=ServerChan3`

### 裸机
```bash
./dir-vigil --directory <the dir path that you wanna vigilance> \
--alert-url <your_request_url> \
--alert-channel <bark or ServerChan3>
```

## 7. 后端其他可配置项
| 配置项 | 类型 | 默认值 | 描述 |
| --- | --- | --- | --- |
| --directory | string | - | 要跟踪以供删除的目录路径 |
| --timeout-seconds | int | 86400 | 删除前超时 (秒)(默认：24小时) |
| --warning-seconds | int | 21600 | 删除前开始提醒的时间 (秒)(默认：12小时) |
| --checking-interval | int | 1200 | warning_seconds 的检查和警报间隔 (秒)(默认：20分钟) |
| --alert-url | string | - | 提醒请求地址 |
| --alert-channel | string | - | 提醒渠道（bark 或 ServerChan3） |
| --port | int | 8080 | 服务端口 |
| --help | bool | false | 显示帮助信息 |