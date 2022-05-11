window.BENCHMARK_DATA = {
  "lastUpdate": 1652261878739,
  "repoUrl": "https://github.com/samuelcolvin/pydantic-core",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "s@muelcolvin.com",
            "name": "Samuel Colvin",
            "username": "samuelcolvin"
          },
          "committer": {
            "email": "s@muelcolvin.com",
            "name": "Samuel Colvin",
            "username": "samuelcolvin"
          },
          "distinct": true,
          "id": "6b430f9de18cfd75141ee518d25fba625137b5dd",
          "message": "temporarily remove paths restriction on benchmarks",
          "timestamp": "2022-05-11T10:32:59+01:00",
          "tree_id": "0ef483164d7dc40843bccf3a93b4297be801364a",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/6b430f9de18cfd75141ee518d25fba625137b5dd"
        },
        "date": 1652261877752,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_pyd_python",
            "value": 1979.606523491039,
            "unit": "iter/sec",
            "range": "stddev: 0.00006901236962328351",
            "extra": "mean: 505.15089141881515 usec\nrounds: 2284"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 25350.024953455166,
            "unit": "iter/sec",
            "range": "stddev: 0.001305452273109351",
            "extra": "mean: 39.447692924803285 usec\nrounds: 54345"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_pyd_json",
            "value": 1805.4185434021902,
            "unit": "iter/sec",
            "range": "stddev: 0.00008641372873082166",
            "extra": "mean: 553.888184905627 usec\nrounds: 2120"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 18016.71413453945,
            "unit": "iter/sec",
            "range": "stddev: 0.0019807224477834374",
            "extra": "mean: 55.504016577746654 usec\nrounds: 32574"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_pyd",
            "value": 24705.105046661993,
            "unit": "iter/sec",
            "range": "stddev: 0.00002050417278558171",
            "extra": "mean: 40.47746399423281 usec\nrounds: 29412"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 215947.6208061508,
            "unit": "iter/sec",
            "range": "stddev: 0.00001279990227496455",
            "extra": "mean: 4.630752569845017 usec\nrounds: 129871"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_pyd",
            "value": 151285.02083082538,
            "unit": "iter/sec",
            "range": "stddev: 0.000010835464567342088",
            "extra": "mean: 6.610039741596434 usec\nrounds: 178604"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 940784.8753177086,
            "unit": "iter/sec",
            "range": "stddev: 0.0000010477713747522003",
            "extra": "mean: 1.0629422583591523 usec\nrounds: 99010"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 170268.86398776842,
            "unit": "iter/sec",
            "range": "stddev: 0.0004811225320834584",
            "extra": "mean: 5.8730643793559745 usec\nrounds: 99010"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_pyd",
            "value": 845.8619314426836,
            "unit": "iter/sec",
            "range": "stddev: 0.01947934329347436",
            "extra": "mean: 1.182226038112889 msec\nrounds: 1653"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 1676.6594747657155,
            "unit": "iter/sec",
            "range": "stddev: 0.027675612373301333",
            "extra": "mean: 596.424029476667 usec\nrounds: 9058"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_pyd",
            "value": 1583.2623397780737,
            "unit": "iter/sec",
            "range": "stddev: 0.00007722638512861943",
            "extra": "mean: 631.6072673971202 usec\nrounds: 1825"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 17358.965045785382,
            "unit": "iter/sec",
            "range": "stddev: 0.000029691219797545142",
            "extra": "mean: 57.607121009947086 usec\nrounds: 20081"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_pyd_py",
            "value": 290.2400398658547,
            "unit": "iter/sec",
            "range": "stddev: 0.00041180233316118734",
            "extra": "mean: 3.4454240030499834 msec\nrounds: 328"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3120.596545716897,
            "unit": "iter/sec",
            "range": "stddev: 0.0001095284458401033",
            "extra": "mean: 320.45155000012 usec\nrounds: 3540"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_pyd_json",
            "value": 277.47792011962036,
            "unit": "iter/sec",
            "range": "stddev: 0.0002868194516754046",
            "extra": "mean: 3.6038903548394097 msec\nrounds: 310"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 5910.097709442715,
            "unit": "iter/sec",
            "range": "stddev: 0.00007716428180793458",
            "extra": "mean: 169.2019403337908 usec\nrounds: 6771"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_pyd",
            "value": 290.1248444195782,
            "unit": "iter/sec",
            "range": "stddev: 0.0002063584807684492",
            "extra": "mean: 3.4467920250008 msec\nrounds: 320"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2501.5343584087886,
            "unit": "iter/sec",
            "range": "stddev: 0.00006278202012118965",
            "extra": "mean: 399.75465323454284 usec\nrounds: 2829"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_pyd_json",
            "value": 276.7059812448329,
            "unit": "iter/sec",
            "range": "stddev: 0.0002598401843714078",
            "extra": "mean: 3.613944286644051 msec\nrounds: 307"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 4411.128098346912,
            "unit": "iter/sec",
            "range": "stddev: 0.00006489568629268579",
            "extra": "mean: 226.69937886744978 usec\nrounds: 5139"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_pyd",
            "value": 145.1186944969275,
            "unit": "iter/sec",
            "range": "stddev: 0.00032181456696154295",
            "extra": "mean: 6.890910943394495 msec\nrounds: 159"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 768.739356314636,
            "unit": "iter/sec",
            "range": "stddev: 0.0003579689462154051",
            "extra": "mean: 1.3008310187135932 msec\nrounds: 855"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_pyd_json",
            "value": 137.82297195408938,
            "unit": "iter/sec",
            "range": "stddev: 0.0007019830762994413",
            "extra": "mean: 7.255684490195967 msec\nrounds: 153"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1378.8414408907752,
            "unit": "iter/sec",
            "range": "stddev: 0.00010030355235325868",
            "extra": "mean: 725.2465514482711 usec\nrounds: 1623"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_pyd",
            "value": 152.50716855855796,
            "unit": "iter/sec",
            "range": "stddev: 0.0009241539608710254",
            "extra": "mean: 6.557068821430721 msec\nrounds: 168"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1500.8985567655768,
            "unit": "iter/sec",
            "range": "stddev: 0.00009544097699597142",
            "extra": "mean: 666.2675471918577 usec\nrounds: 1727"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 267.20326498035223,
            "unit": "iter/sec",
            "range": "stddev: 0.08880268475463554",
            "extra": "mean: 3.7424692399381088 msec\nrounds: 1292"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_pyd",
            "value": 661.6059169043626,
            "unit": "iter/sec",
            "range": "stddev: 0.0002902834166798665",
            "extra": "mean: 1.5114737859041145 msec\nrounds: 752"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 27273.29488253122,
            "unit": "iter/sec",
            "range": "stddev: 0.000017312752550409826",
            "extra": "mean: 36.66590356270113 usec\nrounds: 32363"
          }
        ]
      }
    ]
  }
}