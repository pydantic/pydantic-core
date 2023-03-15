import dataclasses

import pytest

import pydantic
from pydantic_core import core_schema, SchemaValidator


@dataclasses.dataclass
class StdLibDc:
    x: int
    y: int
    z: int


@pytest.mark.benchmark(group='dataclass')
def test_std_lib(benchmark):
    dc = StdLibDc(1, 2, z=3)
    assert dataclasses.asdict(dc) == {'x': 1, 'y': 2, 'z': 3}

    @benchmark
    def t():
        StdLibDc(1, 2, z=3)


@pydantic.dataclasses.dataclass
class PydanticDc:
    x: int
    y: int
    z: int


@pytest.mark.benchmark(group='dataclass')
def test_pydantic(benchmark):
    dc = PydanticDc(1, 2, z=3)
    assert dataclasses.asdict(dc) == {'x': 1, 'y': 2, 'z': 3}

    @benchmark
    def t():
        PydanticDc(1, 2, z=3)


list_data = [{'x': i, 'y': 2, 'z': 3} for i in range(1000)]


@pytest.mark.benchmark(group='dataclass-list')
def test_list_std_lib(benchmark):

    @benchmark
    def t():
        return [StdLibDc(**d) for d in list_data]



# @pytest.mark.benchmark(group='dataclass-list')
# def test_list_pydantic(benchmark):
#     v = SchemaValidator(core_schema.list_schema(PydanticDc.__pydantic_core_schema__))
#     dcs = v.validate_python([{'x': 1, 'y': 2, 'z': 3}, {'x': 4, 'y': 5, 'z': 6}])
#     assert dataclasses.asdict(dcs[0]) == {'x': 1, 'y': 2, 'z': 3}
#     assert dataclasses.asdict(dcs[1]) == {'x': 4, 'y': 5, 'z': 6}
#
#     @benchmark
#     def t():
#         return v.validate_python(list_data)

@pytest.mark.benchmark(group='dataclass-list')
def test_list_pydantic(benchmark):
    class MyModel(pydantic.BaseModel):
        dcs: list[PydanticDc]
    dcs = MyModel(dcs=[{'x': 1, 'y': 2, 'z': 3}, {'x': 4, 'y': 5, 'z': 6}]).dcs
    assert dataclasses.asdict(dcs[0]) == {'x': 1, 'y': 2, 'z': 3}
    assert dataclasses.asdict(dcs[1]) == {'x': 4, 'y': 5, 'z': 6}

    @benchmark
    def t():
        return MyModel(dcs=list_data).dcs
