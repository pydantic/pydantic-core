import pickle

import pytest

from pydantic_core import MultiHostUrl, SchemaSerializer, SchemaValidator, Url, core_schema


def test_url():
    v = SchemaValidator(core_schema.url_schema())
    s = SchemaSerializer(core_schema.url_schema())

    url = v.validate_python('https://example.com')
    assert isinstance(url, Url)
    assert str(url) == 'https://example.com/'
    assert url.host == 'example.com'

    assert s.to_python(url) == url
    assert s.to_python(url, mode='json') == 'https://example.com/'
    assert s.to_json(url) == b'"https://example.com/"'

    with pytest.warns(UserWarning, match='Expected `url` but got `str` - serialized value may not be as expected'):
        assert s.to_python('https://example.com', mode='json') == 'https://example.com'


@pytest.mark.parametrize(
    'url_string,omit_trailing_slash,expected,expected_unicode,path',
    (
        ('https://ex.com', True, 'https://ex.com', 'https://ex.com', None),
        ('https://ex.com/', True, 'https://ex.com', 'https://ex.com', None),
        ('https://ex.com/', True, 'https://ex.com', 'https://ex.com', None),
        ('https://ex.com/api', True, 'https://ex.com/api', 'https://ex.com/api', '/api'),
        ('https://ex.com/api/', True, 'https://ex.com/api/', 'https://ex.com/api/', '/api/'),
        ('ftp://user:pass@localhost:4242?hello=world', True, 'ftp://user:pass@localhost:4242?hello=world', 'ftp://user:pass@localhost:4242?hello=world', None),
        ('ftp://user:pass@localhost:4242/?hello=world', True, 'ftp://user:pass@localhost:4242?hello=world', 'ftp://user:pass@localhost:4242?hello=world', None),
        ('ftp://user:pass@localhost:4242/path?hello=world', True, 'ftp://user:pass@localhost:4242/path?hello=world', 'ftp://user:pass@localhost:4242/path?hello=world', '/path'),
        ('https://ex.com?query=123#fragment', True, 'https://ex.com?query=123#fragment', 'https://ex.com?query=123#fragment', None),
        ('https://ex.com/?query=123#fragment', True, 'https://ex.com?query=123#fragment', 'https://ex.com?query=123#fragment', None),
        ('https://ex.com/path?query=123#fragment', True, 'https://ex.com/path?query=123#fragment', 'https://ex.com/path?query=123#fragment', '/path'),
        ('https://ex.com/path/?query=123#fragment', True, 'https://ex.com/path/?query=123#fragment', 'https://ex.com/path/?query=123#fragment', '/path/'),
        ('https://user:pass@ex.com/?query=123#fragment', True, 'https://user:pass@ex.com?query=123#fragment', 'https://user:pass@ex.com?query=123#fragment', None),
        ('https://user:pass@ex.com//?query=123#fragment', True, 'https://user:pass@ex.com//?query=123#fragment', 'https://user:pass@ex.com//?query=123#fragment', '//'),
        ('https://user:pass@привет-мир.ßß:443', True, 'https://user:pass@xn----ctbjkdxqigq.xn--zcaa', 'https://user:pass@привет-мир.ßß', None),
        ('https://user:pass@привет-мир.ßß:442', True, 'https://user:pass@xn----ctbjkdxqigq.xn--zcaa:442', 'https://user:pass@привет-мир.ßß:442', None),
        ('https://привет-мир.ßß', True, 'https://xn----ctbjkdxqigq.xn--zcaa', 'https://привет-мир.ßß', None),
        ('https://привет-мир.ßß/', True, 'https://xn----ctbjkdxqigq.xn--zcaa', 'https://привет-мир.ßß', None),
        ('https://привет-мир.ßß/api', True, 'https://xn----ctbjkdxqigq.xn--zcaa/api', 'https://привет-мир.ßß/api', '/api'),
        ('https://привет-мир.ßß/api/', True, 'https://xn----ctbjkdxqigq.xn--zcaa/api/', 'https://привет-мир.ßß/api/', '/api/'),
        ('https://user:pass@привет-мир😀.ßß/api/?query=123', True, 'https://user:pass@xn----ctbjkdxqigq06721l.xn--zcaa/api/?query=123', 'https://user:pass@привет-мир😀.ßß/api/?query=123', '/api/'),
        ('https://user:pass@привет-мир😀.ßß/?query=123', True, 'https://user:pass@xn----ctbjkdxqigq06721l.xn--zcaa?query=123', 'https://user:pass@привет-мир😀.ßß?query=123', None),

        ('https://ex.com', False, 'https://ex.com/', 'https://ex.com/', '/'),
        ('https://ex.com/', False, 'https://ex.com/', 'https://ex.com/', '/'),
        ('https://ex.com/', False, 'https://ex.com/', 'https://ex.com/', '/'),
        ('https://ex.com/api', False, 'https://ex.com/api', 'https://ex.com/api', '/api'),
        ('https://ex.com/api/', False, 'https://ex.com/api/', 'https://ex.com/api/', '/api/'),
        ('ftp://user:pass@localhost:4242?hello=world', False, 'ftp://user:pass@localhost:4242/?hello=world', 'ftp://user:pass@localhost:4242/?hello=world', '/'),
        ('ftp://user:pass@localhost:4242/?hello=world', False, 'ftp://user:pass@localhost:4242/?hello=world', 'ftp://user:pass@localhost:4242/?hello=world', '/'),
        ('ftp://user:pass@localhost:4242/path?hello=world', False, 'ftp://user:pass@localhost:4242/path?hello=world', 'ftp://user:pass@localhost:4242/path?hello=world', '/path'),
        ('https://ex.com?query=123#fragment', False, 'https://ex.com/?query=123#fragment', 'https://ex.com/?query=123#fragment', '/'),
        ('https://ex.com/?query=123#fragment', False, 'https://ex.com/?query=123#fragment', 'https://ex.com/?query=123#fragment', '/'),
        ('https://ex.com/path?query=123#fragment', False, 'https://ex.com/path?query=123#fragment', 'https://ex.com/path?query=123#fragment', '/path'),
        ('https://ex.com/path/?query=123#fragment', False, 'https://ex.com/path/?query=123#fragment', 'https://ex.com/path/?query=123#fragment', '/path/'),
        ('https://user:pass@ex.com/?query=123#fragment', False, 'https://user:pass@ex.com/?query=123#fragment', 'https://user:pass@ex.com/?query=123#fragment', '/'),
        ('https://user:pass@ex.com//?query=123#fragment', False, 'https://user:pass@ex.com//?query=123#fragment', 'https://user:pass@ex.com//?query=123#fragment', '//'),
        ('https://user:pass@привет-мир.ßß:443', False, 'https://user:pass@xn----ctbjkdxqigq.xn--zcaa/', 'https://user:pass@привет-мир.ßß/', '/'),
        ('https://user:pass@привет-мир.ßß:442', False, 'https://user:pass@xn----ctbjkdxqigq.xn--zcaa:442/', 'https://user:pass@привет-мир.ßß:442/', '/'),
        ('https://привет-мир.ßß', False, 'https://xn----ctbjkdxqigq.xn--zcaa/', 'https://привет-мир.ßß/', '/'),
        ('https://привет-мир.ßß/', False, 'https://xn----ctbjkdxqigq.xn--zcaa/', 'https://привет-мир.ßß/', '/'),
        ('https://привет-мир.ßß/api', False, 'https://xn----ctbjkdxqigq.xn--zcaa/api', 'https://привет-мир.ßß/api', '/api'),
        ('https://привет-мир.ßß/api/', False, 'https://xn----ctbjkdxqigq.xn--zcaa/api/', 'https://привет-мир.ßß/api/', '/api/'),
        ('https://user:pass@привет-мир😀.ßß/api/?query=123', False, 'https://user:pass@xn----ctbjkdxqigq06721l.xn--zcaa/api/?query=123', 'https://user:pass@привет-мир😀.ßß/api/?query=123', '/api/'),
        ('https://user:pass@привет-мир😀.ßß/?query=123', False, 'https://user:pass@xn----ctbjkdxqigq06721l.xn--zcaa/?query=123', 'https://user:pass@привет-мир😀.ßß/?query=123', '/'),

        ('https://ex.com', None, 'https://ex.com/', 'https://ex.com/', '/'),
        ('https://ex.com/', None, 'https://ex.com/', 'https://ex.com/', '/'),
        ('https://ex.com/', None, 'https://ex.com/', 'https://ex.com/', '/'),
        ('https://ex.com/api', None, 'https://ex.com/api', 'https://ex.com/api', '/api'),
        ('https://ex.com/api/', None, 'https://ex.com/api/', 'https://ex.com/api/', '/api/'),
        ('ftp://user:pass@localhost:4242?hello=world', None, 'ftp://user:pass@localhost:4242/?hello=world', 'ftp://user:pass@localhost:4242/?hello=world', '/'),
        ('ftp://user:pass@localhost:4242/?hello=world', None, 'ftp://user:pass@localhost:4242/?hello=world', 'ftp://user:pass@localhost:4242/?hello=world', '/'),
        ('ftp://user:pass@localhost:4242/path?hello=world', None, 'ftp://user:pass@localhost:4242/path?hello=world', 'ftp://user:pass@localhost:4242/path?hello=world', '/path'),
        ('https://ex.com?query=123#fragment', None, 'https://ex.com/?query=123#fragment', 'https://ex.com/?query=123#fragment', '/'),
        ('https://ex.com/?query=123#fragment', None, 'https://ex.com/?query=123#fragment', 'https://ex.com/?query=123#fragment', '/'),
        ('https://ex.com/path?query=123#fragment', None, 'https://ex.com/path?query=123#fragment', 'https://ex.com/path?query=123#fragment', '/path'),
        ('https://ex.com/path/?query=123#fragment', None, 'https://ex.com/path/?query=123#fragment', 'https://ex.com/path/?query=123#fragment', '/path/'),
        ('https://user:pass@ex.com/?query=123#fragment', None, 'https://user:pass@ex.com/?query=123#fragment', 'https://user:pass@ex.com/?query=123#fragment', '/'),
        ('https://user:pass@ex.com//?query=123#fragment', None, 'https://user:pass@ex.com//?query=123#fragment', 'https://user:pass@ex.com//?query=123#fragment', '//'),
        ('https://user:pass@привет-мир.ßß:443', None, 'https://user:pass@xn----ctbjkdxqigq.xn--zcaa/', 'https://user:pass@привет-мир.ßß/', '/'),
        ('https://user:pass@привет-мир.ßß:442', None, 'https://user:pass@xn----ctbjkdxqigq.xn--zcaa:442/', 'https://user:pass@привет-мир.ßß:442/', '/'),
        ('https://привет-мир.ßß', None, 'https://xn----ctbjkdxqigq.xn--zcaa/', 'https://привет-мир.ßß/', '/'),
        ('https://привет-мир.ßß/', None, 'https://xn----ctbjkdxqigq.xn--zcaa/', 'https://привет-мир.ßß/', '/'),
        ('https://привет-мир.ßß/api', None, 'https://xn----ctbjkdxqigq.xn--zcaa/api', 'https://привет-мир.ßß/api', '/api'),
        ('https://привет-мир.ßß/api/', None, 'https://xn----ctbjkdxqigq.xn--zcaa/api/', 'https://привет-мир.ßß/api/', '/api/'),
        ('https://user:pass@привет-мир😀.ßß/api/?query=123', None, 'https://user:pass@xn----ctbjkdxqigq06721l.xn--zcaa/api/?query=123', 'https://user:pass@привет-мир😀.ßß/api/?query=123', '/api/'),
        ('https://user:pass@привет-мир😀.ßß/?query=123', None, 'https://user:pass@xn----ctbjkdxqigq06721l.xn--zcaa/?query=123', 'https://user:pass@привет-мир😀.ßß/?query=123', '/'),
    ),
)  # fmt: skip
def test_url_with_omit_trailing_slash(
    url_string,
    omit_trailing_slash,
    expected,
    expected_unicode,
    path,
) -> None:
    v = SchemaValidator(core_schema.url_schema(omit_trailing_slash=omit_trailing_slash))
    s = SchemaSerializer(core_schema.url_schema(omit_trailing_slash=omit_trailing_slash))

    url = v.validate_python(url_string)
    assert str(url) == expected
    assert repr(url) == f"Url('{expected}')"
    assert url.unicode_string() == expected_unicode
    assert url.path == path

    assert s.to_python(url) == url
    assert s.to_python(url, mode='json') == expected
    assert s.to_json(url) == f'"{expected}"'.encode()


def test_multiple_urls_with_omit_trailing_slash_feature() -> None:
    url = Url('https://example.com')

    assert str(url) == 'https://example.com/'
    assert repr(url) == "Url('https://example.com/')"
    assert url.unicode_string() == 'https://example.com/'

    url = Url('https://example.org', omit_trailing_slash=True)

    assert str(url) == 'https://example.org'
    assert repr(url) == "Url('https://example.org')"
    assert url.unicode_string() == 'https://example.org'

    url = MultiHostUrl('https://ex.com,ex.org')

    assert str(url) == 'https://ex.com,ex.org/'
    assert repr(url) == "MultiHostUrl('https://ex.com,ex.org/')"
    assert url.unicode_string() == 'https://ex.com,ex.org/'

    url = MultiHostUrl('https://ex.com,ex.org', omit_trailing_slash=True)

    assert str(url) == 'https://ex.com,ex.org'
    assert repr(url) == "MultiHostUrl('https://ex.com,ex.org')"
    assert url.unicode_string() == 'https://ex.com,ex.org'


def test_multi_host_url():
    v = SchemaValidator(core_schema.multi_host_url_schema())
    s = SchemaSerializer(core_schema.multi_host_url_schema())

    url = v.validate_python('https://example.com,example.org/path')
    assert isinstance(url, MultiHostUrl)
    assert str(url) == 'https://example.com,example.org/path'
    assert [h['host'] for h in url.hosts()] == ['example.com', 'example.org']

    assert s.to_python(url) == url
    assert s.to_python(url, mode='json') == 'https://example.com,example.org/path'
    assert s.to_json(url) == b'"https://example.com,example.org/path"'

    with pytest.warns(
        UserWarning, match='Expected `multi-host-url` but got `str` - serialized value may not be as expected'
    ):
        assert s.to_python('https://ex.com,ex.org/path', mode='json') == 'https://ex.com,ex.org/path'


@pytest.mark.parametrize(
    'url_string,omit_trailing_slash,expected,expected_unicode,path',
    (
        ('https://ex.com', True, 'https://ex.com', 'https://ex.com', None),
        ('https://ex.com/', True, 'https://ex.com', 'https://ex.com', None),
        ('https://ex.com,ex.org', True, 'https://ex.com,ex.org', 'https://ex.com,ex.org', None),
        ('https://ex.com,ex.org/', True, 'https://ex.com,ex.org', 'https://ex.com,ex.org', None),
        ('https://ex.com,ex.org//', True, 'https://ex.com,ex.org//', 'https://ex.com,ex.org//', '//'),
        ('https://ex.com,ex.org/path', True, 'https://ex.com,ex.org/path', 'https://ex.com,ex.org/path', '/path'),
        ('https://ex.com,ex.org/path/', True, 'https://ex.com,ex.org/path/', 'https://ex.com,ex.org/path/', '/path/'),
        ('https://user:pass@ex.com,ex.org:4242', True, 'https://user:pass@ex.com,ex.org:4242', 'https://user:pass@ex.com,ex.org:4242', None),
        (
            'https://user:pass@привет-мир😀.ßß,same:pass@привет.org:4242/',
            True,
            'https://user:pass@xn----ctbjkdxqigq06721l.xn--zcaa,same:pass@xn--b1agh1afp.org:4242',
            'https://user:pass@привет-мир😀.ßß,same:pass@привет.org:4242',
            None,
        ),
        (
            'https://user:pass@привет-мир😀.ßß,same:pass@привет.org:4242/api',
            True,
            'https://user:pass@xn----ctbjkdxqigq06721l.xn--zcaa,same:pass@xn--b1agh1afp.org:4242/api',
            'https://user:pass@привет-мир😀.ßß,same:pass@привет.org:4242/api',
            '/api',
        ),

        ('https://ex.com', False, 'https://ex.com/', 'https://ex.com/', '/'),
        ('https://ex.com/', False, 'https://ex.com/', 'https://ex.com/', '/'),
        ('https://ex.com,ex.org', False, 'https://ex.com,ex.org/', 'https://ex.com,ex.org/', '/'),
        ('https://ex.com,ex.org/', False, 'https://ex.com,ex.org/', 'https://ex.com,ex.org/', '/'),
        ('https://ex.com,ex.org//', False, 'https://ex.com,ex.org//', 'https://ex.com,ex.org//', '//'),
        ('https://ex.com,ex.org/path', False, 'https://ex.com,ex.org/path', 'https://ex.com,ex.org/path', '/path'),
        ('https://ex.com,ex.org/path/', False, 'https://ex.com,ex.org/path/', 'https://ex.com,ex.org/path/', '/path/'),
        ('https://user:pass@ex.com,ex.org:4242', False, 'https://user:pass@ex.com,ex.org:4242/', 'https://user:pass@ex.com,ex.org:4242/', '/'),
        (
            'https://user:pass@привет-мир😀.ßß,same:pass@привет.org:4242/',
            False,
            'https://user:pass@xn----ctbjkdxqigq06721l.xn--zcaa,same:pass@xn--b1agh1afp.org:4242/',
            'https://user:pass@привет-мир😀.ßß,same:pass@привет.org:4242/',
            '/',
        ),
        (
            'https://user:pass@привет-мир😀.ßß,same:pass@привет.org:4242/api',
            False,
            'https://user:pass@xn----ctbjkdxqigq06721l.xn--zcaa,same:pass@xn--b1agh1afp.org:4242/api',
            'https://user:pass@привет-мир😀.ßß,same:pass@привет.org:4242/api',
            '/api',
        ),

        ('https://ex.com', None, 'https://ex.com/', 'https://ex.com/', '/'),
        ('https://ex.com/', None, 'https://ex.com/', 'https://ex.com/', '/'),
        ('https://ex.com,ex.org', None, 'https://ex.com,ex.org/', 'https://ex.com,ex.org/', '/'),
        ('https://ex.com,ex.org/', None, 'https://ex.com,ex.org/', 'https://ex.com,ex.org/', '/'),
        ('https://ex.com,ex.org//', None, 'https://ex.com,ex.org//', 'https://ex.com,ex.org//', '//'),
        ('https://ex.com,ex.org/path', None, 'https://ex.com,ex.org/path', 'https://ex.com,ex.org/path', '/path'),
        ('https://ex.com,ex.org/path/', None, 'https://ex.com,ex.org/path/', 'https://ex.com,ex.org/path/', '/path/'),
        ('https://user:pass@ex.com,ex.org:4242', None, 'https://user:pass@ex.com,ex.org:4242/', 'https://user:pass@ex.com,ex.org:4242/', '/'),
        (
            'https://user:pass@привет-мир😀.ßß,same:pass@привет.org:4242/',
            None,
            'https://user:pass@xn----ctbjkdxqigq06721l.xn--zcaa,same:pass@xn--b1agh1afp.org:4242/',
            'https://user:pass@привет-мир😀.ßß,same:pass@привет.org:4242/',
            '/',
        ),
        (
            'https://user:pass@привет-мир😀.ßß,same:pass@привет.org:4242/api',
            None,
            'https://user:pass@xn----ctbjkdxqigq06721l.xn--zcaa,same:pass@xn--b1agh1afp.org:4242/api',
            'https://user:pass@привет-мир😀.ßß,same:pass@привет.org:4242/api',
            '/api',
        ),
    ),
)  # fmt: skip
def test_multi_host_url_with_omit_trailing_slash(
    url_string,
    omit_trailing_slash,
    expected,
    expected_unicode,
    path,
) -> None:
    v = SchemaValidator(core_schema.multi_host_url_schema(omit_trailing_slash=omit_trailing_slash))
    s = SchemaSerializer(core_schema.multi_host_url_schema(omit_trailing_slash=omit_trailing_slash))

    url = v.validate_python(url_string)
    assert isinstance(url, MultiHostUrl)
    assert str(url) == expected
    assert repr(url) == f"MultiHostUrl('{expected}')"
    assert url.path == path
    assert url.unicode_string() == expected_unicode

    assert s.to_python(url) == url
    assert s.to_python(url, mode='json') == expected
    assert s.to_json(url) == f'"{expected}"'.encode()


def test_url_dict_keys():
    v = SchemaValidator(core_schema.url_schema())

    s = SchemaSerializer(core_schema.dict_schema(core_schema.url_schema()))
    url = v.validate_python('https://example.com')
    assert s.to_python({url: 'foo'}) == {url: 'foo'}
    assert s.to_python({url: 'foo'}, mode='json') == {'https://example.com/': 'foo'}
    assert s.to_json({url: 'foo'}) == b'{"https://example.com/":"foo"}'


def test_multi_host_url_dict_keys():
    v = SchemaValidator(core_schema.multi_host_url_schema())

    s = SchemaSerializer(core_schema.dict_schema(core_schema.multi_host_url_schema()))
    url = v.validate_python('https://example.com,example.org/path')
    assert s.to_python({url: 'foo'}) == {url: 'foo'}
    assert s.to_python({url: 'foo'}, mode='json') == {'https://example.com,example.org/path': 'foo'}
    assert s.to_json({url: 'foo'}) == b'{"https://example.com,example.org/path":"foo"}'


def test_any():
    url = Url('https://ex.com')
    multi_host_url = MultiHostUrl('https://ex.com,ex.org/path')

    s = SchemaSerializer(core_schema.any_schema())
    assert s.to_python(url) == url
    assert type(s.to_python(url)) == Url
    assert s.to_python(multi_host_url) == multi_host_url
    assert type(s.to_python(multi_host_url)) == MultiHostUrl
    assert s.to_python(url, mode='json') == 'https://ex.com/'
    assert s.to_python(multi_host_url, mode='json') == 'https://ex.com,ex.org/path'
    assert s.to_json(url) == b'"https://ex.com/"'
    assert s.to_json(multi_host_url) == b'"https://ex.com,ex.org/path"'

    assert s.to_python({url: 1, multi_host_url: 2}) == {url: 1, multi_host_url: 2}
    assert s.to_python({url: 1, multi_host_url: 2}, mode='json') == {
        'https://ex.com/': 1,
        'https://ex.com,ex.org/path': 2,
    }
    assert s.to_json({url: 1, multi_host_url: 2}) == b'{"https://ex.com/":1,"https://ex.com,ex.org/path":2}'


def test_custom_serializer():
    s = SchemaSerializer(core_schema.any_schema(serialization=core_schema.simple_ser_schema('multi-host-url')))

    multi_host_url = MultiHostUrl('https://ex.com,ex.org/path')
    assert s.to_python(multi_host_url) == multi_host_url


@pytest.mark.parametrize('base_class', [Url, MultiHostUrl])
def test_url_subclass(base_class):
    class MyUrl(base_class):
        def some_method(self):
            return self.path + '-success'

    m = MyUrl('http://ex.com/path')
    assert m.some_method() == '/path-success'


@pytest.mark.parametrize('value', (Url('https://example.com'), MultiHostUrl('https://example.com,example.org/path')))
def test_url_pickle(value):
    pickled = pickle.dumps(value)
    unpickled = pickle.loads(pickled)
    assert value == unpickled
