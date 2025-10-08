import os
from enum import Enum

from pydantic_core import SchemaSerializer, core_schema


# serializing enum calls methods in serializers::infer
def test_infer_complex_to_python():
    class MyEnum(Enum):
        complex_ = complex(1, 2)

    v = SchemaSerializer(core_schema.enum_schema(MyEnum, list(MyEnum.__members__.values())))
    assert v.to_python(MyEnum.complex_, mode='json') == '1+2j'


def test_infer_complex_serialize():
    class MyEnum(Enum):
        complex_ = complex(1, 2)

    v = SchemaSerializer(core_schema.enum_schema(MyEnum, list(MyEnum.__members__.values())))
    assert v.to_json(MyEnum.complex_) == b'"1+2j"'


def test_infer_complex_json_key():
    class MyEnum(Enum):
        complex_ = {complex(1, 2): 1}

    v = SchemaSerializer(core_schema.enum_schema(MyEnum, list(MyEnum.__members__.values())))
    assert v.to_json(MyEnum.complex_) == b'{"1+2j":1}'


def test_infer_module_type():
    v = SchemaSerializer(core_schema.any_schema())
    assert v.to_python(os) is os
    assert v.to_json(os).decode('utf-8') == '"os"'
    assert v.to_python(os, serialize_as_any=True) is os
    assert v.to_json(os, serialize_as_any=True).decode('utf-8') == '"os"'

    v_as_key = SchemaSerializer(
        core_schema.dict_schema(keys_schema=core_schema.any_schema(), values_schema=core_schema.any_schema())
    )

    assert v_as_key.to_python({os: 1}) == {os: 1}
    assert v_as_key.to_json({os: 1}).decode('utf-8') == '{"os":1}'
    assert v_as_key.to_python({os: 1}, serialize_as_any=True) == {os: 1}
    assert v_as_key.to_json({os: 1}, serialize_as_any=True).decode('utf-8') == '{"os":1}'
