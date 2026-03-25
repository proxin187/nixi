# nixi

nixi is a unix-like operating system written in rust for x86_64.

## Roadmap
- [X] physical memory allocator
- [X] memory allocator
- [X] tty over serial port
- [X] scheduler
- [ ] GOP graphics driver with bitmap font rendering
- [ ] block device driver
- [ ] filesystem
- [ ] syscalls
- [ ] elf loader

## Design
nixi is self-contained and does not use a traditional bootloader, the kernel itself is a UEFI binary and doesn't need a bootloader.

The kernel is identity mapped, and the userspace uses traditional paging to isolate process memory

## Development
In order to test nixi locally in qemu you must have [OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF) installed. There is a custom runner for the project, so you can build the kernel and execute it in qemu by running `cargo run`.

## License
nixi is licensed under the MIT license
