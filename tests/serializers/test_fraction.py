from fractions import Fraction

import pytest

from pydantic_core import SchemaSerializer, core_schema


def test_fraction():
    v = SchemaSerializer(core_schema.fraction_schema())
    assert v.to_python(Fraction('123.456')) == Fraction('123.456')

    assert v.to_python(Fraction('123.456'), mode='json') == '123.456'
    assert v.to_json(Fraction('123.456')) == b'"123.456"'

    assert v.to_python(Fraction('123456789123456789123456789.123456789123456789123456789')) == Fraction(
        '123456789123456789123456789.123456789123456789123456789'
    )
    assert (
        v.to_json(Fraction('123456789123456789123456789.123456789123456789123456789'))
        == b'"123456789123456789123456789.123456789123456789123456789"'
    )

    # with pytest.warns(UserWarning, match='Expected `fraction` but got `int` - serialized value may not be as expected'):
    #     assert v.to_python(123, mode='json') == 123

    # with pytest.warns(UserWarning, match='Expected `fraction` but got `int` - serialized value may not be as expected'):
    #     assert v.to_json(123) == b'123'


def test_fraction_key():
    v = SchemaSerializer(core_schema.dict_schema(core_schema.fraction_schema(), core_schema.fraction_schema()))
    assert v.to_python({Fraction('123.456'): Fraction('123.456')}) == {Fraction('123.456'): Fraction('123.456')}
    assert v.to_python({Fraction('123.456'): Fraction('123.456')}, mode='json') == {'123.456': '123.456'}
    assert v.to_json({Fraction('123.456'): Fraction('123.456')}) == b'{"123.456":"123.456"}'


@pytest.mark.parametrize(
    'value,expected',
    [
        (Fraction('123.456'), '123.456'),
        (Fraction('Infinity'), 'Infinity'),
        (Fraction('-Infinity'), '-Infinity'),
        (Fraction('NaN'), 'NaN'),
    ],
)
def test_fraction_json(value, expected):
    v = SchemaSerializer(core_schema.fraction_schema())
    assert v.to_python(value, mode='json') == expected
    assert v.to_json(value).decode() == f'"{expected}"'


def test_any_fraction_key():
    v = SchemaSerializer(core_schema.dict_schema())
    input_value = {Fraction('123.456'): 1}

    assert v.to_python(input_value, mode='json') == {'123.456': 1}
    assert v.to_json(input_value) == b'{"123.456":1}'
