import re
from datetime import datetime, timedelta, timezone
from decimal import Decimal

import pytest
import pytz

from pydantic_core import SchemaValidator, ValidationError

from ..conftest import Err


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (datetime(2022, 6, 8, 12, 13, 14), datetime(2022, 6, 8, 12, 13, 14)),
        ('2022-06-08T12:13:14', datetime(2022, 6, 8, 12, 13, 14)),
        (b'2022-06-08T12:13:14', datetime(2022, 6, 8, 12, 13, 14)),
        (b'2022-06-08T12:13:14Z', datetime(2022, 6, 8, 12, 13, 14, tzinfo=timezone.utc)),
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


def test_keep_tz():
    tz = pytz.timezone('Europe/London')
    dt = tz.localize(datetime(2022, 6, 14, 12, 13, 14))
    v = SchemaValidator({'type': 'datetime'})

    output = v.validate_python(dt)
    assert output == dt

    # dst object is unaffected by validation
    assert output.tzinfo.dst(datetime(2022, 6, 1)) == timedelta(seconds=3600)
    assert output.tzinfo.dst(datetime(2022, 1, 1)) == timedelta(seconds=0)


def test_keep_tz_bound():
    tz = pytz.timezone('Europe/London')
    dt = tz.localize(datetime(2022, 6, 14, 12, 13, 14))
    v = SchemaValidator({'type': 'datetime', 'gt': datetime(2022, 1, 1)})

    output = v.validate_python(dt)
    assert output == dt

    # dst object is unaffected by validation
    assert output.tzinfo.dst(datetime(2022, 6, 1)) == timedelta(hours=1)
    assert output.tzinfo.dst(datetime(2022, 1, 1)) == timedelta(0)

    with pytest.raises(ValidationError, match=r'Value must be greater than 2022-01-01T00:00:00 \[kind=greater_than'):
        v.validate_python(tz.localize(datetime(2021, 6, 14)))


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ('2022-06-08T12:13:14', datetime(2022, 6, 8, 12, 13, 14)),
        ('2022-06-08T12:13:14Z', datetime(2022, 6, 8, 12, 13, 14, tzinfo=timezone.utc)),
        (
            '2022-06-08T12:13:14+12:15',
            datetime(2022, 6, 8, 12, 13, 14, tzinfo=timezone(timedelta(hours=12, minutes=15))),
        ),
        (
            '2022-06-08T12:13:14+23:59',
            datetime(2022, 6, 8, 12, 13, 14, tzinfo=timezone(timedelta(hours=23, minutes=59))),
        ),
        (1655205632, datetime(2022, 6, 14, 11, 20, 32)),
        (1655205632.331557, datetime(2022, 6, 14, 11, 20, 32, microsecond=331557)),
        (
            '2022-06-08T12:13:14+24:00',
            Err('Value must be a valid datetime, timezone offset must be less than 24 hours [kind=date_time_parsing,'),
        ),
    ],
)
def test_datetime_json(py_or_json, input_value, expected):
    v = py_or_json({'type': 'datetime'})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_test(input_value)
    else:
        output = v.validate_test(input_value)
        assert output == expected


def test_custom_timezone_repr():
    output = SchemaValidator({'type': 'datetime'}).validate_python('2022-06-08T12:13:14-12:15')
    assert output == datetime(2022, 6, 8, 12, 13, 14, tzinfo=timezone(timedelta(hours=-12, minutes=-15)))
    assert output.tzinfo.utcoffset(output) == timedelta(hours=-12, minutes=-15)
    assert output.tzinfo.dst(output) is None
    assert output.tzinfo.tzname(output) == '-12:15'
    assert str(output.tzinfo) == '-12:15'
    assert repr(output.tzinfo) == 'TzInfo(-12:15)'


def test_custom_timezone_utc_repr():
    output = SchemaValidator({'type': 'datetime'}).validate_python('2022-06-08T12:13:14Z')
    assert output == datetime(2022, 6, 8, 12, 13, 14, tzinfo=timezone(timedelta(0)))
    assert output.tzinfo.utcoffset(output) == timedelta(0)
    assert output.tzinfo.dst(output) is None
    assert output.tzinfo.tzname(output) == 'UTC'
    assert str(output.tzinfo) == 'UTC'
    assert repr(output.tzinfo) == 'TzInfo(UTC)'
