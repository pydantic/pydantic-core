# TODO:

# From the ABC documentation:
# "Since some set operations create new sets, the default mixin methods need a way to create new instances from an iterable. The class constructor is assumed to have a signature in the form ClassName(iterable). That assumption is factored-out to an internal classmethod called _from_iterable() which calls cls(iterable) to produce a new set. If the Set mixin is being used in a class with a different constructor signature, you will need to override _from_iterable() with a classmethod or regular method that can construct new instances from an iterable argument."
# This might allow us to create new instances of custom AbstractSet objects

# DONE:
# Dict
# Models
# Dataclasses
# List
# Tuple
# Unions (smart and left_to_right)
# Tagged Unions
# Nullable
# Set
# FrozenSet
# TypedDict
# `reconstruct_instances`
# `construct_defaults`
# custom subclasses of ABCs

from pydantic_core import SchemaValidator, ValidationError

from pydantic import BaseModel, RootModel, ConfigDict, dataclasses, Field

from collections import deque
from collections.abc import Sequence, MutableSequence, Set, MutableSet, Mapping, MutableMapping, MappingView
from typing import List, Tuple, Optional, Union, Literal, Dict


class Child(BaseModel):
    a: int
    b: int

    model_config = ConfigDict(extra='forbid')

class ChildExtra(BaseModel):
    a: int
    b: int
    extra: str

class SomeModel(BaseModel):
    param: int
    another_param: int
    child: Child | ChildExtra

# print(SomeModel.__pydantic_core_schema__)
# print(SomeModel.__pydantic_validator__)

v = SomeModel.__pydantic_validator__

# print(dir(v))
# print(v.construct_python({}, recursive=False))
# print(v.construct_python({}, recursive=True))

# non union value, has to pass as-is
# model_data = {
#     "param": 10,
#     "another_param": "string",
#     "child": None
# }
# model_copy = v.construct_python(model_data)
# print("\n", model_copy)
# model_copy = v.construct_python(model_data, recursive=True)
# print("\n", model_copy)
# assert model_copy.child is None

# model_data = {
#     "param": 10,
#     "another_param": "string",
#     "child": {
#         "child_param": 20,
#         "child_another_param": 30
#     }
# }
# model_copy = v.construct_python(model_data)
# print("\n", model_copy)
# model_copy = v.construct_python(model_data, recursive=True)
# print("\n", model_copy)
# assert isinstance(model_copy.child, Child) # will be Child because it comes first in Union

# model_data = {
#     "param": 10,
#     "another_param": "string",
#     "child": {
#         "child_param": 20,
#         "child_another_param": 30,
#         "extra": "extra"
#     }
# }
# model_copy = v.construct_python(model_data)
# print("\n", model_copy)
# model_copy = v.construct_python(model_data, recursive=True)
# print("\n", model_copy)
# assert isinstance(model_copy.child, Child) # will be Child because it comes first in Union

# class MyModel(BaseModel):
#     field_a: str
#     field_b: int

#     model_config = ConfigDict(extra="allow")

# input_dict = {'field_a': 'test', 'field_b': 12, 'field_c': 'extra'}
# reference = MyModel.model_construct(**input_dict)
# print(reference)
# print(reference.model_fields_set)
# print(reference.model_extra)
# print(reference.__dict__)
# # current = MyModel.__pydantic_validator__.construct_python(input_dict)
# # print(current)
# # print(current.model_fields_set)
# # print(current.model_extra)

# @dataclasses.dataclass(config=ConfigDict(extra="allow"))
# class TestDataclass:
#     id: int
#     name: str

# class NestedModel(BaseModel):
#     test: TestDataclass

# print(TestDataclass.__pydantic_core_schema__)

# v = NestedModel.__pydantic_validator__
# test_nested_dataclass = v.construct_python({"test": {"id": "wrong", "name": 10, "extra": "extra"}}, recursive=True)
# print("RESULT:")
# print(type(test_nested_dataclass))
# print(repr(test_nested_dataclass))
# print(test_nested_dataclass.test.extra)

class TestRootModel(RootModel):
    root: tuple[int, Child]

v = TestRootModel.__pydantic_validator__

m1 = TestRootModel.model_construct((10, {"a": 10, "b": "something"}))
print("\n", m1)
print(m1.model_fields)
print(m1.model_fields_set)
print(m1.__dict__)
print(m1.model_extra)
m2: TestRootModel = v.construct_python(["wrong", {"a": 10, "b": "something"}])
print("\n", m2)
print(m2.model_fields)
print(m2.model_fields_set)
print(m2.__dict__)
print(m2.model_extra)
m2: TestRootModel = v.construct_python(("wrong", {"a": 10, "b": "something"}), recursive=True)
print("\n", m2)
print(m2.model_fields)
print(m2.model_fields_set)
print(m2.__dict__)
print(m2.model_extra)

# Make sure attempting with incorrect annotated object doesn't raise
m3 = v.construct_python(["wrong", {"a": 10, "b": "something"}], recursive=True)
print(m3)

test_list = [1, 2, 3]
test_deque = deque([1, 2, 3])
assert isinstance(test_list, list)
assert isinstance(test_deque, deque)

res = Child.model_construct(**{"c": 10, "d": 10})
print(res.model_extra)

class D1(BaseModel):
    name: Literal["entity-1"]

class D2(BaseModel):
    name: Literal["entity-2"]

class TestDiscriminators(BaseModel):
    test: Union[D1, D2] = Field(..., discriminator="name")

v = TestDiscriminators.__pydantic_validator__

result = v.construct_python({"test": {"name": "entity-2"}}, recursive=True)
# Should `result.test` be a dict or an instance of Child? It *can* be converted to Child, but should it?
# Exactly if the user wants this to be converted is likely user-dependent...
print(result)
control = v.validate_python({"test": {"name": "entity-1"}})
print(control)

class TestDict(BaseModel):
    test: Dict[str, Child]

v = TestDict.__pydantic_validator__

m = v.construct_python({"test": None})
print("\n", m)
m = v.construct_python({"test": {"string": {"a": 10, "b": "wrong"}}})
print("\n", m)
m = v.construct_python({"test": {"string": {"a": 10, "b": "wrong"}}}, recursive=True)
print("\n", m)

class TestPrimitive(RootModel):
    root: MutableSet[Child]

v = TestPrimitive.__pydantic_validator__

m = v.construct_python([{"a": 10, "b": "wrong"}])
print("\n", m, "\n")
m = v.construct_python([{"a": 10, "b": "wrong"}], recursive=True)
print("\n", m, "\n")

lt = type([])
print(lt)
print(lt([1, 2, 3]))

class MyModel(BaseModel):
    field_a: str
    field_b: str

m = MyModel.model_construct(**{'field_a': 'test', 'field_b': 12, 'field_c': 'extra'})
print("\n", m, "\n")
print(m.__dict__)
print(m.model_extra)
print(m.model_fields_set)

# v = MyModel.__pydantic_validator__
# m = v.construct_python({'field_a': 'test', 'field_b': 12, 'field_c': 'extra'})

