import inspect
import re
import sys
from tempfile import NamedTemporaryFile

import pytest

import pydantic_core

DOCSTRING_REGEX = r'```python(.*)```'


def write_docstrings_to_test_file(f: NamedTemporaryFile):
    for name, obj in inspect.getmembers(pydantic_core.core_schema):
        if obj.__doc__ is not None:
            for i, match in enumerate(re.finditer(DOCSTRING_REGEX, obj.__doc__, re.DOTALL)):
                code = match.group(1)
                f.write(f'def test_{name}_{i}():\n')
                f.write(code)
                f.write('\n\n')
    f.flush()


def test_docstrings():
    with NamedTemporaryFile('w', suffix='.py') as f:
        write_docstrings_to_test_file(f)
        exit_code = pytest.main([f.name])
        if exit_code != 0:
            sys.exit(exit_code)
