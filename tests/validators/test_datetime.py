import re
from datetime import datetime
from decimal import Decimal

import pytest

from pydantic_core import SchemaValidator, ValidationError

from ..conftest import Err


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (datetime(2022, 6, 8, 12, 13, 14), datetime(2022, 6, 8, 12, 13, 14)),
        ('2022-06-08T12:13:14', datetime(2022, 6, 8, 12, 13, 14)),
        (b'2022-06-08T12:13:14', datetime(2022, 6, 8, 12, 13, 14)),
        ((1,), Err('Value must be a valid datetime [kind=date_time_type')),
        (Decimal('1654646400'), datetime(2022, 6, 8)),
        (253_402_300_800_000, Err('must be a valid datetime, dates after 9999 are not supported as unix timestamps')),
        (-20_000_000_000, Err('must be a valid datetime, dates before 1600 are not supported as unix timestamps')),
    ],
)
def test_datetime(input_value, expected):
    v = SchemaValidator({'type': 'datetime'})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        output = v.validate_python(input_value)
        assert output == expected
