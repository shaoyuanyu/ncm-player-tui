#!/bin/bash
mkdir -p ~/.cache/ncm-tui-player/
RUST_LOG=debug ncm-tui-player.1 2>~/.cache/ncm-tui-player/log
