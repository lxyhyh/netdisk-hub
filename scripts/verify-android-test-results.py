#!/usr/bin/env python3
"""校验 Android instrumentation 真的跑过、且全部通过。

判据（任一不满足即 exit 1）：
  1. 至少存在一个测试结果 XML；
  2. failures/errors/skipped 全为 0（跳过也算没跑）；
  3. 通过的用例数达到 EXPECTED_MIN 条；
  4. T-003 的 t01..t10 与 T-103 的 12 个 UseCase 用例名逐个出现。

只做校验，不改任何构建输入。
"""

import glob
import os
import sys
import xml.etree.ElementTree as ET

EXPECTED_MIN = 22

T003 = [
    "t01_nativeLibrariesAreLoaded",
    "t02_probeMagicReturnsAgreedValue",
    "t03_providerIdsAreExposed",
    "t04_twoClientsCanBeCreated",
    "t05_loginYieldsOpaqueSession",
    "t06_listPagingAndQuotaWork",
    "t07_fullFileOperationChain",
    "t08_betaScanLoginIsUnsupported",
    "t09_errorsCrossTheUniffiBoundary",
    "t10_credentialsNeverReachLogcat",
]

T103 = [
    "providersAreListedGenerically",
    "sameCallShapeWorksForBothProviders",
    "wrongPasswordBecomesAuthFailedError",
    "betaScanLoginBecomesUnsupportedError",
    "pagingWalksAllEntries",
    "fullFileOperationChain",
    "missingPathBecomesNotFound",
    "illegalPathBecomesInvalidArgument",
    "closedSessionBecomesInvalidSession",
    "mainThreadCallsAreRejected",
    "unknownProviderBecomesNotFound",
    "resultAccessorsBehaveAsDocumented",
]


def main() -> int:
    roots = [
        "app/build/outputs/androidTest-results",
        "app/build/reports/androidTests",
    ]
    xmls = []
    for root in roots:
        xmls.extend(glob.glob(os.path.join(root, "**", "*.xml"), recursive=True))

    if not xmls:
        print("::error::没有找到任何 instrumentation 测试结果 XML，设备测试可能根本没跑")
        return 1

    tests = failures = errors = skipped = 0
    names = set()
    for path in sorted(xmls):
        try:
            tree = ET.parse(path)
        except ET.ParseError as exc:
            print("::error::无法解析 %s: %s" % (path, exc))
            return 1
        root = tree.getroot()
        suites = [root] if root.tag == "testsuite" else root.findall("testsuite")
        for suite in suites:
            tests += int(suite.get("tests", "0") or 0)
            failures += int(suite.get("failures", "0") or 0)
            errors += int(suite.get("errors", "0") or 0)
            skipped += int(suite.get("skipped", "0") or 0)
            for case in suite.findall("testcase"):
                names.add(case.get("name", ""))

    print("结果文件 %d 个" % len(xmls))
    for path in sorted(xmls):
        print("  %s" % path)
    print("tests=%d failures=%d errors=%d skipped=%d" % (tests, failures, errors, skipped))

    rc = 0
    if failures or errors:
        print("::error::instrumentation 有失败用例")
        rc = 1
    if skipped:
        print("::error::instrumentation 有用例被跳过，算作没跑")
        rc = 1
    if tests < EXPECTED_MIN:
        print("::error::只跑了 %d 条用例，少于预期的 %d 条" % (tests, EXPECTED_MIN))
        rc = 1

    missing = [name for name in (T003 + T103) if name not in names]
    if missing:
        print("::error::缺少这些用例的结果: %s" % ", ".join(missing))
        rc = 1

    if rc == 0:
        print("OK: %d 条用例全部通过，T-003 十项与 T-103 十二项均在真实设备上执行" % tests)
    return rc


if __name__ == "__main__":
    sys.exit(main())
