import re

import pytest

from pydantic_core import ValidationError

from ..conftest import Err, PyAndJson


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ([1, 2, 3], [1, 2, 3]),
        ([1, 2, '3'], [1, 2, 3]),
        ({1: 2, 3: 4}, [1, 3]),
        ('123', [1, 2, 3]),
        (5, Err('Input should be iterable [kind=iterable_type, input_value=5, input_type=int]')),
        (
            [1, 'wrong'],
            Err(
                'Input should be a valid integer, unable to parse string as an integer '
                "[kind=int_parsing, input_value='wrong', input_type=str]"
            ),
        ),
    ],
    ids=repr,
)
def test_generator_json_int(py_and_json: PyAndJson, input_value, expected):
    v = py_and_json({'type': 'generator', 'items_schema': {'type': 'int'}})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            list(v.validate_test(input_value))

    else:
        assert list(v.validate_test(input_value)) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ([1, 2, 3], [1, 2, 3]),
        ([1, 2, '3'], [1, 2, '3']),
        ({'1': 2, '3': 4}, ['1', '3']),
        ('123', ['1', '2', '3']),
        (5, Err('Input should be iterable [kind=iterable_type, input_value=5, input_type=int]')),
        ([1, 'wrong'], [1, 'wrong']),
    ],
    ids=repr,
)
def test_generator_json_any(py_and_json: PyAndJson, input_value, expected):
    v = py_and_json({'type': 'generator'})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            list(v.validate_test(input_value))

    else:
        assert list(v.validate_test(input_value)) == expected
