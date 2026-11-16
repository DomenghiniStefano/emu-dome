
### v-

| Run | Status | Backend, mode | Core FPS | Present/s | New frames/s | New-frame interval ms p50/p95/p99/max | Present p99 ms | Dropped | Off >25% | CPU % (1 core) | CPU % tree | First frame ms | Surface errors |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| v-default-1 | ok | Vulkan AutoVsync | 59.73 | 239.7 | 59.77 | 16.64 / 20.51 / 21.59 / 37.81 | 5.45 | 0 | 106 | 11.0 | 11.2 | 834 | 0 |
| v-default-2 | ok | Vulkan AutoVsync | 59.70 | 239.7 | 59.70 | 16.67 / 20.41 / 21.59 / 54.04 | 5.51 | 2 | 93 | 10.5 | 11.8 | 903 | 0 |
| v-dx12hwnd-autovsync-l1 | ok | Dx12 AutoVsync | 59.70 | 239.0 | 59.57 | 16.67 / 20.79 / 21.85 / 50.35 | 7.36 | 5 | 118 | 8.3 | 9.2 | 517 | 0 |
| v-dx12hwnd-autovsync-l2 | ok | Dx12 AutoVsync | 59.73 | 240.1 | 59.77 | 16.66 / 16.79 / 20.84 / 21.08 | 4.34 | 0 | 23 | 7.3 | 8.7 | 519 | 0 |
| v-dx12hwnd-fifo-l1 | ok | Dx12 Fifo | 59.70 | 239.2 | 59.77 | 16.66 / 17.05 / 20.87 / 25.70 | 4.68 | 0 | 42 | 7.6 | 8.9 | 605 | 0 |
| v-dx12hwnd-fifo-l2 | ok | Dx12 Fifo | 59.70 | 240.0 | 59.77 | 16.66 / 17.00 / 20.87 / 21.15 | 4.58 | 0 | 34 | 8.2 | 8.5 | 519 | 0 |
| v-dx12hwnd-immediate-l1 | ok | Dx12 Immediate | 59.73 | 939.0 | 59.77 | 17.25 / 19.82 / 20.98 / 21.80 | 3.09 | 0 | 55 | 19.0 | 20.4 | 506 | 0 |
| v-dx12hwnd-immediate-l2 | ok | Dx12 Immediate | 59.73 | 1833.4 | 59.70 | 16.70 / 18.67 / 19.32 / 47.62 | 1.70 | 1 | 4 | 28.1 | 28.6 | 527 | 0 |
| v-dx12hwnd-mailbox-l1 | ok | Dx12 Mailbox | 59.70 | 890.0 | 59.73 | 16.71 / 20.34 / 21.20 / 27.53 | 3.10 | 0 | 64 | 18.2 | 19.2 | 506 | 0 |
| v-dx12hwnd-mailbox-l2 | ok | Dx12 Mailbox | 59.70 | 1903.2 | 59.73 | 16.69 / 18.60 / 19.17 / 19.73 | 1.63 | 0 | 0 | 30.7 | 31.4 | 508 | 0 |
| v-dx12visual-autovsync-l1 | ok | Dx12 AutoVsync | 59.73 | 240.1 | 59.73 | 16.66 / 20.81 / 20.88 / 23.06 | 8.18 | 0 | 117 | 7.9 | 8.5 | 772 | 0 |
| v-dx12visual-autovsync-l2 | ok | Dx12 AutoVsync | 59.70 | 240.1 | 59.73 | 16.66 / 16.98 / 20.83 / 33.43 | 4.54 | 1 | 25 | 7.4 | 7.8 | 610 | 0 |
| v-dx12visual-fifo-l1 | ok | Dx12 Fifo | 59.73 | 239.7 | 59.77 | 16.66 / 20.76 / 20.89 / 34.61 | 7.06 | 0 | 79 | 8.7 | 9.7 | 485 | 0 |
| v-dx12visual-fifo-l2 | ok | Dx12 Fifo | 59.73 | 240.1 | 59.73 | 16.66 / 16.91 / 20.84 / 21.04 | 4.66 | 0 | 25 | 7.1 | 7.8 | 530 | 0 |
| v-dx12visual-immediate-l1 | ok | Dx12 Immediate | 59.70 | 5485.4 | 59.73 | 16.71 / 17.42 / 17.85 / 25.20 | 0.89 | 0 | 2 | 69.9 | 69.5 | 659 | 0 |
| v-dx12visual-immediate-l2 | ok | Dx12 Immediate | 59.73 | 8497.9 | 59.77 | 16.72 / 17.39 / 17.97 / 20.21 | 0.78 | 0 | 1 | 80.7 | 80.6 | 676 | 0 |
| v-dx12visual-mailbox-l1 | ok | Dx12 Mailbox | 59.70 | 4898.2 | 59.73 | 16.72 / 17.50 / 18.04 / 20.64 | 1.01 | 0 | 0 | 67.1 | 67.5 | 675 | 0 |
| v-dx12visual-mailbox-l2 | ok | Dx12 Mailbox | 59.73 | 8415.2 | 59.77 | 16.71 / 17.35 / 17.85 / 21.13 | 0.77 | 0 | 3 | 81.4 | 81.5 | 792 | 0 |
| v-gl-autovsync-l1 | ok | Gl AutoVsync | 59.73 | 237.1 | 59.73 | 16.66 / 20.77 / 23.11 / 30.67 | 6.65 | 0 | 83 | 55.7 | 55.0 | 393 | 0 |
| v-gl-autovsync-l2 | ok | Gl AutoVsync | 59.73 | 237.6 | 59.73 | 16.66 / 20.79 / 22.68 / 45.79 | 6.67 | 1 | 89 | 50.4 | 53.8 | 397 | 0 |
| v-gl-fifo-l1 | ok | Gl Fifo | 59.70 | 237.8 | 59.73 | 16.66 / 20.75 / 21.41 / 29.13 | 6.58 | 1 | 78 | 54.2 | 55.1 | 944 | 0 |
| v-gl-fifo-l2 | ok | Gl Fifo | 59.73 | 237.7 | 59.73 | 16.66 / 20.75 / 21.46 / 29.11 | 6.46 | 0 | 75 | 50.2 | 51.7 | 399 | 0 |
| v-gl-immediate-l1 | ok | Gl Immediate | 59.73 | 2922.4 | 59.77 | 16.72 / 17.29 / 17.59 / 25.57 | 0.67 | 0 | 7 | 96.6 | 98.0 | 380 | 0 |
| v-gl-immediate-l2 | ok | Gl Immediate | 59.73 | 2963.7 | 59.77 | 16.72 / 17.28 / 17.60 / 20.15 | 0.66 | 0 | 1 | 97.6 | 97.5 | 389 | 0 |
| v-gl-mailbox-l1 | unsupported | Gl | present mode Mailbox not supported by this surface (supported: Fifo, Immediate) | | | | | | | | | | |
| v-gl-mailbox-l2 | unsupported | Gl | present mode Mailbox not supported by this surface (supported: Fifo, Immediate) | | | | | | | | | | |
| v-vulkan-autovsync-l1 | ok | Vulkan AutoVsync | 59.73 | 238.6 | 59.70 | 16.66 / 20.70 / 21.60 / 26.12 | 5.98 | 1 | 139 | 10.6 | 11.3 | 610 | 0 |
| v-vulkan-autovsync-l2 | ok | Vulkan AutoVsync | 59.73 | 240.0 | 59.77 | 16.67 / 20.51 / 21.46 / 22.06 | 5.27 | 0 | 105 | 10.9 | 11.3 | 623 | 0 |
| v-vulkan-fifo-l1 | ok | Vulkan Fifo | 59.73 | 240.0 | 59.77 | 16.67 / 20.49 / 21.29 / 23.05 | 5.38 | 0 | 99 | 10.7 | 11.8 | 658 | 0 |
| v-vulkan-fifo-l2 | ok | Vulkan Fifo | 59.73 | 239.4 | 59.67 | 16.67 / 20.59 / 21.63 / 49.99 | 5.93 | 3 | 118 | 11.2 | 12.4 | 629 | 0 |
| v-vulkan-immediate-l1 | ok | Vulkan Immediate | 59.73 | 1336.5 | 59.77 | 16.73 / 18.73 / 19.20 / 20.25 | 1.89 | 0 | 1 | 24.0 | 24.3 | 461 | 0 |
| v-vulkan-immediate-l2 | ok | Vulkan Immediate | 59.73 | 2306.3 | 59.77 | 16.74 / 17.93 / 18.47 / 25.92 | 1.86 | 0 | 8 | 32.5 | 34.2 | 471 | 0 |
| v-vulkan-mailbox-l1 | ok | Vulkan Mailbox | 59.70 | 1285.1 | 59.77 | 16.73 / 18.64 / 19.22 / 19.93 | 1.85 | 0 | 1 | 22.8 | 23.2 | 474 | 0 |
| v-vulkan-mailbox-l2 | ok | Vulkan Mailbox | 59.73 | 2156.8 | 59.70 | 16.71 / 17.91 / 18.38 / 58.97 | 1.79 | 2 | 8 | 32.0 | 33.0 | 455 | 0 |
| v-webview-1 | ok | webview  | 59.70 | 239.9 | 59.73 | 16.70 / 19.90 / 20.90 / 26.60 | 6.30 | 0 | 33 | 8.4 | 44.1 | 301 | 0 |
| v-webview-2 | ok | webview  | 59.70 | 240.1 | 59.73 | 16.70 / 20.00 / 20.90 / 22.80 | 6.30 | 0 | 32 | 9.8 | 49.2 | 293 | 0 |

### m-

| Run | Status | Backend, mode | Core FPS | Present/s | New frames/s | New-frame interval ms p50/p95/p99/max | Present p99 ms | Dropped | Off >25% | CPU % (1 core) | CPU % tree | First frame ms | Surface errors |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| m-default-nomenu | ok | Vulkan AutoVsync | 59.66 | 239.6 | 59.83 | 16.66 / 20.82 / 20.97 / 27.15 | 7.65 | 0 | 26 | 9.4 | 14.0 | 736 | 0 |
| m-dx12hwnd | ok | Dx12 AutoVsync | 0.17 | 240.0 | 0.33 | 4.09 / 4.09 / 4.09 / 4.09 | 6.21 | 0 | 1 | 6.8 | 8.9 | 449 | 0 |
| m-dx12visual | ok | Dx12 AutoVsync | 0.00 | 239.5 | 0.17 | - / - / - / - | 6.19 | 0 | 0 | 4.9 | 6.2 | 459 | 0 |
| m-gl | ok | Gl AutoVsync | 0.00 | 237.3 | 0.17 | - / - / - / - | 6.81 | 0 | 0 | 21.9 | 24.1 | 391 | 0 |
| m-vulkan | ok | Vulkan AutoVsync | 0.00 | 240.0 | 0.17 | - / - / - / - | 6.61 | 0 | 0 | 10.2 | 11.0 | 925 | 0 |
| m-webview | ok | webview  | 0.00 | 239.2 | 0.33 | 20.70 / 20.70 / 20.70 / 20.70 | 5.70 | 0 | 0 | 5.7 | 36.8 | 300 | 0 |
