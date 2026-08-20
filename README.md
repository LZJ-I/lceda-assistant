# 立创封装助手

用 **Rust** 编写的立创 / LCSC 器件工具：按型号或立创编号搜索，下载 STEP / OBJ 三维模型，并把原理图符号、PCB 封装写成 **Altium**（`.SchLib` / `.PcbLib`）和 **KiCad**（`.kicad_sym` / `.pretty`）库。

单个原生二进制，搜索和导出走后台线程，界面不卡。库文件在进程内直接写出，**不需要安装 .NET SDK**。Release 默认 LTO + 单 codegen unit，体积小、启动快，适合把 BOM 里一串 `C` 编号一次性导出来。

界面是浅色苹果风（系统蓝、大圆角、半透明磨砂卡片），中 / 英可切换。

## 能做什么

- 按型号、关键字或立创编号（`Cxxxx`）搜索，精确编号会排到最前
- 下载 STEP、OBJ/MTL，GUI 里有器件图和线框 3D 预览
- 导出 Altium 原理图库 / 封装库
- 导出 KiCad 符号与封装（封装目录可挂 STEP）
- 下载数据手册（接口里有 PDF 链接时）
- 批量：一个文本文件一行一个编号或关键字
- 库属性写入立创编号、厂牌、型号，方便对照下单
- 导出失败也会留下 EasyEDA JSON，方便对照

## 性能

- 原生代码，没有解释器启动、没有运行时再编译 C# 工程
- HTTP 超时与最多 3 次重试，大文件有上限保护
- GUI 的搜索、预览、导出都在独立线程，主界面保持可拖动
- Release 开启 LTO，日常就是一个可拷贝的可执行文件

## 编译

需要 Rust 1.80+。若本机 `rustc` 不在默认 PATH（例如装在 `rust-wsl-dashboard`）：

```bash
export CARGO_HOME="$HOME/rust-wsl-dashboard/cargo"
export RUSTUP_HOME="$HOME/rust-wsl-dashboard/rustup"
export PATH="$CARGO_HOME/bin:$PATH"
```

```bash
cargo build --release -p lceda
./target/release/lceda
```

交叉编译 Windows：

```bash
cargo xwin build --release -p lceda --target x86_64-pc-windows-msvc
```

## 使用

无参数默认打开 GUI。

```bash
# 搜索
lceda search C2040

# 下载 3D，并写出 Altium + KiCad 库（同时保留 JSON）
lceda get C2040 --step --ad --kicad -o ./out

# 只要源文件
lceda get C2040 --source -o ./out

# 数据手册
lceda get C2040 --datasheet -o ./out

# 批量（每行一个 C 编号或关键字，# 开头为注释）
lceda batch ids.txt --ad --kicad --step -o ./out

# 英文界面
lceda --lang en gui
```

GUI：输入关键字搜索 → 点选器件 → 选输出目录 → 导出 AD / KiCad / 3D / 手册。也支持从文本文件批量导出。Linux 窗口管理器不一定有系统磨砂，会用半透明白卡片近似。

## 注意

- 立创接口是非官方的，字段或地址可能会变。
- 生成的符号 / 封装请在 Altium 或 KiCad 里打开核对后再用于生产。
- 多边形焊盘仍按包围盒近似；复杂异形请对照 JSON 检查。
- 本仓库源码按 CC BY-NC-4.0 授权，**禁止商用**。

## 许可

[CC BY-NC-4.0](https://creativecommons.org/licenses/by-nc/4.0/)：个人学习与研究可用，商业使用需另行授权。
