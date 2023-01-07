@echo off
set make_action=%1
docker run --rm^
 -v %cd%:/tmp/builder/^
 --env RUSTFLAGS="--remap-path-prefix /tmp/builder=%cd% --remap-path-prefix /root=%USERPROFILE%"^
  os-builder make %make_action%