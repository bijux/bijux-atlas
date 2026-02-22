from __future__ import annotations
import subprocess


def main() -> int:
    return subprocess.call(['cargo','run','-p','bijux-atlas-cli','--bin','bijux-atlas','--','atlas','smoke','--root','artifacts/medium-output','--dataset','110/homo_sapiens/GRCh38'])

if __name__ == '__main__': raise SystemExit(main())
