#!/usr/bin/env python3
"""Check the built Bigger Item Grids mod.

Every UI file ships whole: each MXML text source in src/mxml compiles
to a MBIN, and this decompiles every built MBIN and diffs it against
its committed source, which must round-trip exactly.

It also asserts every built file is fresh (newer than the script that
produced it: the build copies out of the persistent AMUMSS CreatedMODS
directory, so a silent no-op would validate last week's output) and
that no MXML source strays into the shipped mod.

Usage:
  verify_build.py --build BUILD --mod-set MODSET
      --mxml MXMLDIR --mbincompiler MBINCOMPILER
"""

import argparse
import difflib
import os
import subprocess
import sys
import tempfile


def decompile(mbincompiler, mbin_path, workdir):
    """Decompile one MBIN to MXML text, returning the text."""
    base = os.path.basename(mbin_path)
    tmp = os.path.join(workdir, base)
    with open(mbin_path, "rb") as src, open(tmp, "wb") as dst:
        dst.write(src.read())
    proc = subprocess.run(
        [mbincompiler, "-q", "-y", base],
        cwd=workdir,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        sys.exit("decompile failed for %s:\n%s" % (mbin_path, proc.stderr))
    mxml = os.path.join(workdir, os.path.splitext(base)[0] + ".MXML")
    if not os.path.isfile(mxml):
        sys.exit("decompile produced no MXML for %s" % mbin_path)
    with open(mxml, encoding="utf-8") as handle:
        return handle.read()


def find_built(moddir, name):
    """Locate the built MBIN for an MXML source by basename search."""
    want = os.path.splitext(name)[0] + ".MBIN"
    for root, _, files in os.walk(moddir):
        if want in files:
            return os.path.join(root, want)
    return None


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--build", required=True)
    parser.add_argument("--mod-set", required=True)
    parser.add_argument("--mxml", required=True)
    parser.add_argument("--mbincompiler", required=True)
    args = parser.parse_args()

    failures = []
    moddir = os.path.join(args.build, args.mod_set)
    script = os.path.join(args.build, "mod.lua")
    if not os.path.isfile(script):
        failures.append("missing staged script %s" % script)

    names = sorted(
        name for name in os.listdir(args.mxml) if name.endswith(".MXML")
    )
    if not names:
        failures.append("no MXML sources in %s" % args.mxml)

    with tempfile.TemporaryDirectory() as workdir:
        for name in names:
            with open(os.path.join(args.mxml, name), encoding="utf-8") as handle:
                committed = handle.read()
            built = find_built(moddir, name)
            if built is None:
                failures.append("MISSING built file for %s" % name)
                continue
            rel = os.path.relpath(built, moddir)
            if os.path.isfile(script) and os.path.getmtime(built) < os.path.getmtime(script):
                failures.append(
                    "%s is STALE - older than the script that should have produced it"
                    % rel
                )
                continue
            text = decompile(args.mbincompiler, built, workdir)
            if text != committed:
                diff = "".join(
                    difflib.unified_diff(
                        committed.splitlines(True),
                        text.splitlines(True),
                        "committed",
                        "built",
                    )
                )
                failures.append(
                    "%s differs from committed source:\n%s" % (rel, diff[:4000])
                )

    # No sources may ship in the mod folder.
    for root, _, files in os.walk(moddir):
        for name in files:
            if name.endswith(".MXML"):
                failures.append(
                    "stray MXML source ships in the mod: %s"
                    % os.path.relpath(os.path.join(root, name), moddir)
                )

    if failures:
        for failure in failures:
            print("BAD " + failure, file=sys.stderr)
        return 1

    print("verify: OK (%d whole files match)" % len(names))
    return 0


if __name__ == "__main__":
    sys.exit(main())
