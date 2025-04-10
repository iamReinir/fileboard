#!/bin/bash
cp config.dep.toml target/release/config.toml
rsync -avz target/release/* server1:~/fileboard
ssh server1 systemctl --user restart fileboard
