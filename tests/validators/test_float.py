import re

import pytest

from pydantic_core import ValidationError

from ..conftest import Err


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (0, 0),
        (1, 1),
        (42, 42),
        ('42', 42),
        (42.0, 42),
        (42.5, 42.5),
        (1e10, 1e10),
        (True, 1),
        (False, 0),
        ('wrong', Err('Value must be a valid number, unable to parse string as an number [kind=float_parsing')),
        ([1, 2], Err('Value must be a valid number [kind=float_type, input_value=[1, 2], input_type=list]')),
    ],
)
def test_float(py_or_json, input_value, expected):
    v = py_or_json({'type': 'float'})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_test(input_value)
    else:
        assert v.validate_test(input_value) == expected
