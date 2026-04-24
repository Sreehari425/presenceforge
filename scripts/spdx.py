#!/usr/bin/env python3

import os

SPDX_TAG = "// SPDX-License-Identifier: MIT OR Apache-2.0 \n"

COPYRIGHT = "// Copyright (c) 2025-2026 Sreehari Anil and project contributors\n\n"
EXTENSIONS = {'.rs' }

def stamp_files(root_dir):
    for root, _, files in os.walk(root_dir):
        for file in files:
            if any(file.endswith(ext) for ext in EXTENSIONS):
                file_path = os.path.join(root, file)

                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.readlines()

                if content and "SPDX-License-Identifier" in content[0]:
                    print(f"[-] Skipping: {file} (Already protected)")
                    continue

                print(f"[+] Stamping: {file}")
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(SPDX_TAG)
                    f.write(COPYRIGHT)
                    f.writelines(content)

if __name__ == "__main__":
    stamp_files("../")
