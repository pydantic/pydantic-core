import json
import re

import pytest

from pydantic_core import SchemaSerializer, core_schema


@pytest.mark.parametrize('input_value,expected_value', [(True, True), (False, False), (1, 1), (123, 123), (-42, -42)])
def test_union_bool_int(input_value, expected_value):
    v = SchemaSerializer(core_schema.union_schema(core_schema.bool_schema(), core_schema.int_schema()))
    assert v.to_python(input_value) == expected_value
    assert v.to_python(input_value, mode='json') == expected_value
    assert v.to_json(input_value) == json.dumps(expected_value).encode()


def test_union_error():
    v = SchemaSerializer(core_schema.union_schema(core_schema.bool_schema(), core_schema.int_schema()))
    msg = 'Expected `Union[bool, int]` but got `str` - serialized value may not be as expected'
    with pytest.warns(UserWarning, match=re.escape(msg)):
        assert v.to_python('a string') == 'a string'


class ModelA:
    def __init__(self, a, b):
        self.a = a
        self.b = b


class ModelB:
    def __init__(self, c, d):
        self.c = c
        self.d = d


@pytest.fixture(scope='module')
def model_serializer() -> SchemaSerializer:
    return SchemaSerializer(
        {
            'type': 'union',
            'choices': [
                {
                    'type': 'new-class',
                    'cls': ModelA,
                    'schema': {
                        'type': 'typed-dict',
                        'return_fields_set': True,
                        'fields': {
                            'a': {'schema': {'type': 'bytes'}},
                            'b': {
                                'schema': {
                                    'type': 'float',
                                    'serialization': {'type': 'format', 'formatting_string': '0.1f'},
                                }
                            },
                        },
                    },
                },
                {
                    'type': 'new-class',
                    'cls': ModelB,
                    'schema': {
                        'type': 'typed-dict',
                        'return_fields_set': True,
                        'fields': {
                            'c': {'schema': {'type': 'bytes'}},
                            'd': {
                                'schema': {
                                    'type': 'float',
                                    'serialization': {'type': 'format', 'formatting_string': '0.2f'},
                                }
                            },
                        },
                    },
                },
            ],
        }
    )


def test_model_a(model_serializer: SchemaSerializer):
    m_a = ModelA(b'bite', 2.3456)
    assert model_serializer.to_python(m_a) == {'a': b'bite', 'b': '2.3'}
    assert model_serializer.to_python(m_a, mode='json') == {'a': 'bite', 'b': '2.3'}
    assert model_serializer.to_json(m_a) == b'{"a":"bite","b":"2.3"}'


def test_model_b(model_serializer: SchemaSerializer):
    m_b = ModelB(b'bite', 2.3456)
    assert model_serializer.to_python(m_b) == {'c': b'bite', 'd': '2.35'}
    assert model_serializer.to_python(m_b, mode='json') == {'c': 'bite', 'd': '2.35'}
    assert model_serializer.to_json(m_b) == b'{"c":"bite","d":"2.35"}'
