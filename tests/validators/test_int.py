import re

import pytest

from pydantic_core import SchemaValidator, ValidationError

from ..conftest import Err


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (False, 0),
        (True, 1),
        (0, 0),
        ('0', 0),
        (1, 1),
        (42, 42),
        ('42', 42),
        (42.0, 42),
        (int(1e10), int(1e10)),
        pytest.param(
            12.5,
            Err('Value must be a valid integer, got a number with a fractional part [kind=int_from_float'),
            id='float-remainder',
        ),
        pytest.param(
            'wrong',
            Err('Value must be a valid integer, unable to parse string as an integer [kind=int_parsing'),
            id='string',
        ),
        pytest.param([1, 2], Err('Value must be a valid integer [kind=int_type'), id='list'),
    ],
)
def test_int_py_or_json(py_or_json, input_value, expected):
    v = py_or_json({'type': 'int'})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_test(input_value)
    else:
        assert v.validate_test(input_value) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        pytest.param(
            (1, 2),
            Err('Value must be a valid integer [kind=int_type, input_value=(1, 2), input_type=tuple]'),
            id='tuple',
        )
    ],
)
def test_int(input_value, expected):
    v = SchemaValidator({'type': 'int'})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        assert v.validate_python(input_value) == expected
