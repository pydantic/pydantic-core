import pytest

from pydantic_core import SchemaError, SchemaValidator, ValidationError, core_schema

from ..conftest import plain_repr


class Foo:
    pass


class Bar(Foo):
    pass


class Spam:
    pass


def test_is_instance():
    v = SchemaValidator({'type': 'is-instance', 'cls': Foo})
    foo = Foo()
    assert v.validate_python(foo) == foo
    assert v.isinstance_python(foo) is True
    bar = Bar()
    assert v.validate_python(bar) == bar
    s = Spam()
    assert v.isinstance_python(s) is False
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python(s)

    assert exc_info.value.errors() == [
        {
            'kind': 'is_instance_of',
            'loc': [],
            'message': 'Input should be an instance of Foo',
            'input_value': s,
            'context': {'class': 'Foo'},
        }
    ]
    with pytest.raises(ValidationError, match='kind=is_instance_of'):
        v.validate_python(Foo)

    with pytest.raises(ValidationError, match='kind=is_instance_of'):
        v.validate_json('"foo"')


@pytest.mark.parametrize(
    'schema_class,input_val,value',
    [
        (Foo, Foo(), True),
        (Foo, Foo, False),
        (Foo, Bar(), True),
        (Foo, Bar, False),
        (Bar, Foo(), False),
        (Bar, Foo, False),
        (dict, {1: 2}, True),
        (dict, {1, 2}, False),
        (type, Foo, True),
        (type, Foo(), False),
    ],
)
def test_is_instance_cases(schema_class, input_val, value):
    v = SchemaValidator({'type': 'is-instance', 'cls': schema_class})
    assert v.isinstance_python(input_val) == value


@pytest.mark.parametrize('input_cls', [123, 'foo', Foo(), [], {1: 2}])
def test_is_instance_invalid(input_cls):
    with pytest.raises(SchemaError, match="object cannot be converted to 'PyType'"):
        SchemaValidator({'type': 'is-instance', 'cls': input_cls})


class HasIsInstanceMeta(type):
    def __instancecheck__(self, instance) -> bool:
        if 'error' in repr(instance):
            # an error here comes from a problem in the schema, not in the input value, so raise as internal error
            raise TypeError('intentional error')
        return 'true' in repr(instance)


class HasIsInstance(metaclass=HasIsInstanceMeta):
    pass


def test_instancecheck():
    v = SchemaValidator({'type': 'is-instance', 'cls': HasIsInstance})
    assert v.validate_python('true') == 'true'

    with pytest.raises(ValidationError, match='kind=is_instance_of'):
        v.validate_python('other')

    with pytest.raises(TypeError, match='intentional error'):
        v.validate_python('error')


def test_repr():
    v = SchemaValidator({'type': 'union', 'choices': [{'type': 'int'}, {'type': 'is-instance', 'cls': Foo}]})
    assert v.isinstance_python(4) is True
    assert v.isinstance_python(Bar()) is True
    assert v.isinstance_python('foo') is False

    with pytest.raises(ValidationError, match=r'is-instance\[Foo\]\s+Input should be an instance of Foo'):
        v.validate_python('foo')


@pytest.mark.parametrize(
    'input_val,value',
    [
        (Foo, True),
        (Foo(), False),
        (str, True),
        ('foo', False),
        (int, True),
        (1, False),
        (type, True),
        (type('Foobar', (), {'x': 1}), True),
    ],
)
def test_is_type(input_val, value):
    v = SchemaValidator({'type': 'is-instance', 'cls': type})
    assert v.isinstance_python(input_val) == value


@pytest.mark.parametrize(
    'input_val,expected',
    [
        ('null', False),
        ('true', True),
        ('1', False),
        ('1.1', False),
        ('"a string"', True),
        ('["s"]', False),
        ('{"s": 1}', False),
    ],
)
def test_is_instance_json_string_bool(input_val, expected):
    v = SchemaValidator(core_schema.is_instance_schema(Foo, json_types={'str', 'bool'}))
    assert v.isinstance_json(input_val) == expected


@pytest.mark.parametrize(
    'input_val,expected',
    [
        ('null', False),
        ('true', False),
        ('1', False),
        ('1.1', False),
        ('"a string"', False),
        ('["s"]', True),
        ('{"s": 1}', False),
    ],
)
def test_is_instance_json_list(input_val, expected):
    v = SchemaValidator(core_schema.is_instance_schema(Foo, json_types={'list'}))
    assert v.isinstance_json(input_val) == expected


def test_is_instance_dict():
    v = SchemaValidator(
        core_schema.dict_schema(
            keys_schema=core_schema.is_instance_schema(str, json_types={'str'}),
            values_schema=core_schema.is_instance_schema(int, json_types={'int', 'dict'}),
        )
    )
    assert v.isinstance_python({'foo': 1}) is True
    assert v.isinstance_python({1: 1}) is False
    assert v.isinstance_json('{"foo": 1}') is True
    assert v.isinstance_json('{"foo": "1"}') is False
    assert v.isinstance_json('{"foo": {"a": 1}}') is True


def test_json_mask():
    assert 'json_types:128' in plain_repr(SchemaValidator(core_schema.is_instance_schema(str, json_types={'null'})))
    assert 'json_types:0' in plain_repr(SchemaValidator(core_schema.is_instance_schema(str)))
    assert 'json_types:0' in plain_repr(SchemaValidator(core_schema.is_instance_schema(str, json_types=set())))
    v = SchemaValidator(core_schema.is_instance_schema(str, json_types={'list', 'dict'}))
    assert 'json_types:6' in plain_repr(v)  # 2 + 4
