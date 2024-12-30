# im_select

windows 下切换输入法的 `中`/`英` 模式

```bash
# 输出 
#     1：英文模式
#     2：中文模式
im_select

# 切换到英文模式
im_select 1

 # 切换到中文模式
im_select 2
```

## 从源码安装

测试

```bash
cargo test -- --test-threads=1
```

编译，二进制文件在：`.\target\release\im_select.exe`

```bash
cargo build --release
```
