import re

import pytest

from pydantic_core import SchemaValidator, ValidationError

from ..conftest import Err


def test_bytes_constrained():
    v = SchemaValidator({'type': 'bytes', 'max_length': 5})
    assert v.validate_python(b'test') == b'test'

    with pytest.raises(ValidationError, match='Bytes must have at most 5 characters'):
        v.validate_python(b'this is too long')


@pytest.mark.parametrize(
    'opts,input,expected', [({}, 'foo', b'foo'), ({'strict': True}, 'foo', Err("Value must be a valid bytes"))]
)
def test_constrained_bytes(py_or_json, opts, input, expected):
    v = py_or_json({'type': 'bytes', **opts})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_test(input)
    else:
        assert v.validate_test(input) == expected
