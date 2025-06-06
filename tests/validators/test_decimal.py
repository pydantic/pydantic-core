from __future__ import annotations

import json
import math
import re
from decimal import Decimal
from typing import Any

import pytest
from dirty_equals import FunctionCheck, IsStr

from pydantic_core import SchemaError, SchemaValidator, ValidationError
from pydantic_core import core_schema as cs

from ..conftest import Err, PyAndJson, plain_repr


class DecimalSubclass(Decimal):
    pass


# Note: there's another constraint validation (allow_inf_nan=True cannot be used with max_digits or decimal_places).
# but it is tested in Pydantic:
@pytest.mark.parametrize(
    'constraint',
    ['multiple_of', 'le', 'lt', 'ge', 'gt'],
)
def test_constraints_schema_validation_error(constraint: str) -> None:
    with pytest.raises(SchemaError, match=f"'{constraint}' must be coercible to a Decimal instance"):
        SchemaValidator(cs.decimal_schema(**{constraint: 'bad_value'}))


def test_constraints_schema_validation() -> None:
    val = SchemaValidator(cs.decimal_schema(gt='1'))
    with pytest.raises(ValidationError):
        val.validate_python('0')


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (0, Decimal(0)),
        (1, Decimal(1)),
        (42, Decimal(42)),
        ('42', Decimal(42)),
        ('42.123', Decimal('42.123')),
        (42.0, Decimal(42)),
        (42.5, Decimal('42.5')),
        (1e10, Decimal('1E10')),
        (Decimal('42.0'), Decimal(42)),
        (Decimal('42.5'), Decimal('42.5')),
        (Decimal('1e10'), Decimal('1E10')),
        (
            Decimal('123456789123456789123456789.123456789123456789123456789'),
            Decimal('123456789123456789123456789.123456789123456789123456789'),
        ),
        (DecimalSubclass('42.0'), Decimal(42)),
        (DecimalSubclass('42.5'), Decimal('42.5')),
        (DecimalSubclass('1e10'), Decimal('1E10')),
        (
            True,
            Err(
                'Decimal input should be an integer, float, string or Decimal object [type=decimal_type, input_value=True, input_type=bool]'
            ),
        ),
        (
            False,
            Err(
                'Decimal input should be an integer, float, string or Decimal object [type=decimal_type, input_value=False, input_type=bool]'
            ),
        ),
        ('wrong', Err('Input should be a valid decimal [type=decimal_parsing')),
        (
            [1, 2],
            Err(
                'Decimal input should be an integer, float, string or Decimal object [type=decimal_type, input_value=[1, 2], input_type=list]'
            ),
        ),
    ],
)
def test_decimal(py_and_json: PyAndJson, input_value, expected):
    v = py_and_json({'type': 'decimal'})
    # Decimal types are not JSON serializable
    if v.validator_type == 'json' and isinstance(input_value, Decimal):
        input_value = str(input_value)
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_test(input_value)
    else:
        output = v.validate_test(input_value)
        assert output == expected
        assert isinstance(output, Decimal)


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (Decimal(0), Decimal(0)),
        (Decimal(1), Decimal(1)),
        (Decimal(42), Decimal(42)),
        (Decimal('42.0'), Decimal('42.0')),
        (Decimal('42.5'), Decimal('42.5')),
        (42.0, Err('Input should be an instance of Decimal [type=is_instance_of, input_value=42.0, input_type=float]')),
        ('42', Err("Input should be an instance of Decimal [type=is_instance_of, input_value='42', input_type=str]")),
        (42, Err('Input should be an instance of Decimal [type=is_instance_of, input_value=42, input_type=int]')),
        (True, Err('Input should be an instance of Decimal [type=is_instance_of, input_value=True, input_type=bool]')),
    ],
    ids=repr,
)
def test_decimal_strict_py(input_value, expected):
    v = SchemaValidator(cs.decimal_schema(strict=True))
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        output = v.validate_python(input_value)
        assert output == expected
        assert isinstance(output, Decimal)


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (0, Decimal(0)),
        (1, Decimal(1)),
        (42, Decimal(42)),
        ('42.0', Decimal('42.0')),
        ('42.5', Decimal('42.5')),
        (42.0, Decimal('42.0')),
        ('42', Decimal('42')),
        (
            True,
            Err(
                'Decimal input should be an integer, float, string or Decimal object [type=decimal_type, input_value=True, input_type=bool]'
            ),
        ),
    ],
    ids=repr,
)
def test_decimal_strict_json(input_value, expected):
    v = SchemaValidator(cs.decimal_schema(strict=True))
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_json(json.dumps(input_value))
    else:
        output = v.validate_json(json.dumps(input_value))
        assert output == expected
        assert isinstance(output, Decimal)


@pytest.mark.parametrize(
    'kwargs,input_value,expected',
    [
        ({}, 0, Decimal(0)),
        ({}, '123.456', Decimal('123.456')),
        ({'ge': 0}, 0, Decimal(0)),
        (
            {'ge': 0},
            -0.1,
            Err(
                'Input should be greater than or equal to 0 '
                '[type=greater_than_equal, input_value=-0.1, input_type=float]'
            ),
        ),
        ({'gt': 0}, 0.1, Decimal('0.1')),
        ({'gt': 0}, 0, Err('Input should be greater than 0 [type=greater_than, input_value=0, input_type=int]')),
        ({'le': 0}, 0, Decimal(0)),
        ({'le': 0}, -1, Decimal(-1)),
        ({'le': 0}, 0.1, Err('Input should be less than or equal to 0')),
        ({'lt': 0, 'allow_inf_nan': True}, float('nan'), Err('Input should be less than 0')),
        ({'gt': 0, 'allow_inf_nan': True}, float('inf'), Decimal('inf')),
        ({'allow_inf_nan': True}, float('-inf'), Decimal('-inf')),
        ({'allow_inf_nan': True}, float('nan'), FunctionCheck(math.isnan)),
        ({'lt': 0}, 0, Err('Input should be less than 0')),
        ({'lt': 0.123456}, 1, Err('Input should be less than 0.123456')),
    ],
)
def test_decimal_kwargs(py_and_json: PyAndJson, kwargs: dict[str, Any], input_value, expected):
    v = py_and_json({'type': 'decimal', **kwargs})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_test(input_value)
    else:
        output = v.validate_test(input_value)
        assert output == expected
        assert isinstance(output, Decimal)


@pytest.mark.parametrize(
    'multiple_of,input_value,error',
    [
        # Test cases for multiples of 0.5
        *[(0.5, round(i * 0.5, 1), None) for i in range(-4, 5)],
        (0.5, 0.49, Err('Input should be a multiple of 0.5')),
        (0.5, 0.6, Err('Input should be a multiple of 0.5')),
        (0.5, -0.75, Err('Input should be a multiple of 0.5')),
        (0.5, 0.501, Err('Input should be a multiple of 0.5')),
        (0.5, 1_000_000.5, None),
        (0.5, 1_000_000.49, Err('Input should be a multiple of 0.5')),
        (0.5, int(5e10), None),
        # Test cases for multiples of 0.1
        *[(0.1, round(i * 0.1, 1), None) for i in range(-10, 11)],
        (0.1, 0, None),
        (0.1, 0.5001, Err('Input should be a multiple of 0.1')),
        (0.1, 0.05, Err('Input should be a multiple of 0.1')),
        (0.1, -0.15, Err('Input should be a multiple of 0.1')),
        (0.1, 1_000_000.1, None),
        (0.1, 1_000_000.05, Err('Input should be a multiple of 0.1')),
        (0.1, 1, None),
        (0.1, int(5e10), None),
        # Test cases for multiples of 2.0
        *[(2.0, i * 2.0, None) for i in range(-5, 6)],
        (2.0, -2.1, Err('Input should be a multiple of 2')),
        (2.0, -3.0, Err('Input should be a multiple of 2')),
        (2.0, 1_000_002.0, None),
        (2.0, 1_000_001.0, Err('Input should be a multiple of 2')),
        (2.0, int(5e10), None),
        # Test cases for multiples of 0.01
        *[(0.01, round(i * 0.01, 2), None) for i in range(-10, 11)],
        (0.01, 0.005, Err('Input should be a multiple of 0.01')),
        (0.01, -0.015, Err('Input should be a multiple of 0.01')),
        (0.01, 1_000_000.01, None),
        (0.01, 1_000_000.005, Err('Input should be a multiple of 0.01')),
        (0.01, int(5e10), None),
        # Test cases for values very close to zero
        (0.1, 0.00001, Err('Input should be a multiple of 0.1')),
        (0.1, -0.00001, Err('Input should be a multiple of 0.1')),
        (0.01, 0.00001, Err('Input should be a multiple of 0.01')),
        (0.01, -0.00001, Err('Input should be a multiple of 0.01')),
    ],
    ids=repr,
)
def test_decimal_multiple_of(py_and_json: PyAndJson, multiple_of: float, input_value: float, error: Err | None):
    v = py_and_json({'type': 'decimal', 'multiple_of': Decimal(str(multiple_of))})
    if error:
        with pytest.raises(ValidationError, match=re.escape(error.message)):
            v.validate_test(input_value)
    else:
        output = v.validate_test(input_value)
        assert output == Decimal(str(input_value))
        assert isinstance(output, Decimal)


def test_union_decimal_py():
    v = SchemaValidator(cs.union_schema(choices=[cs.decimal_schema(strict=True), cs.decimal_schema(multiple_of=7)]))
    assert v.validate_python('14') == 14
    assert v.validate_python(Decimal(5)) == 5
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python('5')
    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'is_instance_of',
            'loc': ('decimal',),
            'msg': 'Input should be an instance of Decimal',
            'input': '5',
            'ctx': {'class': 'Decimal'},
        },
        {
            'type': 'multiple_of',
            'loc': ('decimal',),
            'msg': 'Input should be a multiple of 7',
            'input': '5',
            'ctx': {'multiple_of': 7},
        },
    ]


def test_union_decimal_json():
    v = SchemaValidator(cs.union_schema(choices=[cs.decimal_schema(strict=True), cs.decimal_schema(multiple_of=7)]))
    assert v.validate_json(json.dumps('14')) == 14
    assert v.validate_json(json.dumps('5')) == 5


def test_union_decimal_simple(py_and_json: PyAndJson):
    v = py_and_json({'type': 'union', 'choices': [{'type': 'decimal'}, {'type': 'list'}]})
    assert v.validate_test('5') == 5
    with pytest.raises(ValidationError) as exc_info:
        v.validate_test('xxx')

    assert exc_info.value.errors(include_url=False) == [
        {'type': 'decimal_parsing', 'loc': ('decimal',), 'msg': 'Input should be a valid decimal', 'input': 'xxx'},
        {
            'type': 'list_type',
            'loc': ('list[any]',),
            'msg': IsStr(regex='Input should be a valid (list|array)'),
            'input': 'xxx',
        },
    ]


def test_decimal_repr():
    v = SchemaValidator(cs.decimal_schema())
    assert plain_repr(v).startswith(
        'SchemaValidator(title="decimal",validator=Decimal(DecimalValidator{strict:false,allow_inf_nan:false'
    )
    v = SchemaValidator(cs.decimal_schema(strict=True))
    assert plain_repr(v).startswith(
        'SchemaValidator(title="decimal",validator=Decimal(DecimalValidator{strict:true,allow_inf_nan:false'
    )
    v = SchemaValidator(cs.decimal_schema(multiple_of=7))
    assert plain_repr(v).startswith('SchemaValidator(title="decimal",validator=Decimal(')


@pytest.mark.parametrize('input_value,expected', [(Decimal('1.23'), Decimal('1.23')), (Decimal('1'), Decimal('1.0'))])
def test_decimal_not_json(input_value, expected):
    v = SchemaValidator(cs.decimal_schema())
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        output = v.validate_python(input_value)
        assert output == expected
        assert isinstance(output, Decimal)


def test_decimal_nan(py_and_json: PyAndJson):
    v = py_and_json({'type': 'decimal', 'allow_inf_nan': True})
    assert v.validate_test('inf') == Decimal('inf')
    assert v.validate_test('-inf') == Decimal('-inf')
    r = v.validate_test('nan')
    assert math.isnan(r)


def test_decimal_key(py_and_json: PyAndJson):
    v = py_and_json({'type': 'dict', 'keys_schema': {'type': 'decimal'}, 'values_schema': {'type': 'int'}})
    assert v.validate_test({'1': 1, '2': 2}) == {Decimal('1'): 1, Decimal('2'): 2}
    assert v.validate_test({'1.5': 1, '2.4': 2}) == {Decimal('1.5'): 1, Decimal('2.4'): 2}
    if v.validator_type == 'python':
        with pytest.raises(ValidationError, match='Input should be an instance of Decimal'):
            v.validate_test({'1.5': 1, '2.5': 2}, strict=True)
    else:
        assert v.validate_test({'1.5': 1, '2.4': 2}, strict=True) == {Decimal('1.5'): 1, Decimal('2.4'): 2}


@pytest.mark.parametrize(
    'input_value,allow_inf_nan,expected',
    [
        ('NaN', True, FunctionCheck(math.isnan)),
        ('NaN', False, Err("Input should be a finite number [type=finite_number, input_value='NaN', input_type=str]")),
        ('+inf', True, FunctionCheck(lambda x: math.isinf(x) and x > 0)),
        (
            '+inf',
            False,
            Err("Input should be a finite number [type=finite_number, input_value='+inf', input_type=str]"),
        ),
        ('+infinity', True, FunctionCheck(lambda x: math.isinf(x) and x > 0)),
        (
            '+infinity',
            False,
            Err("Input should be a finite number [type=finite_number, input_value='+infinity', input_type=str]"),
        ),
        ('-inf', True, FunctionCheck(lambda x: math.isinf(x) and x < 0)),
        (
            '-inf',
            False,
            Err("Input should be a finite number [type=finite_number, input_value='-inf', input_type=str]"),
        ),
        ('-infinity', True, FunctionCheck(lambda x: math.isinf(x) and x < 0)),
        (
            '-infinity',
            False,
            Err("Input should be a finite number [type=finite_number, input_value='-infinity', input_type=str]"),
        ),
        ('0.7', True, Decimal('0.7')),
        ('0.7', False, Decimal('0.7')),
        (
            'pika',
            True,
            Err("Input should be a valid decimal [type=decimal_parsing, input_value='pika', input_type=str]"),
        ),
        (
            'pika',
            False,
            Err("Input should be a valid decimal [type=decimal_parsing, input_value='pika', input_type=str]"),
        ),
    ],
)
def test_non_finite_json_values(py_and_json: PyAndJson, input_value, allow_inf_nan, expected):
    v = py_and_json({'type': 'decimal', 'allow_inf_nan': allow_inf_nan})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_test(input_value)
    else:
        assert v.validate_test(input_value) == expected


@pytest.mark.parametrize('strict', (True, False))
@pytest.mark.parametrize(
    'input_value,allow_inf_nan,expected',
    [
        (Decimal('nan'), True, FunctionCheck(math.isnan)),
        (
            Decimal('nan'),
            False,
            Err("Input should be a finite number [type=finite_number, input_value=Decimal('NaN'), input_type=Decimal]"),
        ),
    ],
)
def test_non_finite_decimal_values(strict, input_value, allow_inf_nan, expected):
    v = SchemaValidator(cs.decimal_schema(allow_inf_nan=allow_inf_nan, strict=strict))
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        assert v.validate_python(input_value) == expected


@pytest.mark.parametrize(
    'input_value,allow_inf_nan,expected',
    [
        (Decimal('+inf'), True, FunctionCheck(lambda x: math.isinf(x) and x > 0)),
        (
            Decimal('+inf'),
            False,
            Err(
                "Input should be a finite number [type=finite_number, input_value=Decimal('Infinity'), input_type=Decimal]"
            ),
        ),
        (
            Decimal('-inf'),
            True,
            Err(
                "Input should be greater than 0 [type=greater_than, input_value=Decimal('-Infinity'), input_type=Decimal]"
            ),
        ),
        (
            Decimal('-inf'),
            False,
            Err(
                "Input should be a finite number [type=finite_number, input_value=Decimal('-Infinity'), input_type=Decimal]"
            ),
        ),
    ],
)
def test_non_finite_constrained_decimal_values(input_value, allow_inf_nan, expected):
    v = SchemaValidator(cs.decimal_schema(allow_inf_nan=allow_inf_nan, gt=0))
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        assert v.validate_python(input_value) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        # lower e, minus
        ('1.0e-12', Decimal('1e-12')),
        ('1e-12', Decimal('1e-12')),
        ('12e-1', Decimal('12e-1')),
        # upper E, minus
        ('1.0E-12', Decimal('1e-12')),
        ('1E-12', Decimal('1e-12')),
        ('12E-1', Decimal('12e-1')),
        # lower E, plus
        ('1.0e+12', Decimal(' 1e12')),
        ('1e+12', Decimal(' 1e12')),
        ('12e+1', Decimal(' 12e1')),
        # upper E, plus
        ('1.0E+12', Decimal(' 1e12')),
        ('1E+12', Decimal(' 1e12')),
        ('12E+1', Decimal(' 12e1')),
        # lower E, unsigned
        ('1.0e12', Decimal(' 1e12')),
        ('1e12', Decimal(' 1e12')),
        ('12e1', Decimal(' 12e1')),
        # upper E, unsigned
        ('1.0E12', Decimal(' 1e12')),
        ('1E12', Decimal(' 1e12')),
        ('12E1', Decimal(' 12e1')),
    ],
)
def test_validate_scientific_notation_from_json(input_value, expected):
    v = SchemaValidator(cs.decimal_schema())
    assert v.validate_json(input_value) == expected


def test_validate_max_digits_and_decimal_places() -> None:
    v = SchemaValidator(cs.decimal_schema(max_digits=5, decimal_places=2))

    # valid inputs
    assert v.validate_json('1.23') == Decimal('1.23')
    assert v.validate_json('123.45') == Decimal('123.45')
    assert v.validate_json('-123.45') == Decimal('-123.45')

    # invalid inputs
    with pytest.raises(ValidationError):
        v.validate_json('1234.56')  # too many digits
    with pytest.raises(ValidationError):
        v.validate_json('123.456')  # too many decimal places
    with pytest.raises(ValidationError):
        v.validate_json('123456')  # too many digits
    with pytest.raises(ValidationError):
        v.validate_json('abc')  # not a valid decimal


def test_validate_max_digits_and_decimal_places_edge_case() -> None:
    v = SchemaValidator(cs.decimal_schema(max_digits=34, decimal_places=18))

    # valid inputs
    assert v.validate_python(Decimal('9999999999999999.999999999999999999')) == Decimal(
        '9999999999999999.999999999999999999'
    )


def test_str_validation_w_strict() -> None:
    s = SchemaValidator(cs.decimal_schema(strict=True))

    with pytest.raises(ValidationError):
        assert s.validate_python('1.23')


def test_str_validation_w_lax() -> None:
    s = SchemaValidator(cs.decimal_schema(strict=False))

    assert s.validate_python('1.23') == Decimal('1.23')


def test_union_with_str_prefers_str() -> None:
    s = SchemaValidator(cs.union_schema([cs.decimal_schema(), cs.str_schema()]))

    assert s.validate_python('1.23') == '1.23'
    assert s.validate_python(1.23) == Decimal('1.23')
