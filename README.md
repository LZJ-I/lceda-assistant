# 立创封装助手

按型号或立创编号搜索器件，下载 STEP / OBJ，导出 Altium（`.SchLib` / `.PcbLib`）和 KiCad（`.kicad_sym` / `.pretty`）库。支持中文 / 英文界面。

许可为 [CC BY-NC-4.0](https://creativecommons.org/licenses/by-nc/4.0/)，禁止商用。

## 安装

从 [Releases](https://github.com/LZJ-I/lceda-assistant/releases) 下载对应系统的压缩包并解压。

- Windows：运行 `lceda.exe`
- Linux：运行 `./lceda`

或从源码编译（需 [Rust](https://rustup.rs/)）：

```bash
cargo build --release -p lceda
./target/release/lceda
```

## 使用

无参数打开图形界面。

```bash
lceda search C2040
lceda get C2040 --step --ad --kicad -o ./out
lceda get C2040 --source -o ./out
lceda get C2040 --datasheet -o ./out
lceda batch ids.txt --ad --kicad --step -o ./out
lceda --lang en gui
```

`batch` 文本文件每行一个编号或关键字，`#` 开头为注释。

导出结果请在 Altium / KiCad 中核对后再使用。立创接口为非官方接口，可能随时变化。
