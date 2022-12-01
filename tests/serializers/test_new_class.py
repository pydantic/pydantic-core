import json

import pytest
from dirty_equals import IsStrictDict

from pydantic_core import SchemaSerializer, core_schema


class BasicModel:
    def __init__(self, **kwargs):
        for key, value in kwargs.items():
            setattr(self, key, value)


def test_new_class():
    v = SchemaSerializer(
        core_schema.new_class_schema(
            type('Anything', (), {}),
            core_schema.typed_dict_schema(
                {
                    'foo': core_schema.typed_dict_field(core_schema.int_schema()),
                    'bar': core_schema.typed_dict_field(core_schema.bytes_schema()),
                }
            ),
        )
    )
    assert v.to_python(BasicModel(foo=1, bar=b'more')) == IsStrictDict(foo=1, bar=b'more')
    assert v.to_python(BasicModel(bar=b'more', foo=1)) == IsStrictDict(bar=b'more', foo=1)
    assert v.to_python(BasicModel(foo=1, c=3, bar=b'more')) == IsStrictDict(foo=1, bar=b'more')
    assert v.to_python(BasicModel(bar=b'more', foo=1, c=3), mode='json') == IsStrictDict(bar='more', foo=1)

    assert v.to_json(BasicModel(bar=b'more', foo=1, c=3)) == b'{"bar":"more","foo":1}'


def test_new_class_allow_extra():
    v = SchemaSerializer(
        core_schema.new_class_schema(
            BasicModel,
            core_schema.typed_dict_schema(
                {
                    'foo': core_schema.typed_dict_field(core_schema.int_schema()),
                    'bar': core_schema.typed_dict_field(core_schema.bytes_schema()),
                },
                extra_behavior='allow',
            ),
        )
    )
    assert v.to_python(BasicModel(foo=1, bar=b'more')) == IsStrictDict(foo=1, bar=b'more')
    assert v.to_python(BasicModel(bar=b'more', foo=1)) == IsStrictDict(bar=b'more', foo=1)
    assert v.to_python(BasicModel(foo=1, c=3, bar=b'more')) == IsStrictDict(foo=1, c=3, bar=b'more')
    assert v.to_python(BasicModel(bar=b'more', c=3, foo=1), mode='json') == IsStrictDict(bar='more', c=3, foo=1)

    assert v.to_json(BasicModel(bar=b'more', foo=1, c=3)) == b'{"bar":"more","foo":1,"c":3}'


@pytest.mark.parametrize(
    'params',
    [
        dict(include=None, exclude=None, expected={'a': 0, 'b': 1, 'c': 2, 'd': 3}),
        dict(include={'a', 'b'}, exclude=None, expected={'a': 0, 'b': 1}),
        dict(include={'a': None, 'b': None}, exclude=None, expected={'a': 0, 'b': 1}),
        dict(include={'a': {1}, 'b': {1}}, exclude=None, expected={'a': 0, 'b': 1}),
        dict(include=None, exclude={'a', 'b'}, expected={'c': 2, 'd': 3}),
        dict(include=None, exclude={'a': None, 'b': None}, expected={'c': 2, 'd': 3}),
        dict(include={'a', 'b'}, exclude={'b', 'c'}, expected={'a': 0}),
        dict(include=None, exclude={'d': {1}}, expected={'a': 0, 'b': 1, 'c': 2, 'd': 3}),
        dict(include={'a', 'b'}, exclude={'d': {1}}, expected={'a': 0, 'b': 1}),
        dict(include={'a', 'b'}, exclude={'b': {1}}, expected={'a': 0, 'b': 1}),
        dict(include={'a', 'b'}, exclude={'b': None}, expected={'a': 0}),
    ],
)
def test_include_exclude_args(params):
    s = SchemaSerializer(
        core_schema.new_class_schema(
            BasicModel,
            core_schema.typed_dict_schema(
                {
                    'a': core_schema.typed_dict_field(core_schema.int_schema()),
                    'b': core_schema.typed_dict_field(core_schema.int_schema()),
                    'c': core_schema.typed_dict_field(core_schema.int_schema()),
                    'd': core_schema.typed_dict_field(core_schema.int_schema()),
                }
            ),
        )
    )

    # user IsStrictDict to check dict order
    include, exclude, expected = params['include'], params['exclude'], IsStrictDict(params['expected'])
    value = BasicModel(a=0, b=1, c=2, d=3)
    assert s.to_python(value, include=include, exclude=exclude) == expected
    assert s.to_python(value, mode='json', include=include, exclude=exclude) == expected
    assert json.loads(s.to_json(value, include=include, exclude=exclude)) == expected


def test_alias():
    s = SchemaSerializer(
        core_schema.new_class_schema(
            BasicModel,
            core_schema.typed_dict_schema(
                {
                    'cat': core_schema.typed_dict_field(core_schema.int_schema(), serialization_alias='Meow'),
                    'dog': core_schema.typed_dict_field(core_schema.int_schema(), serialization_alias='Woof'),
                    'bird': core_schema.typed_dict_field(core_schema.int_schema()),
                }
            ),
        )
    )
    value = BasicModel(cat=0, dog=1, bird=2)
    assert s.to_python(value) == IsStrictDict(Meow=0, Woof=1, bird=2)


def test_new_class_wrong():
    v = SchemaSerializer(
        core_schema.new_class_schema(
            type('Anything', (), {}),
            core_schema.typed_dict_schema(
                {
                    'foo': core_schema.typed_dict_field(core_schema.int_schema()),
                    'bar': core_schema.typed_dict_field(core_schema.bytes_schema()),
                }
            ),
        )
    )
    with pytest.raises(AttributeError, match="'int' object has no attribute '__dict__'"):
        v.to_python(123)
    with pytest.raises(AttributeError, match="'dict' object has no attribute '__dict__'"):
        v.to_python({'foo': 1, 'bar': b'more'})


def test_exclude_none():
    v = SchemaSerializer(
        core_schema.new_class_schema(
            BasicModel,
            core_schema.typed_dict_schema(
                {
                    'foo': core_schema.typed_dict_field(core_schema.nullable_schema(core_schema.int_schema())),
                    'bar': core_schema.typed_dict_field(core_schema.bytes_schema()),
                },
                extra_behavior='ignore',  # this is the default
            ),
        )
    )
    assert v.to_python(BasicModel(foo=1, bar=b'more')) == {'foo': 1, 'bar': b'more'}
    assert v.to_python(BasicModel(foo=None, bar=b'more')) == {'foo': None, 'bar': b'more'}
    assert v.to_python(BasicModel(foo=None, bar=b'more'), exclude_none=True) == {'bar': b'more'}

    assert v.to_python(BasicModel(foo=None, bar=b'more'), mode='json') == {'foo': None, 'bar': 'more'}
    assert v.to_python(BasicModel(foo=None, bar=b'more'), mode='json', exclude_none=True) == {'bar': 'more'}

    assert v.to_json(BasicModel(foo=1, bar=b'more')) == b'{"foo":1,"bar":"more"}'
    assert v.to_json(BasicModel(foo=None, bar=b'more')) == b'{"foo":null,"bar":"more"}'
    assert v.to_json(BasicModel(foo=None, bar=b'more'), exclude_none=True) == b'{"bar":"more"}'
