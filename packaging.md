# 打包

使用 [FPM](https://github.com/jordansissel/fpm) 进行打包。

## 准备工作

注意修改版本号。

```shell
cargo build --release
mkdir -p ./target/package/src ./target/package/output
rm ./target/package/src/ncm-tui-player ./target/package/src/ncm-tui-player.sh
ln ./target/release/ncm-tui ./target/package/src/ncm-tui-player
ln ./bin/ncm-tui-player.sh ./target/package/src/ncm-tui-player.sh
export NCM_TUI_PLAYER_VERSION="2.1.0"
```

## 打包成 `rpm`

```shell
fpm -f -s dir -t rpm \
-p "./target/package/output/ncm-tui-player-$NCM_TUI_PLAYER_VERSION-x86_64.rpm" \
--name ncm-tui-player \
--version "$NCM_TUI_PLAYER_VERSION" \
--architecture x86_64 \
--license GPL-3.0 \
--description "A TUI player client for netease-cloud-music written in Rust." \
--url "https://github.com/shaoyuanyu/ncm-tui-player" \
--maintainer "shaoyuanyu<code200.ysy@gmail.com>" \
--depends gstreamer1 \
--depends gstreamer1-plugins-base \
--verbose \
./target/package/src/ncm-tui-player.sh=/usr/bin/ncm-tui-player \
./target/package/src/ncm-tui-player=/usr/bin/ncm-tui-player.1
```

## 打包成 `deb`

```shell
fpm -f -s dir -t deb \
-p "./target/package/output/ncm-tui-player-$NCM_TUI_PLAYER_VERSION-x86_64.deb" \
--name ncm-tui-player \
--version "$NCM_TUI_PLAYER_VERSION" \
--architecture x86_64 \
--license GPL-3.0 \
--description "A TUI player client for netease-cloud-music written in Rust." \
--url "https://github.com/shaoyuanyu/ncm-tui-player" \
--maintainer "shaoyuanyu<code200.ysy@gmail.com>" \
--depends gstreamer1.0-plugins-base \
--depends gstreamer1.0-plugins-good \
--depends gstreamer1.0-plugins-bad \
--verbose \
./target/package/src/ncm-tui-player.sh=/usr/bin/ncm-tui-player \
./target/package/src/ncm-tui-player=/usr/bin/ncm-tui-player.1
```
