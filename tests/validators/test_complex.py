import math
import re

import pytest

from pydantic_core import SchemaValidator, ValidationError

from ..conftest import Err, PyAndJson

EXPECTED_TYPE_ERROR_MESSAGE = (
    "Input should be a valid dictionary with exactly two keys, 'real' and 'imag', with float values"
)


def test_dict(py_and_json: PyAndJson):
    v = py_and_json({'type': 'complex'})
    assert v.validate_test({'real': 2, 'imag': 4}) == complex(2, 4)
    with pytest.raises(ValidationError, match=re.escape('[type=complex_type, input_value=[], input_type=list]')):
        v.validate_test([])


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (complex(2, 4), complex(2, 4)),
        ({'real': 2, 'imag': 4}, complex(2, 4)),
        ({'real': 2}, complex(2, 0)),
        ({'imag': 2}, complex(0, 2)),
        ({}, complex(0, 0)),
        ({'real': 'test', 'imag': 1}, Err(EXPECTED_TYPE_ERROR_MESSAGE)),
        ('foobar', Err(EXPECTED_TYPE_ERROR_MESSAGE)),
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
    c = v.validate_python({'real': float('nan'), 'imag': float('inf')})
    # c != complex(float('nan'), float('inf')) as nan != nan,
    # so we need to examine the values individually
    assert math.isnan(c.real)
    assert math.isinf(c.imag)


def test_json_complex():
    v = SchemaValidator({'type': 'complex'})
    assert v.validate_json('{"real": 2, "imag": 4}') == complex(2, 4)
    with pytest.raises(ValidationError) as exc_info:
        v.validate_json('1')
    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'complex_type',
            'loc': (),
            'msg': EXPECTED_TYPE_ERROR_MESSAGE,
            'input': 1,
        }
    ]


def test_string_complex():
    v = SchemaValidator({'type': 'complex'})
    with pytest.raises(ValidationError, match=re.escape(EXPECTED_TYPE_ERROR_MESSAGE)):
        v.validate_strings("{'real': float('nan'), 'imag': 0}")
