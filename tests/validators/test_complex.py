import math
import re

import pytest

from pydantic_core import SchemaValidator, ValidationError

from ..conftest import Err

EXPECTED_PARSE_ERROR_MESSAGE = 'Input should be a valid complex string following the rule at https://docs.python.org/3/library/functions.html#complex'
EXPECTED_TYPE_ERROR_MESSAGE = 'Input should be a valid complex number'


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (complex(2, 4), complex(2, 4)),
        ('2', complex(2, 0)),
        ('2j', complex(0, 2)),
        ('+1.23e-4-5.67e+8J', complex(1.23e-4, -5.67e8)),
        ('1.5-j', complex(1.5, -1)),
        ('-j', complex(0, -1)),
        ('j', complex(0, 1)),
        (3, complex(3, 0)),
        (2.0, complex(2, 0)),
        ('1e-700j', complex(0, 0)),
        ('', Err(EXPECTED_PARSE_ERROR_MESSAGE)),
        ({'real': 2, 'imag': 4}, Err(EXPECTED_TYPE_ERROR_MESSAGE)),
        ({'real': 'test', 'imag': 1}, Err(EXPECTED_TYPE_ERROR_MESSAGE)),
        ({'real': True, 'imag': 1}, Err(EXPECTED_TYPE_ERROR_MESSAGE)),
        ('foobar', Err(EXPECTED_PARSE_ERROR_MESSAGE)),
        ([], Err(EXPECTED_TYPE_ERROR_MESSAGE)),
        ([('x', 'y')], Err(EXPECTED_TYPE_ERROR_MESSAGE)),
        ((), Err(EXPECTED_TYPE_ERROR_MESSAGE)),
        ((('x', 'y'),), Err(EXPECTED_TYPE_ERROR_MESSAGE)),
        (
            (type('Foobar', (), {'x': 1})()),
            Err(EXPECTED_TYPE_ERROR_MESSAGE),
        ),
    ],
    ids=repr,
)
def test_complex_cases(input_value, expected):
    v = SchemaValidator({'type': 'complex'})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        assert v.validate_python(input_value) == expected


def test_nan_inf_complex():
    v = SchemaValidator({'type': 'complex'})
    c = v.validate_python('NaN+Infinityj')
    # c != complex(float('nan'), float('inf')) as nan != nan,
    # so we need to examine the values individually
    assert math.isnan(c.real)
    assert math.isinf(c.imag)


def test_overflow_complex():
    # Python simply converts too large float values to inf, so these strings
    # are still valid, even if the numbers are out of range
    v = SchemaValidator({'type': 'complex'})

    c = v.validate_python('5e600j')
    assert math.isinf(c.imag)

    c = v.validate_python('-5e600j')
    assert math.isinf(c.imag)


def test_json_complex():
    v = SchemaValidator({'type': 'complex'})
    assert v.validate_json('"-1.23e+4+5.67e-8J"') == complex(-1.23e4, 5.67e-8)
    assert v.validate_json('1') == complex(1, 0)
    assert v.validate_json('1.0') == complex(1, 0)
    # "1" is a valid complex string
    assert v.validate_json('"1"') == complex(1, 0)

    with pytest.raises(ValidationError) as exc_info:
        v.validate_json('{"real": 2, "imag": 4}')
    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'complex_parsing',
            'loc': (),
            'msg': EXPECTED_PARSE_ERROR_MESSAGE,
            'input': {'real': 2, 'imag': 4},
        }
    ]


def test_string_complex():
    v = SchemaValidator({'type': 'complex'})
    assert v.validate_strings('+1.23e-4-5.67e+8J') == complex(1.23e-4, -5.67e8)
    with pytest.raises(ValidationError, match=re.escape(EXPECTED_PARSE_ERROR_MESSAGE)):
        v.validate_strings("{'real': 1, 'imag': 0}")
