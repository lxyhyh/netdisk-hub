#!/usr/bin/env python3
"""T-002e 第 3 轮：APK 原生库静态校验（ELF / 16 KB 页面 / RELRO / 导出符号）。

用途
----
在 CI 里取代「肉眼看日志」，对 assembleDebug 产出的 APK 做可复现的断言：

1. APK 内 lib/ 下的文件清单必须与 EXPECTED_LIBS 完全一致
   （多一个、少一个、路径写错都失败）。
2. 每个 .so 必须是 ELFCLASS64 小端，且 e_machine 与所在 ABI 匹配。
3. 每个 PT_LOAD 段的 p_align 必须 >= 0x4000。
4. 按 Android 官方 16 KB 页面文档实现 GNU_RELRO 兼容性断言（见下）。
5. libnetdiskhub_core.so 的动态符号表必须含
   netdiskhub_probe / uniffi_netdiskhub_core_checksum_func_probe_magic /
   uniffi_netdiskhub_core_fn_func_probe_magic。
6. ZIP 层的 lib/ 条目必须 STORED，且数据偏移按 16 KB 对齐。

RELRO 判据（严格照官方文档，不做豁免）
-------------------------------------
来源：https://developer.android.com/guide/practices/page-sizes
「Step 2: Verify 16 KB compliance for RELRO」原文给出的三个条件，
满足任意一条即合规：

  A. program header 里没有 GNU_RELRO 段；或
  B. (RELRO.VirtAddr + RELRO.MemSiz) % 0x4000 == 0；或
  C. RELRO 之后在同一 LOAD 段内没有可写数据
     （官方原文：GNU_RELRO occupies the entire remainder of its containing
     LOAD segment，即 LOAD.VirtAddr + LOAD.MemSiz == RELRO.VirtAddr + RELRO.MemSiz）。

本脚本对 C 的实现分两档，顺序判定：

  C1（官方原文的等式）：存在某个 PT_LOAD 满足
     LOAD.VirtAddr + LOAD.MemSiz == RELRO.VirtAddr + RELRO.MemSiz → 合规。
  C2（C 的实质化，官方 Example 1 / Example 4 的判别依据）：把 RELRO 覆盖范围
     按 16 KB 向上/向下取整成页区间 P，P 内 RELRO 之外不得存在任何可写字节
     （来自任何 PF_W 的 PT_LOAD 段）。因为运行时 linker 会把 RELRO 保护范围
     按 16 KB 页边界取整，P 内 RELRO 之外的任何可写数据都会被错误地变成只读，
     这正是官方文档描述的 SIGSEGV 场景。

判定不了的情况一律失败，不静默通过：RELRO 的起始地址不落在任何 PT_LOAD 内时，
无法确定它与 LOAD 段的关系，直接报错。

官方文档 Example 1 / 2 / 3 / 4 已写成 --selftest 的断言（见 selftest()），
用合成 ELF 逐条回归，确保上面的实现与官方示例一一对应。

为什么用 python 而不是 bash + llvm-readelf
------------------------------------------
lib/ 清单严格比对、ELF 头与 program header 解析、dynsym 名称提取、
RELRO 与 LOAD 的区间运算用 shell 拼不出来；python3 在 ubuntu runner 上自带，
无需额外安装。CI 里另有官方 zipalign 交叉验证（build.yml 的独立步骤）。

用法
----
  python3 scripts/verify-apk-native-libs.py <apk>
  python3 scripts/verify-apk-native-libs.py <apk> --no-list-check
  python3 scripts/verify-apk-native-libs.py --selftest

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
PF_W = 2


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

    def check_relro(self) -> tuple[list[str], list[str]]:
        """按官方 16 KB 文档判定 GNU_RELRO 兼容性。

        返回 (failures, notes)。failures 非空即不合规；notes 是给人看的判定过程。
        """
        fails: list[str] = []
        notes: list[str] = []
        rel = self.relro
        if rel is None:
            notes.append("官方条件 A：program header 无 PT_GNU_RELRO 段 → 合规")
            return fails, notes

        rva = rel["vaddr"]
        rsz = rel["memsz"]
        rend = rva + rsz
        notes.append(
            f"PT_GNU_RELRO vaddr=0x{rva:x} memsz=0x{rsz:x} filesz=0x{rel['filesz']:x}"
            f" end=0x{rend:x}"
        )

        # 条件 B：结束地址 16 KB 对齐
        if rend % PAGE_16K == 0:
            notes.append(
                f"官方条件 B：({rend - rsz:#x} + {rsz:#x}) % 0x4000 == 0 → 合规（结束地址 16 KB 对齐）"
            )
            return fails, notes
        notes.append(
            f"官方条件 B 不满足：0x{rend:x} % 0x4000 = 0x{rend % PAGE_16K:x} ≠ 0"
        )

        # 关系判定：RELRO 的起始地址必须落在某个 PT_LOAD 内，否则判定不了
        host = [s for s in self.loads if s["vaddr"] <= rva < s["vaddr"] + s["memsz"]]
        if not host:
            fails.append(
                f"RELRO vaddr=0x{rva:x} 不落在任何 PT_LOAD 段内，无法判定其与 LOAD 段的关系"
                f"（按规则不得静默通过）"
            )
            return fails, notes
        notes.append(
            "所在 LOAD 段：" + "、".join(
                f"#{s['index']} [0x{s['vaddr']:x}, 0x{s['vaddr'] + s['memsz']:x})"
                for s in host
            )
        )

        # 条件 C1：官方等式的直接命中
        exact = [s for s in host if s["vaddr"] + s["memsz"] == rend]
        if exact:
            s = exact[0]
            notes.append(
                f"官方条件 C1：LOAD#{s['index']} 的结束地址 0x{s['vaddr'] + s['memsz']:x}"
                f" == RELRO 结束地址 0x{rend:x}，RELRO 占满该段余部 → 合规"
            )
            return fails, notes
        notes.append("官方条件 C1 不满足：没有任何所在 LOAD 段在 RELRO 结束处结束")

        # 条件 C2：按 16 KB 页取整后，页区间内 RELRO 之外不得有可写字节
        page_lo = (rva // PAGE_16K) * PAGE_16K
        page_hi = ((rend + PAGE_16K - 1) // PAGE_16K) * PAGE_16K
        notes.append(
            f"官方条件 C2 判定：linker 会把 RELRO 保护范围按 16 KB 页取整为"
            f" [0x{page_lo:x}, 0x{page_hi:x})，检查该区间内 RELRO 之外是否存在可写字节"
        )
        offenders: list[tuple[int, int, int]] = []
        for seg in self.loads:
            if not (seg["flags"] & PF_W):
                continue
            seg_lo = seg["vaddr"]
            seg_hi = seg["vaddr"] + seg["memsz"]
            lo = max(seg_lo, page_lo)
            hi = min(seg_hi, page_hi)
            if lo >= hi:
                continue
            # 去掉 RELRO 自身覆盖的部分
            if lo < rva:
                offenders.append((seg["index"], lo, min(hi, rva)))
            if hi > rend:
                offenders.append((seg["index"], max(lo, rend), hi))
        offenders = [o for o in offenders if o[1] < o[2]]
        if offenders:
            for idx, lo, hi in offenders:
                fails.append(
                    f"RELRO 结束地址 0x{rend:x} 未 16 KB 对齐，且同一 16 KB 页内"
                    f" 0x{lo:x}–0x{hi:x} 仍有可写数据（来自 LOAD#{idx}）"
                    f"：进入 16 KB 设备后该区域会被误置只读，写入时 SIGSEGV"
                )
            return fails, notes
        notes.append(
            f"官方条件 C2 满足：页区间 [0x{page_lo:x}, 0x{page_hi:x}) 内 RELRO 之外"
            f" 没有任何可写字节 → 合规"
        )
        return fails, notes

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


def synthetic_elf(segments: list[tuple[int, int, int, int, int, int]]) -> bytes:
    """构造只含 program header 的最小 ELF64 镜像，用于 --selftest。

    segments 每项 = (p_type, p_flags, p_vaddr, p_filesz, p_memsz, p_align)。
    """
    ident = b"\x7fELF" + bytes([2, 1, 1, 0]) + b"\x00" * 8
    phoff = 64
    body = b"".join(
        struct.pack("<IIQQQQQQ", p_type, p_flags, 0, vaddr, vaddr, filesz, memsz, align)
        for (p_type, p_flags, vaddr, filesz, memsz, align) in segments
    )
    header = ident + struct.pack(
        "<HHIQQQIHHHHHH",
        3,                      # e_type = ET_DYN
        183,                    # e_machine = EM_AARCH64
        1,                      # e_version
        0,                      # e_entry
        phoff,                  # e_phoff
        0,                      # e_shoff
        0,                      # e_flags
        64,                     # e_ehsize
        56,                     # e_phentsize
        len(segments),          # e_phnum
        64,                     # e_shentsize
        0,                      # e_shnum
        0,                      # e_shstrndx
    )
    return header + body


def selftest() -> int:
    """把官方文档 Example 1–4 逐条回归，证明判定逻辑与官方一致。"""

    cases = [
        (
            "官方 Example 1（不合规）",
            False,
            [
                (PT_LOAD, 5, 0x0, 0x0CFAF0, 0x0CFAF0, 0x4000),
                (PT_LOAD, 6, 0x0DFAF0, 0x02510, 0x04510, 0x4000),
                (PT_GNU_RELRO, 4, 0x0DFAF0, 0x01510, 0x01510, 1),
            ],
        ),
        (
            "官方 Example 2（结束地址对齐，合规）",
            True,
            [
                (PT_LOAD, 5, 0x0, 0x0CFAF0, 0x0CFAF0, 0x4000),
                (PT_LOAD, 6, 0x0DFAF0, 0x01510, 0x03510, 0x4000),
                (PT_GNU_RELRO, 4, 0x0DFAF0, 0x00510, 0x00510, 1),
            ],
        ),
        (
            "官方 Example 3（无 RELRO，合规）",
            True,
            [
                (PT_LOAD, 5, 0x0, 0x000680, 0x000680, 0x4000),
                (PT_LOAD, 6, 0x004680, 0x0001D8, 0x0001D8, 0x4000),
            ],
        ),
        (
            "官方 Example 4（RELRO 独占 LOAD 段，合规）",
            True,
            [
                (PT_LOAD, 5, 0x0, 0x000680, 0x000680, 0x4000),
                (PT_LOAD, 6, 0x004680, 0x0001D8, 0x000980, 0x4000),
                (PT_GNU_RELRO, 4, 0x004680, 0x0001D8, 0x000980, 1),
            ],
        ),
        (
            "RELRO 不落在任何 LOAD 段内（必须失败，不得静默通过）",
            False,
            [
                (PT_LOAD, 5, 0x0, 0x000680, 0x000680, 0x4000),
                (PT_GNU_RELRO, 4, 0x009000, 0x000010, 0x000010, 1),
            ],
        ),
    ]

    bad = 0
    for label, want_pass, segs in cases:
        elf = Elf(synthetic_elf(segs))
        fails, _notes = elf.check_relro()
        got_pass = not fails
        ok = got_pass == want_pass
        print(f"{'OK  ' if ok else 'FAIL'} {label}: 期望{'合规' if want_pass else '不合规'}，"
              f"实测{'合规' if got_pass else '不合规'}")
        if not ok:
            bad += 1
            for f in fails:
                print(f"       实际失败信息：{f}")
            for n in _notes:
                print(f"       判定过程：{n}")
    if bad:
        print(f"\nselftest 失败 {bad} 项")
        return 1
    print(f"\nselftest 通过：{len(cases)} 个官方示例判定一致")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("apk", nargs="?")
    ap.add_argument(
        "--no-list-check",
        action="store_true",
        help="跳过 lib/ 文件清单严格比对（仅用于本地调试旧 APK）",
    )
    ap.add_argument(
        "--selftest",
        action="store_true",
        help="只跑官方 16 KB 文档示例的回归，不读 APK",
    )
    args = ap.parse_args()

    if args.selftest:
        return selftest()
    if not args.apk:
        ap.error("需要 APK 路径（或使用 --selftest）")

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
                    f" flags={seg['flags']} align=0x{seg['align']:x}"
                )
            load_fails = elf.check_load_alignment()
            for e in load_fails:
                print(f"   FAIL  {e}")
            failures.extend(f"{name}: {e}" for e in load_fails)
            if not load_fails:
                print("   OK    全部 PT_LOAD p_align >= 0x4000")

            relro_fails, relro_notes = elf.check_relro()
            for note in relro_notes:
                print(f"   note  {note}")
            for e in relro_fails:
                print(f"   FAIL  {e}")
            failures.extend(f"{name}: {e}" for e in relro_fails)
            if not relro_fails:
                print("   OK    GNU_RELRO 满足官方 16 KB 条件")

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
        f"清单 / ELF / PT_LOAD 16 KB 对齐 / GNU_RELRO 官方条件 / ZIP 页对齐 / 符号 检查"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
