import pytest

from pydantic_core import SchemaValidator, ValidationError


def test_after():
    def f(input_value, *, context, **kwargs):
        return input_value + f'| context: {context}'

    v = SchemaValidator({'type': 'function', 'mode': 'after', 'function': f, 'schema': 'str'})

    assert v.validate_python('foobar') == 'foobar| context: None'
    assert v.validate_python('foobar', None, {1: 10}) == 'foobar| context: {1: 10}'
    assert v.validate_json('"foobar"', None, {1: 10}) == 'foobar| context: {1: 10}'
    assert v.validate_python('foobar', None, 'frogspawn') == 'foobar| context: frogspawn'


def test_mutable_context():
    def f(input_value, *, context, **kwargs):
        context['foo'] = input_value
        return input_value

    v = SchemaValidator({'type': 'function', 'mode': 'before', 'function': f, 'schema': 'str'})
    mutable_context = {}
    assert v.validate_python('foobar', None, mutable_context) == 'foobar'
    assert mutable_context == {'foo': 'foobar'}


def test_typed_dict():
    def f1(input_value, *, context, **kwargs):
        context['f1'] = input_value
        return input_value + f'| context: {context}'

    def f2(input_value, *, context, **kwargs):
        context['f2'] = input_value
        return input_value + f'| context: {context}'

    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'f1': {'schema': {'type': 'function', 'mode': 'plain', 'function': f1}},
                'f2': {'schema': {'type': 'function', 'mode': 'plain', 'function': f2}},
            },
        }
    )

    assert v.validate_python({'f1': '1', 'f2': '2'}, None, {'x': 'y'}) == {
        'f1': "1| context: {'x': 'y', 'f1': '1'}",
        'f2': "2| context: {'x': 'y', 'f1': '1', 'f2': '2'}",
    }


def test_wrap():
    def f(input_value, *, validator, context, **kwargs):
        return validator(input_value) + f'| context: {context}'

    v = SchemaValidator({'type': 'function', 'mode': 'wrap', 'function': f, 'schema': 'str'})

    assert v.validate_python('foobar') == 'foobar| context: None'
    assert v.validate_python('foobar', None, {1: 10}) == 'foobar| context: {1: 10}'
    assert v.validate_json('"foobar"', None, {1: 10}) == 'foobar| context: {1: 10}'
    assert v.validate_python('foobar', None, 'frogspawn') == 'foobar| context: frogspawn'


def test_isinstance():
    def f(input_value, *, validator, context, **kwargs):
        if 'error' in context:
            raise ValueError('wrong')
        return validator(input_value)

    v = SchemaValidator({'type': 'function', 'mode': 'wrap', 'function': f, 'schema': 'str'})

    assert v.validate_python('foobar', None, {}) == 'foobar'
    assert v.validate_json('"foobar"', None, {}) == 'foobar'

    with pytest.raises(TypeError, match="argument of type 'NoneType' is not iterable"):  # internal error!
        v.validate_python('foobar')

    with pytest.raises(TypeError, match="argument of type 'NoneType' is not iterable"):  # internal error!
        v.isinstance_python('foobar')

    with pytest.raises(ValidationError, match=r'Value error, wrong \[kind=value_error,'):
        v.validate_python('foobar', None, {'error'})

    with pytest.raises(ValidationError, match=r'Value error, wrong \[kind=value_error,'):
        v.validate_json('"foobar"', None, {'error'})

    assert v.isinstance_python('foobar', None, {}) is True
    assert v.isinstance_json('"foobar"', None, {}) is True

    with pytest.raises(TypeError, match="argument of type 'NoneType' is not iterable"):  # internal error!
        v.isinstance_python('foobar')

    assert v.isinstance_python('foobar', None, {'error'}) is False
    assert v.isinstance_json('"foobar"', None, {'error'}) is False
