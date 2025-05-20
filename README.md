# dir-vigil
English | [中文说明](./README_ZH-CN.md) 

dir-vigil is a time-triggered automatic folder cleanup tool, with its core mechanism similar to a "dead man's switch". 

> To be blunt, it's about helping you destroy data after you 💀.

## 1. core process
The tool works through the following process:

### 1.1 Initial configuration:
- User Specified Monitoring Directory Path
- Set File Retention Period (Expiration Time Limit)

### 1.2 Operating mechanism:
- The system continuously monitors the preset expiration time limit
- If the time limit is reached and no reset signal is received, the destination folder will be emptied automatically
- Users need to actively trigger a reset operation before the time limit expires to continue the protection period

### 1.3 Requirements for use:
To ensure the continued existence of the protected folder, the user must perform the reset operation periodically (the recommended interval is less than the set expiration time limit). This design not only ensures the automatic cleaning of temporary files, but also provides a controllable protection mechanism for files that need to be retained for a long time.

## 2. Disclaimer⚠️
### 2.1 Data security tips:
The functions provided by this software (dir-vigil) involve **automatic deletion of files**, please operate with caution. The software developer is **NOT responsible for any data loss resulting from the use of this tool**, including but not limited:

- File deletion due to misoperation, misconfiguration, or failure to reset the expiration time in time
- Data loss caused by system abnormality, permission problem or other uncontrollable factors
### 2.2 Recommendations for use:

- **Make sure important data is backed up** before using on critical folder
- Set the expiration time reasonably to avoid automatic cleaning of files due to negligence
- Regularly check the operating status of the software to ensure that the reset mechanism is working properly

**By continuing to use, you are fully aware of and accept the risks involved.**

## 3.docker installation
Obtain the image packaging file from relaese (note the host environment) and import it to Docker
```bash
docker load -i dir-vigil-linuxamd64.tar
```
Run the image
```bash
docker run -d -p <your_pot>:80 --name dir-vigil -v <your_dir>:/vigilDir dir-vigil:0.1.0
```

## 4. Bare metal installation
Get the source code:
```bash
git clone https://github.com/papudding/dir-vigil.git
```
### Front-end
```bash
cd frontend/h5
yarn install
yarn build
```
> Omit the nginx installation

Copy the files in the `frontend/h5/dist` directory produced by the frontend compilation to the `html` directory of nginx
Configure the config of nginx by referring to `nginx/nginx.conf`

### Backend
> Omit rust installation

```bash
cd backend
cargo build --release
```
The resulting executable file is located in the 'backend/target/release' directory

Startup:
```bash
./dir-vigil -d <the dir path that you wanna vigilance>
```

## 5. Usage:
1. Start the docker container or bare metal program backend
2. QR code printed using a 2FA authenticator terminal (Microsoft Authenticator recommended)
// pic
3. Visit the front-end page regularly to reset the operation

## 6. Reminder configuration
### docker
Supports two reminder methods: bark and server sauce
```bash
docker run -d -p <your_pot>:80 --name dir-vigil \
-v <your_dir>:/vigilDir \
-e ALERT_URL=<your_request_url> \
-e ALERT_CHANNEL=bark \
dir-vigil:0.1.0
```
server-chan: '-e ALERT_CHANNEL=ServerChan3'

### Bare metal
```bash
./dir-vigil --directory <the dir path that you wanna vigilance> \
--alert-url <your_request_url> \
--alert-channel <bark or ServerChan3>
```

## 7. Other configurable items on the backend
| params | Type | Default value | Description |
| --- | --- | --- | --- |
| --directory | string | - | The directory path to be tracked for deletion |
| --timeout-seconds | int | 86400 | Timeout (seconds) before deletion (default: 24 hours).
| --warning-seconds | int | 21600 | The time (seconds) when the reminder starts before deletion (default: 12 hours) |
| --checking-interval | int | 1200 | Check and alert interval (seconds) for warning_seconds (default: 20 minutes).
| --alert-url | string | - | Reminder request address |
| --alert-channel | string | - | Reminder channel (bark or ServerChan3) |
| --port | int | 8080 | Service port |
| --help | bool | false | Show help |