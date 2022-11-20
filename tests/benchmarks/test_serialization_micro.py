from datetime import date

import pytest

from pydantic_core._pydantic_core import SchemaSerializer


@pytest.mark.benchmark(group='list-of-str')
def test_json_direct_list_str(benchmark):
    serializer = SchemaSerializer({'type': 'list', 'items_schema': {'type': 'str'}})
    assert serializer.to_json(list(map(str, range(5)))) == b'["0","1","2","3","4"]'

    items = list(map(str, range(1000)))
    benchmark(serializer.to_json, items)


@pytest.mark.benchmark(group='list-of-str')
def test_python_json_list_str(benchmark):
    serializer = SchemaSerializer({'type': 'list', 'items_schema': {'type': 'str'}})
    assert serializer.to_python(list(map(str, range(5))), mode='json') == ['0', '1', '2', '3', '4']

    items = list(map(str, range(1000)))

    @benchmark
    def t():
        serializer.to_python(items, mode='json')


@pytest.mark.benchmark(group='list-of-str')
def test_json_any_list_str(benchmark):
    serializer = SchemaSerializer({'type': 'list', 'items_schema': {'type': 'any'}})
    assert serializer.to_json(list(map(str, range(5)))) == b'["0","1","2","3","4"]'

    items = list(map(str, range(1000)))
    benchmark(serializer.to_json, items)


@pytest.mark.benchmark(group='list-of-int')
def test_json_direct_list_int(benchmark):
    serializer = SchemaSerializer({'type': 'list', 'items_schema': {'type': 'int'}})
    assert serializer.to_json(list(range(5))) == b'[0,1,2,3,4]'

    items = list(range(1000))
    benchmark(serializer.to_json, items)


@pytest.mark.benchmark(group='list-of-int')
def test_json_any_list_int(benchmark):
    serializer = SchemaSerializer({'type': 'list', 'items_schema': {'type': 'any'}})
    assert serializer.to_json(list(range(5))) == b'[0,1,2,3,4]'

    items = list(range(1000))
    benchmark(serializer.to_json, items)


@pytest.mark.benchmark(group='list-of-int')
def test_python_json_list_int(benchmark):
    serializer = SchemaSerializer({'type': 'list', 'items_schema': {'type': 'int'}})
    assert serializer.to_python(list(range(5)), mode='json') == [0, 1, 2, 3, 4]

    items = list(range(1000))

    @benchmark
    def t():
        serializer.to_python(items, mode='json')


@pytest.mark.benchmark(group='list-of-bool')
def test_python_json_list_none(benchmark):
    serializer = SchemaSerializer({'type': 'list', 'items_schema': {'type': 'none'}})
    assert serializer.to_python([None, None, None], mode='json') == [None, None, None]

    items = [None for v in range(1000)]

    @benchmark
    def t():
        serializer.to_python(items, mode='json')


@pytest.mark.benchmark(group='date-format')
def test_date_format(benchmark):
    serializer = SchemaSerializer({'type': 'any', 'serialization': {'type': 'format', 'formatting_string': '%Y-%m-%d'}})
    d = date(2022, 11, 20)
    assert serializer.to_python(d) == '2022-11-20'

    benchmark(serializer.to_python, d)


@pytest.mark.benchmark(group='date-format')
def test_date_format_function(benchmark):
    def fmt(value, **kwargs):
        return value.strftime('%Y-%m-%d')

    serializer = SchemaSerializer(
        {'type': 'any', 'serialization': {'type': 'function', 'function': fmt, 'return_type': 'str'}}
    )
    d = date(2022, 11, 20)
    assert serializer.to_python(d) == '2022-11-20'

    benchmark(serializer.to_python, d)
