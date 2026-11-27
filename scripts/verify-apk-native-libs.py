#!/usr/bin/env python3
"""T-002e 第 3 轮：APK 原生库静态校验（ELF / 16 KB 页面 / 导出符号）。

用途
----
在 CI 里取代「肉眼看日志」，对 assembleDebug 产出的 APK 做可复现的断言：

1. APK 内 lib/ 下的文件清单必须与 EXPECTED_LIBS 完全一致
   （多一个、少一个、路径写错都失败）。
2. 每个 .so 必须是 ELFCLASS64 小端，且 e_machine 与所在 ABI 匹配。
3. 每个 PT_LOAD 段的 p_align 必须 >= 0x4000。
4. libnetdiskhub_core.so 的动态符号表必须含
   netdiskhub_probe / uniffi_netdiskhub_core_checksum_func_probe_magic /
   uniffi_netdiskhub_core_fn_func_probe_magic。

16 KB 判据的取舍（重要，别「加强」成别的）
------------------------------------------
官方 16 KB 页面要求是 PT_LOAD 段对齐到 16 KB，判定方法即
llvm-readelf -l <so> 里每个 LOAD 行的末列 >= 0x4000。
PT_GNU_RELRO 的结束地址不是官方判据：实测 androidx.graphics.path 1.0.1
自带的 arm64-v8a / x86_64 库 RELRO 结束地址分别是 0x6000 / 0x7000
（都不是 16 KB 的整数倍），而它们是 Google 自己随 AndroidX 发布的库。
因此本脚本对 RELRO 只输出 note，不判失败；也不要按 RELRO 的 p_align
判（实测该字段是 0x1 / 0x4 / 0x20 / 0x40，没有意义）。

为什么用 python 而不是 bash + llvm-readelf
-------------------------------------------
lib/ 清单严格比对、ELF 头与 program header 解析、dynsym 名称提取
用 shell 拼不出来；python3 在 ubuntu runner 上自带，无需额外安装。

用法
----
  python3 scripts/verify-apk-native-libs.py <apk>
  python3 scripts/verify-apk-native-libs.py <apk> --no-list-check

退出码 0 = 全部通过；1 = 有断言失败（每条失败都会打印原因）。
"""

from __future__ import annotations

import argparse
import struct
import sys
import zipfile

PAGE_16K = 0x4000

# 当前原生核心只支持 arm64-v8a 与 x86_64（ADR-004 + AGP abiFilters）。
# 清单写死是有意的：ABI 收敛后多打进任何一个库都说明 abiFilters 失效。
EXPECTED_LIBS = [
    "lib/arm64-v8a/libandroidx.graphics.path.so",
    "lib/arm64-v8a/libjnidispatch.so",
    "lib/arm64-v8a/libnetdiskhub_core.so",
    "lib/x86_64/libandroidx.graphics.path.so",
    "lib/x86_64/libjnidispatch.so",
    "lib/x86_64/libnetdiskhub_core.so",
]

ABI_MACHINE = {
    "arm64-v8a": (183, "EM_AARCH64"),
    "x86_64": (62, "EM_X86_64"),
}

REQUIRED_SYMBOLS = {
    "libnetdiskhub_core.so": [
        "netdiskhub_probe",
        "uniffi_netdiskhub_core_checksum_func_probe_magic",
        "uniffi_netdiskhub_core_fn_func_probe_magic",
    ]
}

PT_LOAD = 1
PT_GNU_RELRO = 0x6474E552
SHT_DYNSYM = 11
SHN_UNDEF = 0


class Elf:
    """最小 ELF64 解析器：只取 program header、section header、dynsym。"""

    def __init__(self, data: bytes) -> None:
        self.data = data
        if len(data) < 64 or data[:4] != b"\x7fELF":
            raise ValueError("不是 ELF 文件（magic 不匹配）")
        self.ei_class = data[4]
        self.ei_data = data[5]
        if self.ei_class != 2:
            raise ValueError(f"EI_CLASS={self.ei_class}（本工程只产出 ELFCLASS64=2）")
        if self.ei_data != 1:
            raise ValueError(f"EI_DATA={self.ei_data}（只接受小端 EI_DATA=1）")
        (self.e_type, self.e_machine) = struct.unpack_from("<HH", data, 16)
        self.e_phoff = struct.unpack_from("<Q", data, 32)[0]
        self.e_shoff = struct.unpack_from("<Q", data, 40)[0]
        (self.e_phentsize, self.e_phnum, self.e_shentsize, self.e_shnum) = struct.unpack_from(
            "<HHHH", data, 54
        )
        self.errors: list[str] = []
        self.loads: list[dict] = []
        self.sections: list[dict] = []
        self.relro: dict | None = None
        self._parse_phdrs()
        self._parse_shdrs()

    def _parse_phdrs(self) -> None:
        for i in range(self.e_phnum):
            off = self.e_phoff + i * self.e_phentsize
            if off + 56 > len(self.data):
                self.errors.append(f"program header #{i} 越界")
                break
            (p_type, p_flags, p_offset, p_vaddr, _p_paddr, p_filesz, p_memsz, p_align) = (
                struct.unpack_from("<IIQQQQQQ", self.data, off)
            )
            hdr = {
                "index": i,
                "type": p_type,
                "flags": p_flags,
                "offset": p_offset,
                "vaddr": p_vaddr,
                "filesz": p_filesz,
                "memsz": p_memsz,
                "align": p_align,
            }
            if p_type == PT_LOAD:
                self.loads.append(hdr)
            elif p_type == PT_GNU_RELRO:
                self.relro = hdr

    def _parse_shdrs(self) -> None:
        if self.e_shoff == 0 or self.e_shnum == 0:
            return
        for i in range(self.e_shnum):
            off = self.e_shoff + i * self.e_shentsize
            if off + 64 > len(self.data):
                break
            (sh_name, sh_type, _sh_flags, sh_addr, sh_offset, sh_size, sh_link, _sh_info,
             _sh_addralign, sh_entsize) = struct.unpack_from("<IIQQQQIIQQ", self.data, off)
            self.sections.append(
                {
                    "name_off": sh_name,
                    "type": sh_type,
                    "addr": sh_addr,
                    "offset": sh_offset,
                    "size": sh_size,
                    "link": sh_link,
                    "entsize": sh_entsize,
                }
            )

    def check_load_alignment(self) -> list[str]:
        fails = []
        if not self.loads:
            fails.append("没有任何 PT_LOAD 段")
        for seg in self.loads:
            if seg["align"] < PAGE_16K:
                fails.append(
                    f"PT_LOAD #{seg['index']} p_align=0x{seg['align']:x} < 0x4000（16 KB 页面要求）"
                )
        return fails

    def relro_note(self) -> str:
        if self.relro is None:
            return "无 PT_GNU_RELRO 段"
        end = self.relro["vaddr"] + self.relro["memsz"]
        aligned = "是" if end % PAGE_16K == 0 else "否"
        return (
            f"PT_GNU_RELRO vaddr=0x{self.relro['vaddr']:x} memsz=0x{self.relro['memsz']:x}"
            f" end=0x{end:x}（16 KB 对齐：{aligned}，非判据，仅记录）"
        )

    def dynamic_symbols(self) -> set[str]:
        """返回已定义（非 SHN_UNDEF）的动态符号名集合。"""
        symtab = next((s for s in self.sections if s["type"] == SHT_DYNSYM), None)
        if symtab is None or symtab["link"] >= len(self.sections):
            return set()
        strtab = self.sections[symtab["link"]]
        entsize = symtab["entsize"] or 24
        names: set[str] = set()
        count = symtab["size"] // entsize
        for i in range(count):
            off = symtab["offset"] + i * entsize
            if off + 24 > len(self.data):
                break
            (st_name, _st_info, _st_other, st_shndx, _st_value, _st_size) = struct.unpack_from(
                "<IBBHQQ", self.data, off
            )
            if st_shndx == SHN_UNDEF or st_name == 0:
                continue
            start = strtab["offset"] + st_name
            stop = self.data.find(b"\x00", start)
            if stop < 0:
                continue
            names.add(self.data[start:stop].decode("utf-8", "replace"))
        return names


def abi_of(name: str) -> str:
    parts = name.split("/")
    return parts[1] if len(parts) > 2 else ""


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("apk")
    ap.add_argument(
        "--no-list-check",
        action="store_true",
        help="跳过 lib/ 文件清单严格比对（仅用于本地调试旧 APK）",
    )
    args = ap.parse_args()

    failures: list[str] = []
    with zipfile.ZipFile(args.apk) as zf:
        libs = sorted(n for n in zf.namelist() if n.startswith("lib/") and not n.endswith("/"))
        print(f"APK = {args.apk}")
        print(f"APK 内 lib/ 条目数 = {len(libs)}")
        for n in libs:
            info = zf.getinfo(n)
            print(
                f"  {n}  {info.file_size} B  compress_type={info.compress_type}"
                f"  (0=STORED 存储不压缩)"
            )

        if not args.no_list_check:
            expected = sorted(EXPECTED_LIBS)
            if libs != expected:
                extra = [n for n in libs if n not in expected]
                missing = [n for n in expected if n not in libs]
                for n in extra:
                    failures.append(f"APK 内多出未预期的原生库: {n}")
                for n in missing:
                    failures.append(f"APK 缺少预期的原生库: {n}")
                if not extra and not missing:
                    failures.append("lib/ 清单与预期集合相同但顺序不一致")
            else:
                print(f"  OK    lib/ 清单与预期 {len(expected)} 条完全一致")

        print("\n=== 逐库 ELF 检查 ===")
        for name in libs:
            print(f"\n-- {name}")
            abi = abi_of(name)
            data = zf.read(name)
            try:
                elf = Elf(data)
            except ValueError as exc:
                failures.append(f"{name}: {exc}")
                print(f"   FAIL  {exc}")
                continue

            print(
                f"   ELFCLASS64 machine={elf.e_machine} load_seg={len(elf.loads)}"
                f" sections={len(elf.sections)} size={len(data)} B"
            )

            want = ABI_MACHINE.get(abi)
            if want is None:
                failures.append(f"{name}: 未预期的 ABI 目录 {abi!r}")
            elif elf.e_machine != want[0]:
                failures.append(
                    f"{name}: e_machine={elf.e_machine}，但 {abi} 要求 {want[0]}（{want[1]}）"
                )
            else:
                print(f"   OK    e_machine={want[1]}")

            for seg in elf.loads:
                print(
                    f"   LOAD #{seg['index']} vaddr=0x{seg['vaddr']:x}"
                    f" filesz=0x{seg['filesz']:x} memsz=0x{seg['memsz']:x}"
                    f" align=0x{seg['align']:x}"
                )
            load_fails = elf.check_load_alignment()
            for e in load_fails:
                print(f"   FAIL  {e}")
            failures.extend(f"{name}: {e}" for e in load_fails)
            if not load_fails:
                print("   OK    全部 PT_LOAD p_align >= 0x4000")
            print(f"   note  {elf.relro_note()}")

            base = name.rsplit("/", 1)[-1]
            required = REQUIRED_SYMBOLS.get(base, [])
            if required:
                defined = elf.dynamic_symbols()
                if not defined:
                    failures.append(f"{name}: 读不到已定义的动态符号")
                else:
                    for sym in required:
                        if sym in defined:
                            print(f"   OK    dynsym {sym}")
                        else:
                            failures.append(f"{name}: 动态符号表缺少 {sym}")
                    print(f"   note  已定义动态符号数={len(defined)}")

    print("\n=== ZIP 存储与对齐 ===")
    with zipfile.ZipFile(args.apk) as zf:
        for name in sorted(n for n in zf.namelist() if n.startswith("lib/") and not n.endswith("/")):
            info = zf.getinfo(name)
            if info.compress_type != zipfile.ZIP_STORED:
                failures.append(
                    f"{name}: compress_type={info.compress_type}，原生库必须是 STORED(0)"
                    f"（压缩后无法按页对齐映射）"
                )
                print(f"   FAIL  {name} compress_type={info.compress_type} != STORED")
                continue
            with open(args.apk, "rb") as fh:
                fh.seek(info.header_offset)
                local = fh.read(30)
                if len(local) < 30 or local[:4] != b"PK\x03\x04":
                    failures.append(f"{name}: 本地文件头损坏")
                    continue
                name_len = struct.unpack_from("<H", local, 26)[0]
                extra_len = struct.unpack_from("<H", local, 28)[0]
                data_off = info.header_offset + 30 + name_len + extra_len
            if data_off % PAGE_16K != 0:
                failures.append(
                    f"{name}: 数据偏移 0x{data_off:x} 未按 16 KB 对齐（0x4000）"
                )
                print(f"   FAIL  {name} data_offset=0x{data_off:x} % 0x4000 = 0x{data_off % PAGE_16K:x}")
            else:
                print(f"   OK    {name} STORED data_offset=0x{data_off:x}（16 KB 对齐）")

    print("\n=== 结果 ===")
    if failures:
        for f in failures:
            print(f"FAILED: {f}")
        print(f"共 {len(failures)} 项失败")
        return 1
    print(
        f"PASS: {len(EXPECTED_LIBS)} 个原生库通过 "
        f"清单 / ELF / PT_LOAD 16 KB 对齐 / ZIP 页对齐 / 符号 检查"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
