# alfred-num-format-converter

Alfred Workflow：输入一个数字，实时给出 Hex / Dec / Oct / Bin 四种进制候选，回车复制。

在 Alfred 输入框输入 `x <number>`：

```
x 100
```

| 候选 | 副标题 |
| --- | --- |
| `0x64` | Hexadecimal |
| `100` | Decimal |
| `0144` | Octal |
| `0b0110 0100` | Binary |

## 输入格式

| 写法 | 进制 | 例子 |
| --- | --- | --- |
| 纯数字 | 十进制 | `x 100` |
| `0x` 前缀 | 十六进制 | `x 0x64` |
| `0` 前缀 | 八进制 | `x 0144` |
| `0b` / `b` 前缀 | 二进制 | `x 0b1100100`、`x b1100100` |
| `-` 前缀 | 负数（以上均可） | `x -100`、`x -0x64` |

正数按 u64 解析（0 ~ 18446744073709551615），负数按 i64（最低 -9223372036854775808），数字里可带 `_` 分隔（如 `x 1_000_000`）。非法输入（如 `x 089`）会显示错误提示。

## 负数

负数时 Hex / Dec / Oct / Bin 显示带符号的绝对值形式，并额外追加一个补码候选：

```
x -100
```

| 候选 | 副标题 |
| --- | --- |
| `-0x64` | Hexadecimal |
| `-100` | Decimal |
| `-0144` | Octal |
| `-0b0110 0100` | Binary |
| `0b1001 1100` | Binary (2's complement, 8-bit) |

补码自动选最小位宽（8/16/32/64-bit），副标题标明位宽。

## 配置项

在 Alfred 设置中右键 workflow → `Configure...` 调整：

| 选项 | 默认 | 说明 |
| --- | --- | --- |
| Show base prefix | 开 | 结果中显示进制前缀（`0x` / `0` / `0b`） |
| Pad hex digits | 关 | hex 补零：`0x4` → `0x04` |
| Hex splitting | none | hex 分组：`none` → `0x12345`，`pairs` → `0x1 23 45`，`quads` → `0x1 2345` |
| Decimal splitting | none | 千位分组：`none` / `comma`（1,234,567）/ `period` / `underscore` |

补位与分组联动：分组开启时勾选 Pad，会补到组大小的整数倍，例如 quads + pad：`0x12345` → `0x0001 2345`。

二进制输出固定补到整字节后按 4 位分组：`0b0110 0100`。

## 安装

双击 `numfmt.alfredworkflow`，或拖入 Alfred 设置的 Workflows 面板。

## 从源码构建

需要 Rust 工具链（无第三方依赖，纯 std）：

```sh
./package.sh        # cargo build --release + 打包 numfmt.alfredworkflow
cargo test          # 运行单元测试
```

产物：`workflow/info.plist` + 编译出的 `numfmt` 二进制，打包为 `numfmt.alfredworkflow`（本质是 zip）。
