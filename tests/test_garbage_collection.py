import gc
from typing import Any
from weakref import WeakValueDictionary

from pydantic_core import SchemaSerializer, SchemaValidator, core_schema


def test_gc_schema_serializer() -> None:
    # test for https://github.com/pydantic/pydantic/issues/5136
    class BaseModel:
        __schema__: SchemaSerializer

        def __init_subclass__(cls) -> None:
            cls.__schema__ = SchemaSerializer(core_schema.model_schema(cls, core_schema.typed_dict_schema({})))

    cache: 'WeakValueDictionary[int, Any]' = WeakValueDictionary()

    for _ in range(10_000):

        class MyModel(BaseModel):
            pass

        cache[id(MyModel)] = MyModel

        del MyModel

    gc.collect(0)
    gc.collect(1)
    gc.collect(2)

    assert len(cache) == 0


def test_gc_schema_validator() -> None:
    # test for https://github.com/pydantic/pydantic/issues/5136
    class BaseModel:
        __validator__: SchemaValidator

        def __init_subclass__(cls) -> None:
            cls.__validator__ = SchemaValidator(core_schema.model_schema(cls, core_schema.typed_dict_schema({})))

    cache: 'WeakValueDictionary[int, Any]' = WeakValueDictionary()

    for _ in range(10_000):

        class MyModel(BaseModel):
            pass

        cache[id(MyModel)] = MyModel

        del MyModel

    gc.collect(0)
    gc.collect(1)
    gc.collect(2)

    assert len(cache) == 0
