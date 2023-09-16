# This PR:

* Adds a `construct_python()` function to `SchemaValidator`, which functions identically to the `model_construct()` function currently used by Pydantic
    * Has an additional `recursive` flag which enables the traverse the schema and construct any found sub-models, similar to `validate`
* Is a rewrite of [this Pydantic PR]() I made a couple weeks ago; see there for motivation on why I believe this is a worthwhile feature

# Features:

"Constructs" a given Model from an input if possible, skipping any validation checks. This is theoretically more performant than validating, but more importantly it allows the user to control exactly when they can validate a given model; a user can construct a model and gain the same structure as a validated model, and validate it later at their own discretion, or even never. In addition, this function is able to "recurse" through the model, traversing it's schema looking for fields annotated with instances of `Model` or `Dataclass`, which can also be constructed. Recursion also searches annotation arguments, so annotations like `Dict[str, BaseModel]` or `List[List[Dataclass]]` are properly handled. This allows models with deeply nested annotations of other models to be generated with a single line of code instead of having the user manually traverse their own structure/schema themselves.

Recursion is implemented for:

* `Model` and `Dataclass`
* `Unions`, smart, left-to-right, and discriminated
* `Dict`, both keys and values can be constructed
* `Tuple`, both positional and variable
* `List`
* `Set` and `FrozenSet`
* `Nullable` 

# Implementation Details:

Added the method `construct_python()` and `construct_json()` to `SchemaValidator`. Both methods share the same signature:

* `input`: A dictionary object to traverse,
* `fields_set`: The set of fields to overwrite the newly constructed model, as currently implemented in `BaseModel.model_construct()`, and
* a new parameter `recursive`, which indicates whether or not to just construct the base-most model (current behavior), or to recurse through annotations and try to construct them as well (similar to `validate()`).

Both methods call a new trait method `Validator.construct()`. The `construct()` method is provided a default which simply returns the input object, so special recursive implementations only need to be implemented on the classes that need them. I added a new method instead of editing `validate()` for 3 reasons:

1. Construction is similar to Validation, but distinct; a lot of code can be reused, but they are distinct operations that would probably benefit from being their own methods.
2. Breakages/bugs in one set of methods will not affect the other, especially since `construct()` is new and experimental, both in terms of concept and functionality.
3. Can omit bringing extra information for validation that we ultimately will never use, as the data needed for one operation is largely different from the other.

The `construct()` function takes the same parameters as `validate()`, except that instead of a `ValidationState` struc a `ConstructionState` struct is passed. `ConstructionState` is basically a stripped down version of `ValidationState`, since the amount of information we need to carry when traversing is much smaller than with validation. It only has 5 members:

```rs
pub struct ConstructionState<'a> {
    pub fields_set: Option<&'a PyAny>,
    pub recursive: bool,
    pub strict: bool,
    pub ultra_strict: bool,
    pub definitions: &'a Definitions<CombinedValidator>,
}
```

`fields_set` and `recursive` are self-explanatory. `strict` and `ultra_strict` are used in smart union construction to indicate "quality" of the construction. If `ultra_strict` is true, then model construct will only succeed when passed in a instance of that particular model; if `strict` is true, it will only succeed if the constructed model has no extra fields. Both of these cases raise a `ValError::Omit`, which is caught in the parent smart union `construct` function. Both `strict` and `ultra_strict` are always set to `false` unless traversing through a smart validator. Perhaps somewhat inelegant, but whatever; I understand union code is getting an overhaul soon anyway.

`ConstructionState` also brings along the schema `definitions`, similar to `ValidationState`; this is so that we can construct a new `ValidationState` during construction at any point if construction absolutely *needs* to use `validate` to determine fitness. This is mainly used as a fallback in a few places in case I absolutely needed a ValidationState for a few operations, but ideally would probably be omitted in a final draft.

## Defaults:

Currently, defaults are respected when constructing (both `default` and `default_factory`), as per the current behavior of `BaseModel.model_construct()`:

```py
class Child(BaseModel):
    a: int
    b: int

class DefaultTest(BaseModel):
    a: Child = "something"
    b: Child = Field(default_factory=lambda: "something else")

m = DefaultTest.model_construct()
assert m.__dict__ == {}

m = DefaultTest.model_construct(_recursive=True)
assert m.a == "something"
assert m.b == "something else"
```

This PR also includes a new configuration parameter `construct_defaults` (foil to `validate_defaults`), which determines whether or not the default value should be constructed or not:

```py
class DefaultTest(BaseModel):
    a: Child = Field({"a": 10, "b": "wrong"}, construct_defaults=False)
    b: Child = Field({"a": 10, "b": "wrong"}, construct_defaults=True)
    # `default_factory` is also supported; construction is called after the factory
    c: Child = Field(default_factory=lambda: {"a": 10, "b": "generated"}, construct_defaults=False)
    d: Child = Field(default_factory=lambda: {"a": 10, "b": "generated"}, construct_defaults=True)

m = DefaultTest.model_construct(_recursive=True)
print(repr(m))
# m = DefaultTest(
#   a={"a": 10, "b": "wrong"},
#   b=Child(a=10, b="wrong"),
#   c={"a": 10, "b": "generated"},
#   d=Child(a=10, b="generated")
# )
```

`on_error='default'` arguments are also supported, though since construction will almost never error this parameter has almost no effect.

## Unions:

All 3 primary types of unions are implemented:

### Smart

Smart unions are probably overkill, but the current process is:

1. An ultra-strict CONSTRUCTION pass, looking for already made instances of the union. Defers to pre-made models before anything else.
2. An ultra-strict VALIDATION pass; if any succeeds, the first one found is returned. This is only done if `ultra_strict_required` is `true`.
3. A strict VALIDATION pass; this is done so we prefer a model that has its fields correctly set over a model that has its fields incorrectly set.
4. A strict CONSTRUCTION pass; returns the first model that constructs without extra fields.
5. A CONSTRUCTION pass; returns the first successful construction of anything.
6. Returns the input object unchanged.

### Left-to-right

Left-to-right unions are first checked for any exact instances; if any are found, that one is used. Failing that, the first one that constructs without error is returned, and if all members fail to be constructed the input object is returned unchanged.

### Tagged

Identical to `validate`'s implementation, except if there's any error with trying to read the discriminator, the process simply early-exits and returns the original object unchanged.

## `reconstruct_instances`, compliment to `revalidate_instances`:

Suppose a 3-level nested model scenario:

```py
class Item(BaseModel):
    a: int
    b: int

class User(BaseModel):
    item: Item

class Transaction(BaseModel):
    user: User

# Construct a user with `item` as an unconstructed dict form
user = User.model_construct(item={"a": "something", "b": "wrong"})
# Construct the transaction recursively
transaction = Transaction.model_construct(user=user, _recursive=True)
# What should transaction.user.item be?
# Should it be an instance of Item, or a dict?
```

By my estimation, there are cases where you would want both behaviors depending on the situation. Passing an existing instance to a construct method might be an indication that you want to keep the contents of that instance unchanged, wheras in other cases you might want to ensure that everything is maximally constructed regardless of the input data. Furthermore, you might want to customize this behavior on a submodel-to-submodel basis; some models should be maximally constructed, others left alone. This is already similar to the purpose of `revalidate_instances`, so this PR includes the proposed parameter:

```py
class Item(BaseModel, reconstruct_instances="never"):
    name: str

class ItemRecursive(BaseModel, reconstruct_instances="always"):
    name: str

class ItemParent(BaseModel, reconstruct_instances="subclass-instances"):
    name: str

class SubItem(ItemParent):
    location: str

class ProductCollection(BaseModel):
    a: Item
    b: ItemRecursive
    c: SubItem

class Server(BaseModel):
    pc: ProductCollection

pc = ProductCollection.model_construct(
    a={"name": "something"}
    b={"name": 123}
    c={"name": "something else", "location": "shelf 12"}
)

server = Server.model_construct(pc=pc, _recursive=True)
print(repr(server))
# pc = ProductCollection(
#   a={"name": "something"},
#   b=ItemRecursive(name=123),
#   c=ItemParent(name="something else")
# )
```

This way would allow more flexibility with construction and would tie it directly to the model specification.

## Construction Errors:

Constructing almost never generates errors, usually defaulting to returning the input object unchanged if there was an error interpreting it. In the cases where it does, all errors are returned as `ValidationErrors`(TODO: this is not the case!); I didn't see much use in creating a `ConstructionError` type, as there is minimal difference in function (both conceptually and implementation-wise). Constructing generates errors when:

1. Iterating over an input generator object that returns a `RuntimeError`
2. Calling a `default_factory` function that is either malformed or raises a `RuntimeError`
3. A discriminator cannot be found when determining the type of an element in a `TaggedUnion`
4. `DataclassArgs` provided with both positional and keyword arguments of the same value
5. recursively constructing custom implementations of abstract classes (`Sequence`, `Set`, `Mapping`, etc.) that don't implement an `__init__(iterable)` method

## Pydantic integration:

For implementing into Pydantic, the corresponding `model_construct` function would simply be rewritten to:

```py
    def model_construct(
        cls: type[Model], _fields_set: set[str] | None = None, _recursive: bool = False, **values: Any
    ) -> Model
        """
        ...
        """
        return cls.__pydantic_validator__.construct_python(
            values, fields_set=_fields_set, recursive=_recursive
        )
```

# Issues:

## Mutability and Storage

The most outstanding issue with this PR is that the existing definition for what `model_construct` *should* do is only well defined when no mutation of the input data is allowed. Going off the current behavior of `model_construct`, we can assert three axioms:

1. `output_model.field is input_dict["field"]`; and, by extension:
2. `output_model.field == input_dict["field"]`,
3. `type(output_model.field) == type(input_dict["field"])`.

This nice list goes out the window however as soon as you, say, convert `input_dict["field"]` from a dict value to a BaseModel instance; now none of the axioms are true, and you have a set of rather puzzling questions about the nature and form of the newly constructed data:

Where exactly do we put our newly coerced BaseModel? If the input object is mutable, then it might be feasible to modify the data in the original object to preserve the first axiom, maintaining current behavior. But this might very well be a bug from the user's perspective; would you expect `model_construct` to *modify* the dict it was passed in? Could be particularly insidious if that dict was being used elsewhere. And of course, if the input type is *immutable*, modifying in-place no longer becomes an option (or, at least it really shouldn't be).

So we're probably going to have to set the field to data unrelated to the input, violating at least the first axiom. But since we're now potentially copying *some* elements, we have a situation where part of the constructed model might point to the original passed-in data and parts might point to entirely new data. To me, this inconsistent ownership and non-rigorous definition sounds like a bug waiting to happen, but since we know that some data *might* be copied, as far as I can tell our only alternative is to have *all* the output's fields be copied.

### Generators, good and bad
When do you traverse a generator?
If we're copying, how do you copy a generator? (Protip: you cant)
Does it make sense for a generator to end up in a constructed model? Validated models traverse the generator during their traversal; do we want to do the same, or let the user consume the generator and just pass it along?
If we traverse a generator and construct it's values, what does the output type actually end up as? Does it end up as the annotated type? Breaks axiom 3!

Generators and iterators pose some interesting problems that I was unable to come up with an interface that didn't have a number of gotchas.
In the case of a "bad" iterator (e.g. one that raises a RuntimeError), should the user be able to construct a model with one of these iterators? In other words, should the iterator be traversed when constructing? Intuition says that NO, generators should not be traversed when constructing, *except for when `recursive=True`*:

```py
def badgen():
    raise RuntimeError

v = SchemaValidator(...) # List[Model]

v.construct_python(badgen()) # passes, because we don't need to traverse any of the iterators fields during surface construction; will instead fail when accessing attribute
v.construct_python(badgen(), recursive=True) # Fails, because we need to iterate over the generator and call `construct` on it's members
```

We can't copy generators, so we have 2 options:

1. Pass the generator object as-is
    Pros:
    - Simple and straightforward
    - Allows the user to traverse the generator on their own time and of their own volition
    Cons:
    - Makes generator objects non-coercable
2. Traverse the iterator and return it as some other type, likely the annotated type
    Pros:
    - Makes generator objects coercable
    Cons:
    - Violates axiom number 2 and 3, particularly on type annotations where you would expect recursion to not affect output ([see section below]())

### Concrete Types annotated as Abstract Base Classes


### `dict_view` objects

If you type `dict.keys()` or `dict.values()`, is that an assertion from the user that these fields *must* point to the original `dict`?
How do you construct a new one of these? If you pass this into a construction, what should it even return? The same dict.keys()? That is no longer the case since we're copying. A new anonymous dict object with the values changed? Do you want to ensure that the keys point to the original dict? Should it just coerce it to whatever type (list, tuple, dict) and return that? Then *none* of our previously defined axioms will be true!

This behavior seems most consistent with what `model_construct` is advertised as doing. However, there are some other complications when working with dict_view objects:
dict_view objects cannot be constructed wholesale (from what I can tell; and doing so would be something of a mistake, since they're only supposed to view data, not be it). Consider a field annotated as a List[Model] (where model is a BaseModel); if we were to coerce the input values into their models, we would have changed the input data; do we alter the data that the view points to that was passed in (potentially fishy), or do we construct a new dict object and then return a view that points to that object? (also potentially fishy)
Either way, it's not particularly clear what *should* happen.
We can sidestep this issue somewhat by copping-out and just coercing a dict_view to the annotated type, but this has it's own concerning problems:

```py
d = {"a": 1, "b": 2}
v = SchemaValidator(...) # Tuple[str]
v.construct_python(d.keys()) == d.keys() # Because we're not recursive, we can just pass as is; predictable and expected
v.construct_python(d.keys(), recursive=True) == ("a", "b") # Because we *might* change the data, we return a new object, this time as a tuple due to annotation
# But this now means that:
v.construct_python(d.keys()) != v.construct_python(d.keys(), recursive=True) #, even though you would expect them to be equal with this annotation!
# Plus, there's the users side of things. They passed in a .keys() view, which theoretically should point to the oringal dict 'd'; the tuple no-longer does that
```

We can also always treat dict_views as their annotated type, but this means that we have to construct a new object regardless of `recursive`, which means traversing the iterable no matter what... which now makes our previous excercise with `badgen()` impossible to replicate (simply, at least)

Ensuring the third axiom is also particularly difficult.

### Above edge cases can lead to overly dependent behavior

Finally, there is one more matter to consider, and it is the effect of the recursive flag on the output model, and how it affects the specified axioms. Different outcomes are possible between recursive and non-recursive construct; this is intuitive for nested models, but not intuitive for non-nested annotations in certain circumstances. Illustration:

```py
v = SchemaValidator(...) # Tuple[str]
# You would expect the recursive flag to not affect this annotation
# And indeed, for most inputs, this is absolutely the case
t = ("works", "fine")
assert v.construct_python(t) == v.construct_python(t, recursive=True)
s = "no problem"
assert v.construct_python(s) == v.construct_python(s, recursive=True)

# But according to the dict_view behavior specified above:
d = {"something": "horrible"}
res_non = v.construct_python(d.keys(), recursive=False) # d.keys()
res_rec = v.construct_python(d.keys(), recursive=True) # ("something",)
# Now recursive *does* matter with this annotation!
assert res_non != res_rec
```

So, in other words, because of the many edge cases defined above, output value is not only dependent on the annotation and the recursive flag, but also the input value, which is very not ideal.

To summarize, the current functionality of `model_construct` is not sufficient to fully deduce a recursive implementation. Thus, a new set of axioms for `model_construct` should be discussed, defined, and abided by:

* Does `model_construct` pass existing data into the newly constructed object, or does it copy the data into the output object? If it passes the data along, how should it handle immutable values whose data needs to be converted into something else when `recursive=True`?
* Should `model_construct` assert that the type of a set field will match the type it was passed in as? If so, what to do about dict_views? Is this a worthwhile goal to attempt, or are we better off just coercing the input value to it's annotated type?
* Should constructing models recursively generate different outputs than constructing models normally for non-recursive annotations? This is obviously true for nested model annotations, but what about simple ones like `Tuple[str]`? Does it make sense for constructions of this annotation to return different values based on whether recursive was specified? Intuition says no, but strict logic says yes.
* Should `model_construct` consume generators? If so, when? And is the consumption dependent on the recursive flag?

## Output Type

The task of actually coercing new data into an object of the same type is also not as trivial as one would hope; this is particularly hard when given immutable objects, `dict_views`, or any sort of Abstract Base subclass of `Sequence` or similar. For regular types such as `list`, `set`, `dict`, our desired behavior is well defined; we can always expect to create a new object of these types with it's contents correctly set. Not every input falls into these categories, however:

### Generators and `dict_view` objects

Intuition suggests that generators should only be traversed (and therefore consumed) when `recursive=True`, and this is how they are implemented currently:

```py
v = SchemaValidator(...) # List[int]

def generator():
    yield 1
    yield 2
    yield 3

gen_obj = generator()
assert v.construct_python(gen_obj, recursive=False) is gen_obj

test_dict = {1: "some", 2: "string"}
keys_obj = test_dict.keys()
assert v.construct_python(keys_obj, recursive=False) is keys_obj
# Same thing with values() and items()

# Because generators are only traversed when recursive is true, generators only raise errors if recursive is true:

def generator():
    raise RuntimeError

gen = generator()

assert v.construct_python(gen) is gen # Fine; wait until the user traverses it
try:
    v.construct_python(gen, recursive=True)
except RuntimeError:
    print("Fails because we have to traverse it ourselves")
```

However, if recursive *is* true, what should the output types of these values be? Creating a copy of a generator is ill defined in python; and a "correct" way to handle dict views would be to construct a new dictionary object and return the dict views that point to that; but how do you determine the keys if you were only passed the values?

Currently, this PR sidesteps this by coercing these "special" types to the specified annotated type:

```py
v = SchemaValidator(...) # List[int]

assert v.construct_python(gen_obj, recursive=True) == [1, 2, 3]
assert v.construct_python(test_dict.keys(), recursive=True) == [1, 2]
assert v.construct_python(test_dict.values(), recursive=True) == ["some", "string"]
```

Doing this breaks axiom 3, but this is the most meaningful compromise that I could come up with that preserves `model_construct`'s original behavior, adds the recursion logic I'm looking for, and doesn't do anything "too bizarre". The only real downside to this system is that now recursive can sometimes produce different outcomes depending on recursive for annotations that are not recursive; or in other words:

`v.construct_python(some_input, recursive=False)` may or may not be equal to `v.construct_python(some_input, recursive=True)`

where `v` is a non-recursive annotation like `List[str]`. I think this is tolerable, as long as this behavior is well documented.

### Custom implementations of Abstract Base Classes (`MySequence`, `MyMapping`, `MySet`, etc.)

For abstract objects we can't rely on just returning it as the annotated type; it would probably be very annoying to a user to construct a model by passing in a `CustomList` object and have it be constructed to a regular one. But abstract classes provide very little in the way of concrete implementation - actually constructing a new instance of `CustomList` in all cases is impossible to know from Pydantic's perspective.

Mutable objects have a number of contracted methods (`append`, `add`, `__setitem__`) which we can use to guarantee that we can make new instances of these objects; but such contracts are not provided for the immutable types. As a result, the only valid way forward that I can see is to establish some form of "expected form" or callback that the input subclass must mimic in order for `construct()` to interpret it properly.

Specifically, the current implementation currently asserts that the custom class must implement an `__init__()` method, and that method must take at least a single positional argument which is an iterator of new values to assign to the custom class. This is identical to the signatures that builtins have, which is by intention; this logic applies to them as well:

```py
class MySequence(collections.abc.Sequence):
    def __init__(self, data: Iterable):
        self._data = [elem for elem in data]

    def __getitem__(self, index: int) -> Any:
        return self._data[index]

    def __len__(self):
        return len(self._data)

    def __repr__(self) -> str:
        return f'MySequence({repr(self._data)})'
    
    def __eq__(self, other):
        if isinstance(other, MySequence):
            return self._data == other._data
        return False

v = SchemaValidator(...) # List[int]

# The result is simply `type(input_obect).__init__(output_value_iterator)`, which works for all default types
assert v.construct_python([1, 2, 3]) == [1, 2, 3]
assert v.construct_python((1, 2, 3)) == (1, 2, 3)
assert v.construct_python({1, 2, 3}) == {1, 2, 3}
# Because __init__ is correct for MySequence, this also now works as expected
# If __init__ was malformed or missing, this would create an error
assert v.construct_python(MySequence([1, 2, 3])) == MySequence([1, 2, 3])
# Doing things this way also allows us to distinguish between deque and list, which I found very difficult to do otherwise
assert v.construct_python(deque([1, 2, 3])) == deque([1, 2, 3])
```

Of course, using `__init__()` might be too important a callback for something like this; perhaps it should search for a `__pydantic_construct_init__()` method or similar.

---

Another thing to consider is that should annotations match in order to actually recurse through an object and construct submodels? Consider another example:

```py
class SetOfChildren(RootModel):
    root: Set[Child]

v = SetOfChildren.__pydantic_validator__

# Suppose we pass a list of valid child instances
m = v.construct_python([{"a": 10, "b": "wrong"}], recursive=True)
# Should: 
# m.root == [Child(10, "wrong")], becase the iterable could be traversed, or
# m.root == [{"a": 10, "b": "wrong"}], because a list is not a set
```

I think I'm leaning towards the second option, if only because it makes implementation simpler. Instead of having to coerce any type into any-other type, we now only have to coerce a known type into any other type.

## Generators and Iterators

Generators are traversed 

## Discriminated Unions

Regular literals and self-schema discriminators work as you expect.
Functional discriminators dont do any coercion, as I believe this would be more explicitly handled in the function itself if a conversion is desired. (I also am a little unsure of the exact purpose of functional discriminators, and neither the existing documentation nor the current test suite is super descriptive of their use-case.)

Note that discriminated unions might run into sore spots in select circumstances:

```py
class Child(BaseModel):
    a: int
    b: int

class D1(BaseModel):
    elem: Child

class D2(BaseModel):
    elem: dict

class TestDiscriminators(BaseModel):
    test: Union[D1, D2] = Field(..., discriminator="elem")

v = TestDiscriminators.__pydantic_validator__

result = v.construct_python({"test": {"a": 1, "b": 2}}, recursive=True)
# Should `result.test` be a dict or an instance of Child?
```

This question becomes more difficult to answer the more the input dict diverges from the class instance. In fact, passing a dict with *any* set of keys can be constructed into a `Child`, which is likely not what the user wants if they specify a union like above. On the other hand, if someone has a tagged union where the discriminator can be one of many models, you might very well want to pass in a dict and have it get coerced to the closest Model type available.

## Other Remarks:

* With the introduction of `construct_python()`, it's trivial to add an equivalent `construct_json()` so people can construct models directly from trusted JSON data; an example implementation is included in this PR, though not rigorously tested.
* Some structures like `Tuple` and `List` have their item validators as optionals, while others like `Dict` and `Set` get set with an `Any` validator by default. Is there a particular reason for this inconsistency? From an outsider looking in, this felt odd.
* TypedDicts with custom paths and aliases are respected as long as they are able to be resolved; if a path or alias cannot be found, resolved, or otherwise raises a `ValidationError`, the input object remains unchanged.
* Extra fields are passed along no matter what `ExtraBehavior` is set to. TODO: difference between Models and Dataclasses?

Included:
static Defaults, custom init, post init

Excluded:
Validators of all types, default factories, callbacks/functions that run on validation errors that are not triggered when constructing


When given a dict to an field annotated with a model
    If the dict has extra attributes that are not captured by the model, the model should still have these extra fields even if the model itself doens't specify them
    This should be carried over to dataclasses, if possible

