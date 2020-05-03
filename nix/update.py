#!/usr/bin/env python3

"""
Fetch the latest commit info from upstream nixpkgs/mozilla-nixpkgs.
"""

import os
import subprocess
import sys
import json
from typing import Dict
from pathlib import Path

SCRIPT_DIR = Path(os.path.dirname(os.path.realpath(__file__)))

def get_github_commit_info(user: str, repo: str, branch: str) -> Dict[str, str]:
    parts = subprocess.check_output(['git', 'ls-remote', f"git@github.com:{user}/{repo}", branch]).split()

    commit = parts[0].decode()
    archive_url = f"https://github.com/{user}/{repo}/archive/{commit}.tar.gz"

    nix_hash = subprocess.check_output(['nix-prefetch-url', '--unpack', archive_url]).strip().decode()

    return {
        'url': archive_url,
        'sha256': nix_hash
    }


def update_upstream(user: str, repo: str, branch: str, output_name: str) -> None:
    print(f'Updating {user}/{repo}...')
    info = get_github_commit_info(user, repo, branch)
    with open(SCRIPT_DIR / f'{output_name}.json', mode='w', encoding='utf-8') as f:
        json.dump(info, f, indent=2)


if __name__ == '__main__':
    update_upstream('NixOS', 'nixpkgs', 'nixpkgs-unstable', 'nixpkgs')