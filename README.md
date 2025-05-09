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

## 3. Installation
Get the source code then build:
```bash
cargo build --release
```

## 4. Usage
```bash
./dir-vigil -d <the dir path that you wanna vigilance>
```
for help:
```bash
./dir-vigil -h
```