# EN-OS-Zram-Manager
Universal ZRam manager for EN-OS.
Version: 1.0

ZRam-Manager works on both EN-OS and other distributions, including Arch Linux, Ubuntu, and any others that support ZRam and Systemd.

Usage:
*  `zram-manager`: show zram status (algorithm, size, mountpoint), memory and CPU usage.
*  `zram-manager -h (--help)`: show available arguments.
*  `sudo zram-manager --install`: calculate optimal ZRam size and compress algorithm in relation to your CPU power and size of your memory.
*  `sudo zram-manager --install -a (--alg) -g (--gb)`: choose manually compress algorithm or ZRam size.
*  `sudo zram-manager --uninstall`: remove ZRam service

*  `sudo zram-manager -a (--alg) -g (--gb)`: Start ZRam with custom algorithm and ZRam size. ZRam will be reset after reboot.

Screenshots

<div align="center">

| zram-manager | zram-manager --install |
| :---: | :---: |
| ![zram-manager](https://github.com/Endscape-Coding/EN-OS-Zram-Manager/blob/main/images/zram-manager.png) | ![zram-manager --install](https://github.com/Endscape-Coding/EN-OS-Zram-Manager/blob/main/images/zram-manager1.png) |

| zram-manager --install | zram-manager --help |
| :---: | :---: |
| ![zram-manager --uninstall](https://github.com/Endscape-Coding/EN-OS-Zram-Manager/blob/main/images/zram-manager2.png) | ![zram-manager --help](https://github.com/Endscape-Coding/EN-OS-Zram-Manager/blob/main/images/zram-manager3.png) |

</div>
