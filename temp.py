# test

from pydantic_core import SchemaValidator, ValidationError

from pydantic import BaseModel

class Child(BaseModel):
    param: int
    another_param: int

class SomeModel(BaseModel):
    param: int
    another_param: int
    child: Child | None = None

print(SomeModel.__pydantic_core_schema__)
print(SomeModel.__pydantic_validator__)

v = SomeModel.__pydantic_validator__

print(dir(v))
print(v.construct_python({}, recursive=False))
print(v.construct_python({}, recursive=True))

model_data = {
    "param": 10,
    "another_param": "string",
    "child": {
        "param": 20,
        "another_param": 30
    }
}
model_copy = v.construct_python(model_data)

print(type(model_copy))
print(model_copy)

model_copy = v.construct_python(model_data, recursive=True)

print(type(model_copy))
print(model_copy)
