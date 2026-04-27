# rain-rs

`rain.c` 的 Rust 移植版 — 经典终端数字雨动画。

```
                                  000      00
                            0000000   0000
               0      00  00000000000000000
             0000 0  000000000000000000000000       0
          000000000000000000000000000000000000000 000
         0000000000000000000000000000000000000000000000
     000000000000000000000000000000000000000000000000
 00000000000000000000000000000000000000000000000000000000
             C
                 O M        |
                     F          |
                  Y                         |         |
             |                R  A
                                   I N
                        I N   |
               |                                    |
         |                            Y O
                 |                   U        R
                    |            T E

                                      R   |   |
                          |            M
                                I N
                                         AL
```

原版 [rain.c](https://github.com/nkleemann/ascii-rain) 由 @nkleemann 用 C + ncurses 写成。本项目用 crossterm 跨平台库移植为 Rust，支持风效、密度调节和多种配色。

## 快速开始

```bash
cargo run
```

按 `q` 退出。

## 安装

```bash
cargo install --path .
```

## 选项

```
-s, --speed <SPEED>        下落速度倍率 [0.5..3.0]  [default: 1.0]
-d, --density <DENSITY>    雨滴密度倍率 [0.1..5.0]  [default: 1.0]
-w, --wind <WIND>          风力偏移   [-5.0..5.0]  [default: 0]
-c, --color-mode <MODE>    配色模式                  [default: gradient]
-h, --help                 查看帮助
-V, --version              显示版本
```

## 示例

```bash
cargo run                                           # 默认渐变色
cargo run -- -c matrix                              # 矩阵风
cargo run -- -c dracula -s 1.5                      # Dracula 加速
cargo run -- -c catppuccin -d 0.3 -s 0.5            # 稀疏慢速柔风雪
cargo run -- -c solarized -w -2.0                   # Solarized 左斜风
cargo run -- -c monokai -d 2.0 -s 2.0 -w 3.0        # 密集快速大风 Monokai
```

## 许可

MIT
