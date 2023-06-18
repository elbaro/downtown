# downtown

![](./screenshot.png)

## Requirement
- Python built with --with-dtrace
    - on Linux, verify with `readelf -S ./python | grep .note.stapsdt`

## Usage

- `./downtown src/mycode.py`: monitor all python processes (`/usr/bin/env python`)
- `./downtown src/mycode.py --pid 123`: monitor /proc/123/exe
- `./downtown src/mycode.py --python-bin ./python`: monitor a python binary
- up, down - scroll
- enter - toggle

## BPF Probe test
```
> sudo  /usr/share/bcc/tools/tplist -l /home/user/projects/cpython/python
> sudo bpftrace -lv  # show args
> sudo bpftrace -l -p 146893 | rg usdt
> sudo bpftrace -e 'usdt:/home/user/projects/cpython/python:line { printf("%s %s %d\n", str(arg0), str(arg1), arg2); }'
> sudo bpftrace -e 'usdt:/home/user/projects/cpython/python:python:function__entry { printf("%s %s\n", str(arg0), str(arg1)); }'
```
