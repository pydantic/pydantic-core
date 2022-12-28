import inspect
import re
import sys
from io import StringIO
from tempfile import NamedTemporaryFile
from typing import Any, TextIO

import pytest

import pydantic_core

DOCSTRING_REGEX = r'```python(.*)```'


class DocstringTest:
    def method_a(self):
        """
        ```python
        assert 1 == 1
        assert 1 != 2
        ```
        """
        pass

    def method_b(self):
        """
        ```python
        print('hello')
        print('world')
        ```
        """
        pass


def write_docstrings_to_test_file(obj_with_docstrings: Any, f: TextIO):
    for name, obj in inspect.getmembers(obj_with_docstrings):
        if obj.__doc__ is not None:
            for i, match in enumerate(re.finditer(DOCSTRING_REGEX, obj.__doc__, re.DOTALL)):
                code = match.group(1)
                f.write(f'def test_{name}_{i}():\n')
                for line in code.splitlines():
                    if line.strip():
                        f.write(line)
                        f.write('\n')
                f.write('\n')
    f.flush()


def test_write_docstrings_to_test_file():
    with StringIO('') as f:
        write_docstrings_to_test_file(DocstringTest, f)
        assert (
            f.getvalue()
            == """def test_method_a_0():
        assert 1 == 1
        assert 1 != 2

def test_method_b_0():
        print('hello')
        print('world')

"""
        )


def test_docstrings():
    with NamedTemporaryFile('w', suffix='.py') as f:
        write_docstrings_to_test_file(pydantic_core.core_schema, f)
        exit_code = pytest.main([f.name])
        if exit_code != 0:
            sys.exit(exit_code)
