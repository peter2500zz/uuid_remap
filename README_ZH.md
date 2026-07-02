<div align="center">

<img src="src-tauri/icons/icon.png" width="128" alt="uuid_remap icon" />

## UUID 交换器

[![Tauri](https://img.shields.io/badge/Tauri-2-FFC131?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app) [![mca](https://img.shields.io/badge/mca-2.1.2-44903F?style=for-the-badge&logo=rust&logoColor=white)](https://crates.io/crates/mca) [![quartz_nbt](https://img.shields.io/badge/quartz__nbt-0.2.9-5C6BC0?style=for-the-badge&logo=rust&logoColor=white)](https://crates.io/crates/quartz_nbt)


[EN](README.md) | [中文](README_ZH.md)

</div>

uuid_remap 是一个用于批量替换 Minecraft 世界存档中 UUID 的工具，可用于解决服务器 `online_mode` 设置变化后玩家 UUID 改变导致的背包、宠物、进度等数据无法对应的问题。此外，本工具对玩家以外的 UUID 也起效。

### 它如何工作？

uuid_remap 会根据 UUID 映射表遍历存档或服务器目录中的所有文件，并尝试将每个文件识别为 NBT 文件、MCA 文件或文本文件进行处理。

- 对于 NBT 文件，程序会遍历 NBT 树，将其中匹配到的 UUID 进行替换。

- 对于 MCA 文件，程序会遍历其中所有已存在的区块，并对每个区块内的 NBT 数据执行相同的替换操作。

- 对于文本文件，程序会自动交换其中存在的合法 UUID 字符串。

无法识别的文件将被直接跳过，不会进行修改。

对于使用 MOD 的存档，本工具会尽可能处理大部分常见的情况，且文件遍历足够宽泛，但由于 MOD 数量庞大，因此不能保证可以完美替换所有 MOD 数据。

### 使用方法

uuid_remap 同时支持 GUI 与 CLI，你可以在 [Releases](https://github.com/peter2500zz/uuid_remap/releases) 页面找到它们。

GUI 与 CLI 版本均无需连接互联网，但如果能够访问互联网，GUI 版本将能够根据玩家 ID 自动计算 UUID 与头像。

#### GUI

如果你使用 macOS，由于 GUI 版本没有经过 Apple 签名与公证，下载的 `.app` 打开时会提示“已损坏，无法打开”。在终端执行以下命令解除隔离后就可以打开：

```bash
xattr -cr <uuid_remap.app 的路径>
```

1. 选择世界目录

通过“浏览”按钮选择存档文件夹，或者手动输入路径。如果能够访问到服务器，推荐直接选择服务器文件夹。

2. 设定 UUID 交换规则

uuid_remap 会尝试自动检测世界中的玩家 UUID。每一行中的两个 UUID 会互相交换。你也可以导出一份 json 文件，包含当前的交换规则以供 CLI 使用。

你也可以自己添加任意数量的 UUID 交换规则，但一个 UUID 只能出现一次。

两个 UUID 之间不必有关联，如果你想的话甚至可以交换不同玩家的 UUID 或者写入任意 UUID。

3. 执行转换

确保关闭所有正在使用这个存档的 Minecraft 客户端与服务器。**请备份你的世界存档**，尽管通过了测试，程序仍然可能存在未知的 BUG，并导致存档损坏。

点击“转换”按钮后，uuid_remap 会自动开始处理世界文件夹。在处理完毕前，请不要关闭或退出程序，否则世界存档将可能损坏。

#### CLI

1. 准备 UUID 映射 JSON 文件

准备一份 JSON/JSONC 文件，其中的键值对应当全是字符串并是有效 UUID。

以下是一个 JSONC 文件的例子，由 GUI 版本导出：

```jsonc
{
    // Player[Online] <-> Player[Offline]
    "bd346dd5-ac1c-427d-87e8-73bdd4bf3e13": "a01e3843-e521-3998-958a-f459800e4d11",

    // Peter_2500[Online] <-> Peter_2500[Offline]
    "9db4226c-1015-40da-8fa5-4335aab896b6": "59c66d96-d356-364a-a84e-0511b286a31b"
}
```

交换规则与 GUI 版本一致。请注意 CLI 版本不会关心注释以及是否存在重复 UUID。

2. 执行转换

确保关闭所有正在使用这个存档的 Minecraft 客户端与服务器。**请备份你的世界存档**，尽管通过了测试，程序仍然可能存在未知的 BUG，并导致存档损坏。

```bash
./uuid_remap --world <存档路径> --map <JSON 映射文件路径>
```

在处理完毕前，请不要关闭或退出程序，否则世界存档将可能损坏。

### 构建

将项目 clone 到本地

```bash
git clone https://github.com/peter2500zz/uuid_remap.git && cd uuid_remap
```

使用 bun 安装依赖

```bash
bun install
```

启动测试版本

```bash
bun run tauri dev
```

构建 GUI

```bash
bun run tauri build
```

构建 CLI

```bash
cd src-tauri && cargo build --release -p remapper
```

### 贡献

欢迎任何 issue 与 PR，如果你愿意协助完善这个工具我会相当感谢的。

---

本项目在 Minecraft 游戏版本 `1.14` `1.16` `1.17` `26.1` 进行过测试，并且在我的服务器存档（约 17GB）上工作正常。
