#!/bin/sh

set -e

cargo build --release

mkdir -p esp/EFI/BOOT

cp ../target/x86_64-unknown-uefi/release/nixi.efi esp/EFI/BOOT/BOOTX64.EFI

qemu-system-x86_64 \
  -M q35 \
  -m 512 \
  -nographic \
  -serial mon:stdio \
  -drive if=pflash,format=raw,unit=0,readonly=on,file=/usr/share/edk2/x64/OVMF_CODE.4m.fd \
  -drive if=pflash,format=raw,unit=1,file=./OVMF_VARS.fd \
  -drive format=raw,file=fat:rw:./esp,media=disk


