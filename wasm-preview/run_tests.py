import base64
import micropip
import importlib
import sys
from io import BytesIO
from zipfile import ZipFile

import pytest

# this seems to be required for me on M1 Mac
sys.setrecursionlimit(200)


async def main(tests_zip: str, pydantic_core_wheel_url: str):
    print('python running...')
    ZipFile(BytesIO(base64.b64decode(tests_zip))).extractall('.')
    print(f'Mounted test files, installing dependencies...')

    await micropip.install(['dirty-equals', 'hypothesis', 'pytest-speed', pydantic_core_wheel_url])
    importlib.invalidate_caches()

    # print('installed packages:')
    # print(micropip.list())
    print('Running tests...')
    pytest.main()

try:
    await main(tests_zip, '{{ pydantic_core_wheel_url }}')
except Exception as e:
    print(f'ERROR: {e}')
