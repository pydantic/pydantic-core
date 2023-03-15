from pydantic_core import SchemaValidator


class MyModel:
    # this is not required, but it avoids `__fields_set__` being included in `__dict__`
    __slots__ = '__dict__', '__fields_set__'
    field_a: str
    field_b: int


def test_model_init():

    v = SchemaValidator(
        {
            'type': 'model',
            'cls': MyModel,
            'schema': {
                'type': 'typed-dict',
                'return_fields_set': True,
                'fields': {'field_a': {'schema': {'type': 'str'}}, 'field_b': {'schema': {'type': 'int'}}},
            },
        }
    )
    m = v.validate_python({'field_a': 'test', 'field_b': 12})
    assert isinstance(m, MyModel)
    assert m.field_a == 'test'
    assert m.field_b == 12
    d, fields_set = v.validate_python({'field_a': 'test', 'field_b': 12}, init_mode=True)
    assert d == {'field_a': 'test', 'field_b': 12}
    assert fields_set == {'field_a', 'field_b'}


def test_model_init_nested():
    class MyModel:
        # this is not required, but it avoids `__fields_set__` being included in `__dict__`
        __slots__ = '__dict__', '__fields_set__'

    v = SchemaValidator(
        {
            'type': 'model',
            'cls': MyModel,
            'schema': {
                'type': 'typed-dict',
                'return_fields_set': True,
                'fields': {
                    'field_a': {'schema': {'type': 'str'}},
                    'field_b': {
                        'schema': {
                            'type': 'model',
                            'cls': MyModel,
                            'schema': {
                                'type': 'typed-dict',
                                'return_fields_set': True,
                                'fields': {'x_a': {'schema': {'type': 'str'}}, 'x_b': {'schema': {'type': 'int'}}},
                            },
                        }
                    },
                },
            },
        }
    )
    m = v.validate_python({'field_a': 'test', 'field_b': {'x_a': 'foo', 'x_b': 12}})
    assert isinstance(m, MyModel)
    assert m.field_a == 'test'
    assert isinstance(m.field_b, MyModel)
    assert m.field_b.x_a == 'foo'
    assert m.field_b.x_b == 12
    d, fields_set = v.validate_python({'field_a': 'test', 'field_b': {'x_a': 'foo', 'x_b': 12}}, init_mode=True)
    assert d['field_a'] == 'test'
    assert isinstance(d['field_b'], MyModel)
    assert d['field_b'].x_a == 'foo'
    assert d['field_b'].x_b == 12

    assert fields_set == {'field_a', 'field_b'}


def test_function_before():
    def f(input_value, _info):
        assert isinstance(input_value, dict)
        input_value['field_a'] += b' XX'
        return input_value

    v = SchemaValidator(
        {
            'type': 'function',
            'mode': 'before',
            'function': f,
            'schema': {
                'type': 'model',
                'cls': MyModel,
                'schema': {
                    'type': 'typed-dict',
                    'return_fields_set': True,
                    'fields': {'field_a': {'schema': {'type': 'str'}}, 'field_b': {'schema': {'type': 'int'}}},
                },
            },
        }
    )

    m = v.validate_python({'field_a': b'321', 'field_b': '12'})
    assert isinstance(m, MyModel)
    assert m.field_a == '321 XX'
    assert m.field_b == 12

    d, fields_set = v.validate_python({'field_a': b'321', 'field_b': '12'}, init_mode=True)
    assert d == {'field_a': '321 XX', 'field_b': 12}
    assert fields_set == {'field_a', 'field_b'}


def test_function_after():
    value_type = None

    def f(input_value, _info):
        nonlocal value_type
        if isinstance(input_value, MyModel):
            value_type = 'model'
            input_value.field_a += ' Changed'
        else:
            value_type = 'dict,fields_set'
            input_value[0]['field_a'] += ' Changed'
        return input_value

    v = SchemaValidator(
        {
            'type': 'function',
            'mode': 'after',
            'function': f,
            'schema': {
                'type': 'model',
                'cls': MyModel,
                'schema': {
                    'type': 'typed-dict',
                    'return_fields_set': True,
                    'fields': {'field_a': {'schema': {'type': 'str'}}, 'field_b': {'schema': {'type': 'int'}}},
                },
            },
        }
    )

    m = v.validate_python({'field_a': b'321', 'field_b': '12'})
    assert isinstance(m, MyModel)
    assert m.field_a == '321 Changed'
    assert m.field_b == 12
    assert value_type == 'model'

    d, fields_set = v.validate_python({'field_a': b'321', 'field_b': '12'}, init_mode=True)
    assert d == {'field_a': '321 Changed', 'field_b': 12}
    assert fields_set == {'field_a', 'field_b'}
    assert value_type == 'dict,fields_set'


def test_function_wrap():
    value_type = None

    def f(input_value, handler, _info):
        nonlocal value_type
        assert isinstance(input_value, dict)
        v = handler(input_value)
        if isinstance(v, MyModel):
            value_type = 'model'
            v.field_a += ' Changed'
        else:
            value_type = 'dict,fields_set'
            v[0]['field_a'] += ' Changed'
        return v

    v = SchemaValidator(
        {
            'type': 'function',
            'mode': 'wrap',
            'function': f,
            'schema': {
                'type': 'model',
                'cls': MyModel,
                'schema': {
                    'type': 'typed-dict',
                    'return_fields_set': True,
                    'fields': {'field_a': {'schema': {'type': 'str'}}, 'field_b': {'schema': {'type': 'int'}}},
                },
            },
        }
    )

    m = v.validate_python({'field_a': b'321', 'field_b': '12'})
    assert isinstance(m, MyModel)
    assert m.field_a == '321 Changed'
    assert m.field_b == 12
    assert value_type == 'model'

    d, fields_set = v.validate_python({'field_a': b'321', 'field_b': '12'}, init_mode=True)
    assert d == {'field_a': '321 Changed', 'field_b': 12}
    assert fields_set == {'field_a', 'field_b'}
    assert value_type == 'dict,fields_set'


def test_simple():
    v = SchemaValidator({'type': 'str'})
    assert v.validate_python(b'abc') == 'abc'
    assert v.isinstance_python(b'abc') is True

    assert v.validate_python(b'abc', init_mode=True) == 'abc'
    assert v.isinstance_python(b'abc', init_mode=True) is True

    assert v.validate_json('"abc"') == 'abc'
    assert v.isinstance_json('"abc"') is True

    assert v.validate_json('"abc"', init_mode=True) == 'abc'
    assert v.isinstance_json('"abc"', init_mode=True) is True
