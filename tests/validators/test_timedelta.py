import re
from datetime import timedelta
from decimal import Decimal

import pytest

from pydantic_core import SchemaError, SchemaValidator, ValidationError

from ..conftest import Err


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (
            timedelta(days=3, weeks=2, hours=1, minutes=2, seconds=3, milliseconds=500),
            timedelta(days=3, weeks=2, hours=1, minutes=2, seconds=3, milliseconds=500),
        ),
        ('P0Y0M3D2WT1H2M3.5S', timedelta(days=3, weeks=2, hours=1, minutes=2, seconds=3, milliseconds=500)),
        (b'P0Y0M3D2WT1H2M3.5S', timedelta(days=3, weeks=2, hours=1, minutes=2, seconds=3, milliseconds=500)),
        ((-1,), Err('Value must be a valid timedelta [kind=timedelta_type')),
        (3601, timedelta(hours=1, seconds=1)),
        (Decimal('3601.123456'), timedelta(hours=1, seconds=1, microseconds=123456)),
        (Decimal('3601.1234562'), timedelta(hours=1, seconds=1, microseconds=123456)),
        (Decimal('3601.1234568'), timedelta(hours=1, seconds=1, microseconds=123457)),
        (-3601, timedelta(hours=-2, seconds=3599)),
        (Decimal('-3601.222222'), timedelta(hours=-2, seconds=3598, microseconds=777778)),
        (Decimal('-3601.2222222'), timedelta(hours=-2, seconds=3598, microseconds=777778)),
        (Decimal('-3601.2222227'), timedelta(hours=-2, seconds=3598, microseconds=777777)),
        (2**32 + 1, timedelta(seconds=2**32 + 1)),
    ],
    ids=repr,
)
def test_timedelta(input_value, expected):
    v = SchemaValidator({'type': 'timedelta'})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        output = v.validate_python(input_value)
        assert output == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ('"P0Y0M3D2WT1H2M3.5S"', timedelta(days=3, weeks=2, hours=1, minutes=2, seconds=3, milliseconds=500)),
        ('"errordata"', Err('Value must be a valid timedelta, invalid digit in duration [kind=timedelta_parsing')),
        ('true', Err('Value must be a valid timedelta [kind=timedelta_type')),
        ('3601', timedelta(hours=1, seconds=1)),
        ('3601.123456', timedelta(hours=1, seconds=1, microseconds=123456)),
        ('-3601', timedelta(hours=-2, seconds=3599)),
        ('-3601.222222', timedelta(hours=-2, seconds=3598, microseconds=777778)),
        ('-3601.2222222', timedelta(hours=-2, seconds=3598, microseconds=777778)),
    ],
    ids=repr,
)
def test_timedelta_json(input_value, expected):
    v = SchemaValidator({'type': 'timedelta'})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_json(input_value)
    else:
        output = v.validate_json(input_value)
        assert output == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (
            timedelta(days=3, weeks=2, hours=1, minutes=2, seconds=3, milliseconds=500),
            timedelta(days=3, weeks=2, hours=1, minutes=2, seconds=3, milliseconds=500),
        ),
        ('P0Y0M3D2WT1H2M3.5S', Err('Value must be a valid timedelta [kind=timedelta_type')),
        (b'P0Y0M3D2WT1H2M3.5S', Err('Value must be a valid timedelta [kind=timedelta_type')),
    ],
)
def test_timedelta_strict(input_value, expected):
    v = SchemaValidator({'type': 'timedelta', 'strict': True})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        output = v.validate_python(input_value)
        assert output == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ('"P0Y0M3D2WT1H2M3.5S"', timedelta(days=3, weeks=2, hours=1, minutes=2, seconds=3, milliseconds=500)),
        ('"12345"', Err('Value must be a valid timedelta')),
        ('true', Err('Value must be a valid timedelta [kind=timedelta_type')),
    ],
)
def test_timedelta_strict_json(input_value, expected):
    v = SchemaValidator({'type': 'timedelta', 'strict': True})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_json(input_value)
    else:
        output = v.validate_json(input_value)
        assert output == expected


@pytest.mark.parametrize(
    'kwargs,input_value,expected',
    [
        ({}, 'P0Y0M3D2WT1H2M3S', timedelta(days=3, weeks=2, hours=1, minutes=2, seconds=3)),
        ({'le': timedelta(days=3)}, 'P2DT1H', timedelta(days=2, hours=1)),
        ({'le': timedelta(days=3)}, 'P3DT0H', timedelta(days=3)),
        ({'le': timedelta(days=3)}, 'P3DT1H', Err('Value must be less than or equal to P3D')),
        ({'lt': timedelta(days=3)}, 'P2DT1H', timedelta(days=2, hours=1)),
        ({'lt': timedelta(days=3)}, 'P3DT1H', Err('Value must be less than P3D')),
        ({'ge': timedelta(days=3)}, 'P3DT1H', timedelta(days=3, hours=1)),
        ({'ge': timedelta(days=3)}, 'P3D', timedelta(days=3)),
        ({'ge': timedelta(days=3)}, 'P2DT1H', Err('Value must be greater than or equal to P3D')),
        ({'gt': timedelta(days=3)}, 'P3DT1H', timedelta(days=3, hours=1)),
        ({'gt': 'P3D'}, 'P2DT1H', Err('Value must be greater than P3D')),
    ],
)
def test_timedelta_kwargs(kwargs, input_value, expected):
    v = SchemaValidator({'type': 'timedelta', **kwargs})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        output = v.validate_python(input_value)
        assert output == expected


def test_timedelta_kwargs_strict():
    v = SchemaValidator({'type': 'timedelta', 'strict': True, 'le': timedelta(days=3)})
    output = v.validate_python(timedelta(days=2, hours=1))
    assert output == timedelta(days=2, hours=1)


def test_invalid_constraint():
    with pytest.raises(SchemaError, match='Invalid "gt" constraint for timedelta:  Value must be a valid timedelta'):
        SchemaValidator({'type': 'timedelta', 'gt': 'foobar'})

    with pytest.raises(SchemaError, match='Invalid "le" constraint for timedelta:  Value must be a valid timedelta'):
        SchemaValidator({'type': 'timedelta', 'le': 'foobar'})

    with pytest.raises(SchemaError, match='Invalid "lt" constraint for timedelta:  Value must be a valid timedelta'):
        SchemaValidator({'type': 'timedelta', 'lt': 'foobar'})

    with pytest.raises(SchemaError, match='Invalid "ge" constraint for timedelta:  Value must be a valid timedelta'):
        SchemaValidator({'type': 'timedelta', 'ge': 'foobar'})


def test_dict_py():
    v = SchemaValidator({'type': 'dict', 'keys_schema': 'timedelta', 'values_schema': 'int'})
    assert v.validate_python({timedelta(days=2, hours=1): 2, timedelta(days=2, hours=2): 4}) == {
        timedelta(days=2, hours=1): 2,
        timedelta(days=2, hours=2): 4,
    }


def test_dict_key(py_or_json):
    v = py_or_json({'type': 'dict', 'keys_schema': 'timedelta', 'values_schema': 'int'})
    assert v.validate_test({'P2DT1H': 2, 'P2DT2H': 4}) == {timedelta(days=2, hours=1): 2, timedelta(days=2, hours=2): 4}

    with pytest.raises(
        ValidationError,
        match=re.escape('Value must be a valid timedelta, invalid digit in duration [kind=timedelta_parsing'),
    ):
        v.validate_test({'errordata': 2})


def test_dict_value(py_or_json):
    v = py_or_json({'type': 'dict', 'keys_schema': 'int', 'values_schema': 'timedelta'})
    assert v.validate_test({2: 'P2DT1H', 4: 'P2DT2H'}) == {2: timedelta(days=2, hours=1), 4: timedelta(days=2, hours=2)}

    with pytest.raises(
        ValidationError,
        match=re.escape('Value must be a valid timedelta, invalid digit in duration [kind=timedelta_parsing'),
    ):
        v.validate_test({4: 'errordata'})


def test_union():
    v = SchemaValidator({'type': 'union', 'choices': ['str', 'timedelta']})
    assert v.validate_python('P2DT1H') == 'P2DT1H'
    assert v.validate_python(timedelta(days=2, hours=1)) == timedelta(days=2, hours=1)

    v = SchemaValidator({'type': 'union', 'choices': ['timedelta', 'str']})
    assert v.validate_python('P2DT1H') == 'P2DT1H'
    assert v.validate_python(timedelta(days=2, hours=1)) == timedelta(days=2, hours=1)
