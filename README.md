# downtown

A realtime BPF profiler.

```
cargo install downtown
```

![](./screenshot.png)

v0.1.0 - supports Python.

## Usage
- `./downtown src/mycode.py`: monitor all python processes using the default python interpreter
- `./downtown src/mycode.py --pid 123`: monitor /proc/123/exe
- `./downtown src/mycode.py --python-bin ./python`: monitor all python processes running ./python
- up, down - scroll
- enter - toggle

## Requirement
- Python built with --with-dtrace
    - on Linux, verify with `readelf -S ./python | grep .note.stapsdt`

## Limitations
- async functions can be profiled, but they work [differently](https://github.com/elbaro/downtown/wiki/Profiling-Async-Functions).
