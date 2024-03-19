import re
from enum import Enum

import pytest

from pydantic_core import SchemaValidator, ValidationError, core_schema


def test_plain_enum():
    class MyEnum(Enum):
        a = 1
        b = 2

    v = SchemaValidator(core_schema.enum_schema(MyEnum, list(MyEnum.__members__.values())))

    # debug(v)
    assert v.validate_python(MyEnum.a) is MyEnum.a
    assert v.validate_python(1) is MyEnum.a

    assert v.validate_json('1') is MyEnum.a

    with pytest.raises(ValidationError, match=r'Input should be 1 or 2 \[type=enum, input_value=3, input_type=int\]'):
        v.validate_python(3)

    with pytest.raises(ValidationError, match=r"Input should be 1 or 2 \[type=enum, input_value='1', input_type=str\]"):
        v.validate_python('1')

    assert v.validate_python(MyEnum.a, strict=True) is MyEnum.a

    e = (
        'Input should be an instance of test_plain_enum.<locals>.MyEnum '
        '[type=is_instance_of, input_value=1, input_type=int]'
    )
    with pytest.raises(ValidationError, match=re.escape(e)):
        v.validate_python(1, strict=True)


def test_int_enum():
    class MyEnum(int, Enum):
        a = 1
        b = 2

    v = SchemaValidator(core_schema.enum_schema(MyEnum, list(MyEnum.__members__.values()), sub_type='int'))

    # debug(v)
    assert v.validate_python(MyEnum.a) is MyEnum.a
    assert v.validate_python(1) is MyEnum.a
    assert v.validate_python(1.0) is MyEnum.a
    assert v.validate_python('1') is MyEnum.a

    assert v.validate_json('1') is MyEnum.a
    assert v.validate_json('"1"') is MyEnum.a

    with pytest.raises(ValidationError, match=r'Input should be 1 or 2 \[type=enum, input_value=3, input_type=int\]'):
        v.validate_python(3)

    assert v.validate_python(MyEnum.a, strict=True) is MyEnum.a

    e = (
        'Input should be an instance of test_int_enum.<locals>.MyEnum '
        '[type=is_instance_of, input_value=1, input_type=int]'
    )
    with pytest.raises(ValidationError, match=re.escape(e)):
        v.validate_python(1, strict=True)


def test_str_enum():
    class MyEnum(str, Enum):
        a = 'x'
        b = 'y'

    v = SchemaValidator(core_schema.enum_schema(MyEnum, list(MyEnum.__members__.values()), sub_type='str'))

    # debug(v)
    assert v.validate_python('x') is MyEnum.a
    assert v.validate_python(MyEnum.a) is MyEnum.a
    assert v.validate_python(b'x') is MyEnum.a

    assert v.validate_json('"x"') is MyEnum.a

    with pytest.raises(
        ValidationError, match=r"Input should be 'x' or 'y' \[type=enum, input_value='a', input_type=str\]"
    ):
        v.validate_python('a')

    assert v.validate_python(MyEnum.a, strict=True) is MyEnum.a

    e = (
        'Input should be an instance of test_str_enum.<locals>.MyEnum '
        "[type=is_instance_of, input_value='x', input_type=str]"
    )
    with pytest.raises(ValidationError, match=re.escape(e)):
        v.validate_python('x', strict=True)


def test_float_enum():
    class MyEnum(float, Enum):
        a = 1.5
        b = 2.5
        c = 3.0

    v = SchemaValidator(core_schema.enum_schema(MyEnum, list(MyEnum.__members__.values()), sub_type='float'))

    # debug(v)
    assert v.validate_python(MyEnum.a) is MyEnum.a
    assert v.validate_python(1.5) is MyEnum.a
    assert v.validate_python('1.5') is MyEnum.a
    assert v.validate_python(3) is MyEnum.c

    assert v.validate_json('1.5') is MyEnum.a
    # assert v.validate_json('"1.5"') is MyEnum.a

    e = r'Input should be 1.5, 2.5 or 3.0 \[type=enum, input_value=4.0, input_type=float\]'
    with pytest.raises(ValidationError, match=e):
        v.validate_python(4.0)

    assert v.validate_python(MyEnum.a, strict=True) is MyEnum.a

    e = (
        'Input should be an instance of test_float_enum.<locals>.MyEnum '
        '[type=is_instance_of, input_value=1.5, input_type=float]'
    )
    with pytest.raises(ValidationError, match=re.escape(e)):
        v.validate_python(1.5, strict=True)
