"""
Logic required for running tests with webassembly/emscripten
"""
import json
import os
import subprocess
from pathlib import Path


def main():
    root_dir = Path(__file__).resolve().parent.parent
    with os.chdir(root_dir / 'node_modules' / 'pyodide'):
        subprocess.run(['node', '../prettier/bin-prettier.js', '-w', 'pyodide.asm.js'], check=True)
        with open('repodata.json') as f:
            emscripten_version = json.load(f)['info']['platform'].split('_', 1)[1].replace('_', '.')
            if github_env := os.getenv('GITHUB_ENV'):
                with open(github_env, 'w+') as f:
                    f.write(f'EMSCRIPTEN_VERSION={emscripten_version}\n')


if __name__ == '__main__':
    main()
