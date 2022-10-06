"""
Numerous benchmarks of specific functionality.
"""
import json
import os
import platform
from datetime import date, datetime, timedelta, timezone
from decimal import Decimal
from typing import Dict, FrozenSet, List, Optional, Set, Union

import pytest

from pydantic_core import (
    PydanticValueError,
    SchemaValidator,
    ValidationError,
    ValidationError as CoreValidationError,
    core_schema,
)

if os.getenv('BENCHMARK_VS_PYDANTIC'):
    try:
        from pydantic import BaseModel, StrictBool, StrictInt, StrictStr, ValidationError as PydanticValidationError
    except ImportError:
        BaseModel = None
else:
    BaseModel = None

skip_pydantic = pytest.mark.skipif(BaseModel is None, reason='skipping benchmarks vs. pydantic')

bool_cases = [True, False, 0, 1, '0', '1', 'true', 'false', 'True', 'False']


@skip_pydantic
@pytest.mark.benchmark(group='bool')
def test_bool_pyd(benchmark):
    class PydanticModel(BaseModel):
        value: bool

    @benchmark
    def t():
        for case in bool_cases:
            PydanticModel(value=case)


@pytest.mark.benchmark(group='bool')
def test_bool_core(benchmark):
    schema_validator = SchemaValidator({'type': 'bool'})

    @benchmark
    def t():
        for case in bool_cases:
            schema_validator.validate_python(case)


small_class_data = {'name': 'John', 'age': 42}


@skip_pydantic
@pytest.mark.benchmark(group='create small model')
def test_small_class_pyd(benchmark):
    class PydanticModel(BaseModel):
        name: str
        age: int

    benchmark(PydanticModel.parse_obj, small_class_data)


@pytest.mark.benchmark(group='create small model')
def test_small_class_core_dict(benchmark):
    model_schema = {
        'type': 'typed-dict',
        'fields': {'name': {'schema': {'type': 'str'}}, 'age': {'schema': {'type': 'int'}}},
    }
    dict_schema_validator = SchemaValidator(model_schema)
    benchmark(dict_schema_validator.validate_python, small_class_data)


@pytest.mark.benchmark(group='create small model')
def test_small_class_core_model(benchmark):
    class MyCoreModel:
        # this is not required, but it avoids `__fields_set__` being included in `__dict__`
        __slots__ = '__dict__', '__fields_set__'
        # these are here just as decoration
        name: str
        age: int

    model_schema_validator = SchemaValidator(
        {
            'type': 'new-class',
            'cls': MyCoreModel,
            'schema': {
                'type': 'typed-dict',
                'return_fields_set': True,
                'fields': {'name': {'schema': {'type': 'str'}}, 'age': {'schema': {'type': 'int'}}},
            },
        }
    )
    benchmark(model_schema_validator.validate_python, small_class_data)


@pytest.mark.benchmark(group='string')
def test_core_string_lax(benchmark):
    validator = SchemaValidator({'type': 'str'})
    input_str = 'Hello ' * 20

    benchmark(validator.validate_python, input_str)


@pytest.fixture
def recursive_model_data():
    data = {'width': -1}

    _data = data
    for i in range(100):
        _data['branch'] = {'width': i}
        _data = _data['branch']
    return data


@pytest.mark.skipif(platform.python_implementation() == 'PyPy', reason='crashes on pypy due to recursion depth')
@skip_pydantic
@pytest.mark.benchmark(group='recursive model')
def test_recursive_model_pyd(recursive_model_data, benchmark):
    class PydanticBranch(BaseModel):
        width: int
        branch: Optional['PydanticBranch'] = None  # noqa: F821

    benchmark(PydanticBranch.parse_obj, recursive_model_data)


@pytest.mark.skipif(platform.python_implementation() == 'PyPy', reason='crashes on pypy due to recursion depth')
@pytest.mark.benchmark(group='recursive model')
def test_recursive_model_core(recursive_model_data, benchmark):
    class CoreBranch:
        # this is not required, but it avoids `__fields_set__` being included in `__dict__`
        __slots__ = '__dict__', '__fields_set__'

    v = SchemaValidator(
        {
            'ref': 'Branch',
            'type': 'new-class',
            'cls': CoreBranch,
            'schema': {
                'type': 'typed-dict',
                'return_fields_set': True,
                'fields': {
                    'width': {'schema': {'type': 'int'}},
                    'branch': {
                        'schema': {
                            'type': 'default',
                            'schema': {'type': 'nullable', 'schema': {'type': 'recursive-ref', 'schema_ref': 'Branch'}},
                            'default': None,
                        }
                    },
                },
            },
        }
    )
    benchmark(v.validate_python, recursive_model_data)


@skip_pydantic
@pytest.mark.benchmark(group='List[TypedDict]')
def test_list_of_dict_models_pyd(benchmark):
    class PydanticBranch(BaseModel):
        width: int

    class PydanticRoot(BaseModel):
        __root__: List[PydanticBranch]

    data = [{'width': i} for i in range(100)]
    benchmark(PydanticRoot.parse_obj, data)


@pytest.mark.benchmark(group='List[TypedDict]')
def test_list_of_dict_models_core(benchmark):
    v = SchemaValidator(
        {'type': 'list', 'items_schema': {'type': 'typed-dict', 'fields': {'width': {'schema': {'type': 'int'}}}}}
    )

    data = [{'width': i} for i in range(100)]
    benchmark(v.validate_python, data)


list_of_ints_data = ([i for i in range(1000)], [str(i) for i in range(1000)])


@skip_pydantic
@pytest.mark.benchmark(group='List[int]')
def test_list_of_ints_pyd_py(benchmark):
    class PydanticModel(BaseModel):
        __root__: List[int]

    @benchmark
    def t():
        PydanticModel.parse_obj(list_of_ints_data[0])
        PydanticModel.parse_obj(list_of_ints_data[1])


@pytest.mark.benchmark(group='List[int]')
def test_list_of_ints_core_py(benchmark):
    v = SchemaValidator({'type': 'list', 'items_schema': {'type': 'int'}})

    @benchmark
    def t():
        v.validate_python(list_of_ints_data[0])
        v.validate_python(list_of_ints_data[1])


@skip_pydantic
@pytest.mark.benchmark(group='List[int] JSON')
def test_list_of_ints_pyd_json(benchmark):
    class PydanticModel(BaseModel):
        __root__: List[int]

    json_data = [json.dumps(d) for d in list_of_ints_data]

    @benchmark
    def t():
        PydanticModel.parse_obj(json.loads(json_data[0]))
        PydanticModel.parse_obj(json.loads(json_data[1]))


@pytest.mark.benchmark(group='List[int] JSON')
def test_list_of_ints_core_json(benchmark):
    v = SchemaValidator({'type': 'list', 'items_schema': {'type': 'int'}})

    json_data = [json.dumps(d) for d in list_of_ints_data]

    @benchmark
    def t():
        v.validate_json(json_data[0])
        v.validate_json(json_data[1])


@skip_pydantic
@pytest.mark.benchmark(group='List[Any]')
def test_list_of_any_pyd_py(benchmark):
    class PydanticModel(BaseModel):
        __root__: list

    @benchmark
    def t():
        PydanticModel.parse_obj(list_of_ints_data[0])
        PydanticModel.parse_obj(list_of_ints_data[1])


@pytest.mark.benchmark(group='List[Any]')
def test_list_of_any_core_py(benchmark):
    v = SchemaValidator({'type': 'list'})

    @benchmark
    def t():
        v.validate_python(list_of_ints_data[0])
        v.validate_python(list_of_ints_data[1])


set_of_ints_data = ({i for i in range(1000)}, {str(i) for i in range(1000)})


@skip_pydantic
@pytest.mark.benchmark(group='Set[int]')
def test_set_of_ints_pyd(benchmark):
    class PydanticModel(BaseModel):
        __root__: Set[int]

    @benchmark
    def t():
        PydanticModel.parse_obj(set_of_ints_data[0])
        PydanticModel.parse_obj(set_of_ints_data[1])


@pytest.mark.benchmark(group='Set[int]')
def test_set_of_ints_core(benchmark):
    v = SchemaValidator({'type': 'set', 'items_schema': {'type': 'int'}})

    @benchmark
    def t():
        v.validate_python(set_of_ints_data[0])
        v.validate_python(set_of_ints_data[1])


@skip_pydantic
@pytest.mark.benchmark(group='Set[int] JSON')
def test_set_of_ints_pyd_json(benchmark):
    class PydanticModel(BaseModel):
        __root__: Set[int]

    json_data = [json.dumps(list(d)) for d in set_of_ints_data]

    @benchmark
    def t():
        PydanticModel.parse_obj(json.loads(json_data[0]))
        PydanticModel.parse_obj(json.loads(json_data[1]))

