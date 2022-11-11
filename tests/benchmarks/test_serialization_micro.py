import pytest

from pydantic_core._pydantic_core import SchemaSerializer


@pytest.mark.benchmark(group='list-of-str')
def test_direct_list_str(benchmark):
    serializer = SchemaSerializer({'type': 'list', 'items_schema': {'type': 'str'}})
    assert serializer.to_json(list(map(str, range(5)))) == b'["0","1","2","3","4"]'

    items = list(map(str, range(1000)))
    benchmark(serializer.to_json, items)


@pytest.mark.benchmark(group='list-of-str')
def test_any_list_str(benchmark):
    serializer = SchemaSerializer({'type': 'list', 'items_schema': {'type': 'any'}})
    assert serializer.to_json(list(map(str, range(5)))) == b'["0","1","2","3","4"]'

    items = list(map(str, range(1000)))
    benchmark(serializer.to_json, items)


@pytest.mark.benchmark(group='list-of-int')
def test_direct_list_int(benchmark):
    serializer = SchemaSerializer({'type': 'list', 'items_schema': {'type': 'int'}})
    assert serializer.to_json(list(range(5))) == b'[0,1,2,3,4]'

    items = list(range(1000))
    benchmark(serializer.to_json, items)


@pytest.mark.benchmark(group='list-of-int')
def test_any_list_int(benchmark):
    serializer = SchemaSerializer({'type': 'list', 'items_schema': {'type': 'any'}})
    assert serializer.to_json(list(range(5))) == b'[0,1,2,3,4]'

    items = list(range(1000))
    benchmark(serializer.to_json, items)
