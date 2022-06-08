import re
from datetime import date
from decimal import Decimal

import pytest

from pydantic_core import SchemaError, SchemaValidator, ValidationError

from ..conftest import Err


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (date(2022, 6, 8), date(2022, 6, 8)),
        ('2022-06-08', date(2022, 6, 8)),
        (b'2022-06-08', date(2022, 6, 8)),
        ((1,), Err('Value must be a valid date [kind=date_type')),
        (Decimal('1654646400'), date(2022, 6, 8)),
        # (253_402_300_800_000, Err('format YYYY-MM-DD, dates after 9999 are not supported as unix timestamps')),
        (253_402_300_800_000, Err('Value must be a valid date')),
        # (-20_000_000_000, Err('format YYYY-MM-DD, dates before 1600 are not supported as unix timestamps')),
        (-20_000_000_000, Err('Value must be a valid date')),
    ],
)
def test_date(input_value, expected):
    v = SchemaValidator({'type': 'date'})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        output = v.validate_python(input_value)
        assert output == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ('2022-06-08', date(2022, 6, 8)),
        ('1453-01-28', date(1453, 1, 28)),
        (1654646400, date(2022, 6, 8)),
        (1654646400.0, date(2022, 6, 8)),
        (
            1654646401,
            Err(
                'Datetimes provided to dates must have zero time - e.g. be exact dates [kind=date_from_datetime_inexact'
            ),
        ),
        ('wrong', Err('Value must be a valid date or datetime, day value is outside expected range [kind=date_from_datetime_parsing')),
        ('2000-02-29', date(2000, 2, 29)),
        ('2001-02-29', Err('Value must be a valid date in the format YYYY-MM-DD, day value is outside expected range')),
        ([1], Err('Value must be a valid date [kind=date_type')),
    ],
)
def test_date_json(py_or_json, input_value, expected):
    v = py_or_json({'type': 'date'})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_test(input_value)
    else:
        output = v.validate_test(input_value)
        assert output == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (date(2022, 6, 8), date(2022, 6, 8)),
        ('2022-06-08', Err('Value must be a valid date [kind=date_type')),
        (b'2022-06-08', Err('Value must be a valid date [kind=date_type')),
        (1654646400, Err('Value must be a valid date [kind=date_type')),
    ],
)
def test_date_strict(input_value, expected):
    v = SchemaValidator({'type': 'date', 'strict': True})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        output = v.validate_python(input_value)
        assert output == expected


@pytest.mark.parametrize(
    'kwargs,input_value,expected',
    [
        ({}, '2000-01-01', date(2000, 1, 1)),
        ({'le': date(2000, 1, 1)}, '2000-01-01', date(2000, 1, 1)),
        (
            {'le': date(2000, 1, 1)},
            '2000-01-02',
            Err('Value must be less than or equal to 2000-01-01 [kind=less_than_equal,'),
        ),
        ({'lt': date(2000, 1, 1)}, '1999-12-31', date(1999, 12, 31)),
        ({'lt': date(2000, 1, 1)}, '2000-01-01', Err('Value must be less than 2000-01-01 [kind=less_than,')),
        ({'ge': date(2000, 1, 1)}, '2000-01-01', date(2000, 1, 1)),
        (
            {'ge': date(2000, 1, 1)},
            '1999-12-31',
            Err('Value must be greater than or equal to 2000-01-01 [kind=greater_than_equal,'),
        ),
        ({'gt': date(2000, 1, 1)}, '2000-01-02', date(2000, 1, 2)),
        ({'gt': date(2000, 1, 1)}, '2000-01-01', Err('Value must be greater than 2000-01-01 [kind=greater_than,')),
    ],
)
def test_date_kwargs(kwargs, input_value, expected):
    v = SchemaValidator({'type': 'date', **kwargs})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        output = v.validate_python(input_value)
        assert output == expected


def test_invalid_constraint():
    with pytest.raises(SchemaError, match="'str' object cannot be converted to 'PyDate'"):
        SchemaValidator({'type': 'date', 'gt': '2000-01-01'})
