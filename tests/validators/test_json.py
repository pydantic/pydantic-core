import re

import pytest

from pydantic_core import SchemaValidator, ValidationError, core_schema

from ..conftest import Err


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ('{"a": 1}', {'a': 1}),
        ('"a"', 'a'),
        ('1', 1),
        ('[1, 2, 3, "4"]', [1, 2, 3, '4']),
        (
            '{1: 2}',
            Err(
                'Invalid JSON: key must be a string at line 1 column 2 [kind=invalid_json,',
                [
                    {
                        'kind': 'invalid_json',
                        'loc': [],
                        'message': 'Invalid JSON: key must be a string at line 1 column 2',
                        'input_value': '{1: 2}',
                        'context': {'error': 'key must be a string at line 1 column 2'},
                    }
                ],
            ),
        ),
        (44, Err('JSON input should be str, bytes or bytearray [kind=json_type, input_value=44, input_type=int')),
    ],
)
def test_any(input_value, expected):
    v = SchemaValidator(core_schema.json_schema())
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.validate_python(input_value)

        if expected.errors is not None:
            # debug(exc_info.value.errors())
            assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_python(input_value) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ('[1]', [1]),
        ('[1, 2, 3, "4"]', [1, 2, 3, 4]),
        ('44', Err('Input should be a valid list/array [kind=list_type, input_value=44, input_type=int')),
        ('"x"', Err("Input should be a valid list/array [kind=list_type, input_value='x', input_type=str")),
        (
            '[1, 2, 3, "err"]',
            Err(
                'Input should be a valid integer, unable to parse string as an integer [kind=int_parsing,',
                [
                    {
                        'kind': 'int_parsing',
                        'loc': [3],
                        'message': 'Input should be a valid integer, unable to parse string as an integer',
                        'input_value': 'err',
                    }
                ],
            ),
        ),
    ],
)
def test_list_int(input_value, expected):
    v = SchemaValidator(core_schema.json_schema(core_schema.list_schema(core_schema.int_schema())))
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.validate_python(input_value)

        if expected.errors is not None:
            debug(exc_info.value.errors())
            assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_python(input_value) == expected
