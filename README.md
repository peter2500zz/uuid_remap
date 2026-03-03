### Minecraft服务器存档UUID转换器
当服务器online-mode参数变化时，玩家的UUID会发生改变，这大概会让你没法用原本的背包，末影箱，还有宠物(尤其是某天转正但放不下存档的时候)</br>
使用此插件，可以从NBT层面改变UUID，以及处理就的玩家存档。</br>

首先需要编写一个json，用于指示UUID的变更</br>
可以在 [Calculators/Player UUID](https://minecraft.wiki/w/Calculators/Player_UUID) 中用玩家ID快速获取离线与在线的UUID</br>

example.json
```json
{
    "59c66d96-d356-364a-a84e-0511b286a31b": "9db4226c-1015-40da-8fa5-4335aab896b6"
}
```

之后通过参数传入存档目录以及此UUID映射表(在这个示例中是`example.json`)

```shell
./uuid_remap --world=/path/to/world --map=example.json --dry
```

`--dry` 参数用于仅仅扫描可供替换的UUID，不携带此参数便会真正的替换</br>

**真正运行前请备份存档**，虽然代码已经尽可能处理了错误，但无法保证绝对可靠，也请务必在上线后测试结果(比如猫猫狗狗的主人是否正确)</br>

更多可用参数请传入`--help`获取
