<div align="center">

<img src="src-tauri/icons/icon.png" width="128" alt="uuid_remap icon" />

## UUID Remapper

[![Tauri](https://img.shields.io/badge/Tauri-2-FFC131?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app) [![mca](https://img.shields.io/badge/mca-2.1.2-44903F?style=for-the-badge&logo=rust&logoColor=white)](https://crates.io/crates/mca) [![quartz_nbt](https://img.shields.io/badge/quartz__nbt-0.2.9-5C6BC0?style=for-the-badge&logo=rust&logoColor=white)](https://crates.io/crates/quartz_nbt)

[EN](README.md) | [中文](README_ZH.md)

</div>

uuid_remap is a tool for batch-replacing UUIDs in Minecraft world saves. It can be used to fix the problem where inventories, pets, advancements and other data no longer match up after player UUIDs change due to a change of the server's `online_mode` setting. The tool also works on UUIDs that don't belong to players.

### How does it work?

Following a UUID mapping table, uuid_remap walks through every file in the save or server directory and tries to identify each one as an NBT file, an MCA file, or a text file:

- For NBT files, the program traverses the NBT tree and replaces any matching UUIDs found in it.

- For MCA files, the program iterates over all existing chunks and performs the same replacement on the NBT data inside each chunk.

- For text files, the program automatically swaps any valid UUID strings found in them.

Files that cannot be identified are skipped and left unmodified.

For modded saves, this tool handles most common cases as well as it can, and its file traversal is broad enough; however, since there is a huge number of mods out there, a perfect replacement of all mod data cannot be guaranteed.

### Usage

uuid_remap ships with both a GUI and a CLI, which you can find on the [Releases](https://github.com/peter2500zz/uuid_remap/releases) page.

Neither the GUI nor the CLI requires an internet connection, but with internet access the GUI version can automatically compute UUIDs and avatars from player IDs.

#### GUI

1. Select the world directory

Pick the save folder with the "Browse" button, or type the path in manually. If you can access the server, selecting the server folder directly is recommended.

2. Configure the UUID swap rules

uuid_remap tries to auto-detect the player UUIDs in the world. The two UUIDs on each row will be swapped with each other. You can also export a JSON file containing the current swap rules for use with the CLI.

You can add any number of UUID swap rules yourself, but each UUID may only appear once.

The two UUIDs don't need to be related — you can even swap UUIDs between different players, or enter arbitrary UUIDs if you want to.

3. Run the conversion

Make sure every Minecraft client and server using this save is closed. **Back up your world save** — even though the program has been tested, it may still contain unknown bugs that could corrupt your save.

After you click the "Start Conversion" button, uuid_remap will automatically start processing the world folder. Do not close or quit the program before it finishes, or the world save may be corrupted.

#### CLI

1. Prepare a UUID mapping JSON file

Prepare a JSON/JSONC file whose key-value pairs are all strings and valid UUIDs.

Here is an example JSONC file, exported by the GUI version:

```jsonc
{
    // Player[Online] <-> Player[Offline]
    "bd346dd5-ac1c-427d-87e8-73bdd4bf3e13": "a01e3843-e521-3998-958a-f459800e4d11",

    // Peter_2500[Online] <-> Peter_2500[Offline]
    "9db4226c-1015-40da-8fa5-4335aab896b6": "59c66d96-d356-364a-a84e-0511b286a31b"
}
```

The swap rules are the same as in the GUI version. Note that the CLI version does not care about comments or whether duplicate UUIDs exist.

2. Run the conversion

Make sure every Minecraft client and server using this save is closed. **Back up your world save** — even though the program has been tested, it may still contain unknown bugs that could corrupt your save.

```bash
./uuid_remap --world <path/to/save> --map <path/to/mapping.json>
```

Do not close or quit the program before processing finishes, or the world save may be corrupted.

### Building

Clone the project:

```bash
git clone https://github.com/peter2500zz/uuid_remap.git && cd uuid_remap
```

Install the dependencies with bun:

```bash
bun install
```

Start the development version:

```bash
bun run tauri dev
```

Build the GUI:

```bash
bun run tauri build
```

Build the CLI:

```bash
cd src-tauri && cargo build --release -p remapper
```

### Contributing

Any issues and PRs are welcome — I'd really appreciate your help in improving this tool.

---

This project has been tested on Minecraft game versions `1.14` `1.16` `1.17` `26.1`, and works fine on my server save (about 17 GB).
