# nixi

nixi is a unix-like operating system.

## Roadmap
- [ ] builtin bootloader
- [X] physical memory allocator

## Design
nixi is self-contained and does not use a traditional bootloader, the kernel itself is a UEFI binary and doesn't need a bootloader.

The kernel is identity mapped, and the userspace uses traditional paging to isolate process memory

## Development
The script for running in qemu can be found under `dev/`. In order to run in qemu you must have [OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF) installed.

## License
nixi is licensed under the MIT license
