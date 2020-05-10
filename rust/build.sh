#!/bin/sh
tmux renamew -t $TMX_WINID building...
clear
if exectime cargo build --release; then
cp target/release/libdeso3d.so ../godot/lib/libdeso3d.so
fi

