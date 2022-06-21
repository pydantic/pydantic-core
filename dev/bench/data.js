window.BENCHMARK_DATA = {
  "lastUpdate": 1655817315788,
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
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 25350.024953455166,
            "unit": "iter/sec",
            "range": "stddev: 0.001305452273109351",
            "extra": "mean: 39.447692924803285 usec\nrounds: 54345"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 215947.6208061508,
            "unit": "iter/sec",
            "range": "stddev: 0.00001279990227496455",
            "extra": "mean: 4.630752569845017 usec\nrounds: 129871"
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
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 1676.6594747657155,
            "unit": "iter/sec",
            "range": "stddev: 0.027675612373301333",
            "extra": "mean: 596.424029476667 usec\nrounds: 9058"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 17358.965045785382,
            "unit": "iter/sec",
            "range": "stddev: 0.000029691219797545142",
            "extra": "mean: 57.607121009947086 usec\nrounds: 20081"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3120.596545716897,
            "unit": "iter/sec",
            "range": "stddev: 0.0001095284458401033",
            "extra": "mean: 320.45155000012 usec\nrounds: 3540"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 5910.097709442715,
            "unit": "iter/sec",
            "range": "stddev: 0.00007716428180793458",
            "extra": "mean: 169.2019403337908 usec\nrounds: 6771"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2501.5343584087886,
            "unit": "iter/sec",
            "range": "stddev: 0.00006278202012118965",
            "extra": "mean: 399.75465323454284 usec\nrounds: 2829"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 4411.128098346912,
            "unit": "iter/sec",
            "range": "stddev: 0.00006489568629268579",
            "extra": "mean: 226.69937886744978 usec\nrounds: 5139"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 768.739356314636,
            "unit": "iter/sec",
            "range": "stddev: 0.0003579689462154051",
            "extra": "mean: 1.3008310187135932 msec\nrounds: 855"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1378.8414408907752,
            "unit": "iter/sec",
            "range": "stddev: 0.00010030355235325868",
            "extra": "mean: 725.2465514482711 usec\nrounds: 1623"
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
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 27273.29488253122,
            "unit": "iter/sec",
            "range": "stddev: 0.000017312752550409826",
            "extra": "mean: 36.66590356270113 usec\nrounds: 32363"
          }
        ]
      },
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
          "id": "22c1cae28b8eac6039f51d7a33c236d20bba3fed",
          "message": "tweaks to benchmarks CI",
          "timestamp": "2022-05-11T10:40:22+01:00",
          "tree_id": "586e12010011afafe3d6e41d99968d9a93093ba8",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/22c1cae28b8eac6039f51d7a33c236d20bba3fed"
        },
        "date": 1652262182317,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 34478.43001360939,
            "unit": "iter/sec",
            "range": "stddev: 0.0008853902776968035",
            "extra": "mean: 29.003640815584646 usec\nrounds: 64936"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 22047.052882601965,
            "unit": "iter/sec",
            "range": "stddev: 0.0017989817789205095",
            "extra": "mean: 45.35753623510977 usec\nrounds: 40320"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 304774.7114874032,
            "unit": "iter/sec",
            "range": "stddev: 4.850711985617787e-7",
            "extra": "mean: 3.2811121208832037 usec\nrounds: 156251"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1219172.8936220936,
            "unit": "iter/sec",
            "range": "stddev: 4.7599405274193255e-8",
            "extra": "mean: 820.2282098224388 nsec\nrounds: 123457"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 191828.7480400903,
            "unit": "iter/sec",
            "range": "stddev: 0.0004203535846785866",
            "extra": "mean: 5.212982987258082 usec\nrounds: 89286"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 2191.7446912232904,
            "unit": "iter/sec",
            "range": "stddev: 0.019210177770001724",
            "extra": "mean: 456.25752123613654 usec\nrounds: 11325"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 22880.06102813421,
            "unit": "iter/sec",
            "range": "stddev: 0.0000010866602842384837",
            "extra": "mean: 43.706177128214875 usec\nrounds: 22989"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3867.060942662483,
            "unit": "iter/sec",
            "range": "stddev: 0.0000017846776508847355",
            "extra": "mean: 258.59432132752914 usec\nrounds: 3887"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 8181.079786855855,
            "unit": "iter/sec",
            "range": "stddev: 0.000001259592010680422",
            "extra": "mean: 122.23325356228057 usec\nrounds: 8211"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 3429.0223473492,
            "unit": "iter/sec",
            "range": "stddev: 0.000002707741346858946",
            "extra": "mean: 291.62831230104064 usec\nrounds: 3455"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 6206.210226807806,
            "unit": "iter/sec",
            "range": "stddev: 0.0000015960080522008193",
            "extra": "mean: 161.12892787300163 usec\nrounds: 6239"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1083.6496892849405,
            "unit": "iter/sec",
            "range": "stddev: 0.000004191645523801962",
            "extra": "mean: 922.8074440365154 usec\nrounds: 1090"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1904.825536775508,
            "unit": "iter/sec",
            "range": "stddev: 0.000008904548536466302",
            "extra": "mean: 524.9824620121387 usec\nrounds: 1948"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1992.8849188928682,
            "unit": "iter/sec",
            "range": "stddev: 0.0001021306419722112",
            "extra": "mean: 501.7851209168377 usec\nrounds: 2051"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 329.2671309190059,
            "unit": "iter/sec",
            "range": "stddev: 0.07157791471822271",
            "extra": "mean: 3.0370477527135344 msec\nrounds: 1290"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 37822.93552312102,
            "unit": "iter/sec",
            "range": "stddev: 6.427638900628171e-7",
            "extra": "mean: 26.43898434030071 usec\nrounds: 38315"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "s@muelcolvin.com",
            "name": "Samuel Colvin",
            "username": "samuelcolvin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "18c4102a3c89515fd3999d7a5e05121df6ef3abf",
          "message": "add benchmarks to CI (#66)\n\n* add benchmarks to CI\r\n\r\n* improving benchmarks\r\n\r\n* decrease warmup iterations\r\n\r\n* add link to benchmarks to readme\r\n\r\n* remove benchmarks from ci.yml",
          "timestamp": "2022-05-11T11:10:40+01:00",
          "tree_id": "daa9781b24da75fe5a2b37456432b0e61eacf7a8",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/18c4102a3c89515fd3999d7a5e05121df6ef3abf"
        },
        "date": 1652263966333,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 35948.72849252688,
            "unit": "iter/sec",
            "range": "stddev: 0.0005112228789003178",
            "extra": "mean: 27.81739555010639 usec\nrounds: 56180"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 24233.791619751304,
            "unit": "iter/sec",
            "range": "stddev: 0.0010573158437508264",
            "extra": "mean: 41.26469417955086 usec\nrounds: 34723"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 252491.87851524877,
            "unit": "iter/sec",
            "range": "stddev: 1.63212302457631e-7",
            "extra": "mean: 3.9605234270519594 usec\nrounds: 129871"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 984497.5109723401,
            "unit": "iter/sec",
            "range": "stddev: 4.589521201877868e-8",
            "extra": "mean: 1.015746600529712 usec\nrounds: 100001"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 212081.125326338,
            "unit": "iter/sec",
            "range": "stddev: 0.0003119399855076994",
            "extra": "mean: 4.715176791245636 usec\nrounds: 74627"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 2528.0329095564457,
            "unit": "iter/sec",
            "range": "stddev: 0.013044415433749774",
            "extra": "mean: 395.5644707866775 usec\nrounds: 8592"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 19320.4844170213,
            "unit": "iter/sec",
            "range": "stddev: 9.700711282877019e-7",
            "extra": "mean: 51.758536608895916 usec\nrounds: 19763"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3797.688620750334,
            "unit": "iter/sec",
            "range": "stddev: 0.0000021026532075322585",
            "extra": "mean: 263.3180599736541 usec\nrounds: 3835"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 7027.210504976325,
            "unit": "iter/sec",
            "range": "stddev: 0.0000015235936252939916",
            "extra": "mean: 142.30397670481753 usec\nrounds: 7083"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 3102.381270728865,
            "unit": "iter/sec",
            "range": "stddev: 0.0000025815862512631983",
            "extra": "mean: 322.33304443752746 usec\nrounds: 3128"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5517.800116951731,
            "unit": "iter/sec",
            "range": "stddev: 0.0000020278599697715628",
            "extra": "mean: 181.23164645413846 usec\nrounds: 5541"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 919.2315866008385,
            "unit": "iter/sec",
            "range": "stddev: 0.000004035725505970004",
            "extra": "mean: 1.0878651414686795 msec\nrounds: 926"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1621.6838047974477,
            "unit": "iter/sec",
            "range": "stddev: 0.000004152889618443191",
            "extra": "mean: 616.6430206934838 usec\nrounds: 1643"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1719.9167879850604,
            "unit": "iter/sec",
            "range": "stddev: 0.000006697017179830309",
            "extra": "mean: 581.4234775692452 usec\nrounds: 1761"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 422.58846569927977,
            "unit": "iter/sec",
            "range": "stddev: 0.04453661325612685",
            "extra": "mean: 2.3663684202673316 msec\nrounds: 1273"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 34154.59513672076,
            "unit": "iter/sec",
            "range": "stddev: 7.825404243656271e-7",
            "extra": "mean: 29.278637208170743 usec\nrounds: 35089"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "s@muelcolvin.com",
            "name": "Samuel Colvin",
            "username": "samuelcolvin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "69098c4f8a4b20f08a8b982f66079bba10481600",
          "message": "List as enum (#54)\n\n* use enum dispatch for sequences\r\n\r\n* rename ListInput GenericSequence\r\n\r\n* implement generic dict as enum\r\n\r\n* more enum dispatch on sequences\r\n\r\n* more enum dispatch on mappings\r\n\r\n* a few more inlines\r\n\r\n* tweak dict validator\r\n\r\n* incorporate enumerate\r\n\r\n* remove unenecessary iterator\r\n\r\n* more list and dict tests",
          "timestamp": "2022-05-11T11:13:05+01:00",
          "tree_id": "dd48feedd71ddc6c8e903078fbdc6ea0e03e7d39",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/69098c4f8a4b20f08a8b982f66079bba10481600"
        },
        "date": 1652264131174,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 26425.928721566695,
            "unit": "iter/sec",
            "range": "stddev: 0.0008253042824140784",
            "extra": "mean: 37.8416217850418 usec\nrounds: 55249"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 15692.497237175516,
            "unit": "iter/sec",
            "range": "stddev: 0.0022479104248595686",
            "extra": "mean: 63.72472047540022 usec\nrounds: 30212"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 202749.6986856139,
            "unit": "iter/sec",
            "range": "stddev: 0.000009514269588002117",
            "extra": "mean: 4.932189820664601 usec\nrounds: 138889"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 967009.3276073326,
            "unit": "iter/sec",
            "range": "stddev: 9.963077551663712e-7",
            "extra": "mean: 1.0341161883869452 usec\nrounds: 113637"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 160385.82002064548,
            "unit": "iter/sec",
            "range": "stddev: 0.000484811131064702",
            "extra": "mean: 6.234965160082686 usec\nrounds: 99010"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 1393.2334570120404,
            "unit": "iter/sec",
            "range": "stddev: 0.027217136197092463",
            "extra": "mean: 717.7547990733889 usec\nrounds: 7988"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 17714.985874435533,
            "unit": "iter/sec",
            "range": "stddev: 0.00005837626954685201",
            "extra": "mean: 56.44938173183069 usec\nrounds: 20834"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 2930.935027629462,
            "unit": "iter/sec",
            "range": "stddev: 0.00007522970552979807",
            "extra": "mean: 341.18804769575496 usec\nrounds: 3732"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 5072.416889774781,
            "unit": "iter/sec",
            "range": "stddev: 0.00007534671856079127",
            "extra": "mean: 197.1446791007749 usec\nrounds: 6407"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2325.300805720719,
            "unit": "iter/sec",
            "range": "stddev: 0.00008595765958114616",
            "extra": "mean: 430.0518872826234 usec\nrounds: 3043"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 3893.5109545926093,
            "unit": "iter/sec",
            "range": "stddev: 0.00012570944997130018",
            "extra": "mean: 256.83759764960854 usec\nrounds: 4936"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 791.3929305485312,
            "unit": "iter/sec",
            "range": "stddev: 0.00014191811760875798",
            "extra": "mean: 1.2635948103641494 msec\nrounds: 907"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1517.635066037196,
            "unit": "iter/sec",
            "range": "stddev: 0.00008983328680023544",
            "extra": "mean: 658.9199356148052 usec\nrounds: 1724"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1454.3250879528025,
            "unit": "iter/sec",
            "range": "stddev: 0.0001173224075931947",
            "extra": "mean: 687.6041734297945 usec\nrounds: 1799"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 277.75573655824604,
            "unit": "iter/sec",
            "range": "stddev: 0.07403441379634378",
            "extra": "mean: 3.600285676873131 msec\nrounds: 1241"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 24912.50305087494,
            "unit": "iter/sec",
            "range": "stddev: 0.00001950293516996522",
            "extra": "mean: 40.14048680527424 usec\nrounds: 30770"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "1755071+adriangb@users.noreply.github.com",
            "name": "Adrian Garcia Badaracco",
            "username": "adriangb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f6d4184389f3dda4cecc1dc3402dc63d76a8ccab",
          "message": "implement __reduce__ to make SchemaValidator picklable (#53)\n\n* attempt at implementing __reduce__\r\n\r\n* python version\r\n\r\n* remove loc\r\n\r\n* back to rust\r\n\r\n* use array\r\n\r\n* minimize changes\r\n\r\n* again use array\r\n\r\n* remove import\r\n\r\n* test all protocol versions\r\n\r\n* Update src/validators/mod.rs\r\n\r\nCo-authored-by: Samuel Colvin <samcolvin@gmail.com>\r\n\r\n* Update src/validators/mod.rs\r\n\r\nCo-authored-by: Samuel Colvin <samcolvin@gmail.com>\r\n\r\n* pr feedback\r\n\r\n* pointlessly tweak layout\r\n\r\nCo-authored-by: Samuel Colvin <samcolvin@gmail.com>\r\nCo-authored-by: Samuel Colvin <s@muelcolvin.com>",
          "timestamp": "2022-05-11T12:22:54+01:00",
          "tree_id": "009c56dc5106873a6189dc7f72b41e81c90c6b15",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/f6d4184389f3dda4cecc1dc3402dc63d76a8ccab"
        },
        "date": 1652268315085,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 27646.82615096265,
            "unit": "iter/sec",
            "range": "stddev: 0.0007975823425689881",
            "extra": "mean: 36.17051717038343 usec\nrounds: 48077"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 15149.687370138572,
            "unit": "iter/sec",
            "range": "stddev: 0.0022246765055872137",
            "extra": "mean: 66.0079627762545 usec\nrounds: 27778"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 210741.81608731038,
            "unit": "iter/sec",
            "range": "stddev: 0.0000067515951706193614",
            "extra": "mean: 4.745142746543001 usec\nrounds: 112360"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 883093.3428389295,
            "unit": "iter/sec",
            "range": "stddev: 0.0000014495441739468722",
            "extra": "mean: 1.1323831258711896 usec\nrounds: 91744"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 161641.09455036782,
            "unit": "iter/sec",
            "range": "stddev: 0.00042499719434656096",
            "extra": "mean: 6.186545585958117 usec\nrounds: 70423"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 1572.7391911969876,
            "unit": "iter/sec",
            "range": "stddev: 0.022113796840178408",
            "extra": "mean: 635.833331805584 usec\nrounds: 7200"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 16907.91211195713,
            "unit": "iter/sec",
            "range": "stddev: 0.00005580116911805513",
            "extra": "mean: 59.143908093348124 usec\nrounds: 17953"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 2832.9325369466587,
            "unit": "iter/sec",
            "range": "stddev: 0.0000803674449148478",
            "extra": "mean: 352.99110972752015 usec\nrounds: 2971"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 5210.640827559687,
            "unit": "iter/sec",
            "range": "stddev: 0.00011717054010032895",
            "extra": "mean: 191.91497420257474 usec\nrounds: 5737"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2324.545962760404,
            "unit": "iter/sec",
            "range": "stddev: 0.00010386925226006375",
            "extra": "mean: 430.19153676466675 usec\nrounds: 2448"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 4094.5118114319876,
            "unit": "iter/sec",
            "range": "stddev: 0.00005619113276618101",
            "extra": "mean: 244.22936019087138 usec\nrounds: 4406"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 695.119966686474,
            "unit": "iter/sec",
            "range": "stddev: 0.00026569760420190066",
            "extra": "mean: 1.4386005983497214 msec\nrounds: 727"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1321.9664933888203,
            "unit": "iter/sec",
            "range": "stddev: 0.00011226090217757379",
            "extra": "mean: 756.4488245360371 usec\nrounds: 1402"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 653.3834516478503,
            "unit": "iter/sec",
            "range": "stddev: 0.033256881933444185",
            "extra": "mean: 1.5304948380280732 msec\nrounds: 1562"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 302.54897093272393,
            "unit": "iter/sec",
            "range": "stddev: 0.06511875669096467",
            "extra": "mean: 3.3052500456938065 msec\nrounds: 1138"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 26686.354120505068,
            "unit": "iter/sec",
            "range": "stddev: 0.000025274712767584005",
            "extra": "mean: 37.47233494258503 usec\nrounds: 27548"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "1755071+adriangb@users.noreply.github.com",
            "name": "Adrian Garcia Badaracco",
            "username": "adriangb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6d0da7806d338ed9f2d34561921ed1dc0a1eb67b",
          "message": "add types (#50)\n\n* add types\r\n\r\n* guard versions\r\n\r\n* move TypedDict\r\n\r\n* remove mypy in favor of pyright, add test for typing\r\n\r\n* revert unecessary change\r\n\r\n* Update pydantic_core/_types.py\r\n\r\nCo-authored-by: Samuel Colvin <samcolvin@gmail.com>\r\n\r\n* pr feedback\r\n\r\n* fix filename\r\n\r\n* Update Makefile\r\n\r\n* a few tweaks\r\n\r\n* :-( fix tests\r\n\r\n* fix pyright complaint with pytest\r\n\r\nCo-authored-by: Samuel Colvin <samcolvin@gmail.com>\r\nCo-authored-by: Samuel Colvin <s@muelcolvin.com>",
          "timestamp": "2022-05-11T12:25:50+01:00",
          "tree_id": "082d7ece6733b164470ad48b37be1e2ac9ddf592",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/6d0da7806d338ed9f2d34561921ed1dc0a1eb67b"
        },
        "date": 1652268498810,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 25888.45636406876,
            "unit": "iter/sec",
            "range": "stddev: 0.0011369151152302384",
            "extra": "mean: 38.62725478634273 usec\nrounds: 52911"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 17969.151835993896,
            "unit": "iter/sec",
            "range": "stddev: 0.0015764323353242965",
            "extra": "mean: 55.650929388715284 usec\nrounds: 30675"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 206548.13777352165,
            "unit": "iter/sec",
            "range": "stddev: 0.000005368090415345586",
            "extra": "mean: 4.841486400117011 usec\nrounds: 123457"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 924146.7842567787,
            "unit": "iter/sec",
            "range": "stddev: 0.000001241803955507628",
            "extra": "mean: 1.0820791859426202 usec\nrounds: 99010"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 172170.43180692432,
            "unit": "iter/sec",
            "range": "stddev: 0.00044408414093048194",
            "extra": "mean: 5.808198245802231 usec\nrounds: 102041"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 1574.7511305022754,
            "unit": "iter/sec",
            "range": "stddev: 0.023881813937707237",
            "extra": "mean: 635.0209760961052 usec\nrounds: 8827"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 18199.33484699339,
            "unit": "iter/sec",
            "range": "stddev: 0.000019786986406034487",
            "extra": "mean: 54.947063088143814 usec\nrounds: 23697"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 2909.444171372192,
            "unit": "iter/sec",
            "range": "stddev: 0.00006072032027683822",
            "extra": "mean: 343.7082621621044 usec\nrounds: 3700"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 5538.861457677753,
            "unit": "iter/sec",
            "range": "stddev: 0.000033933606324276596",
            "extra": "mean: 180.54251900701345 usec\nrounds: 6366"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2379.620570168635,
            "unit": "iter/sec",
            "range": "stddev: 0.00006199910384062807",
            "extra": "mean: 420.2350629071649 usec\nrounds: 3068"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 4364.255145223726,
            "unit": "iter/sec",
            "range": "stddev: 0.00004341270610409454",
            "extra": "mean: 229.13417449811746 usec\nrounds: 5129"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 796.2700752827271,
            "unit": "iter/sec",
            "range": "stddev: 0.0001432222183208433",
            "extra": "mean: 1.2558553071895056 msec\nrounds: 918"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1470.3445720979482,
            "unit": "iter/sec",
            "range": "stddev: 0.00008388658171775615",
            "extra": "mean: 680.1126885333815 usec\nrounds: 1875"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1454.207870762067,
            "unit": "iter/sec",
            "range": "stddev: 0.00010392745442016923",
            "extra": "mean: 687.6595981260625 usec\nrounds: 2028"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 272.6086190203901,
            "unit": "iter/sec",
            "range": "stddev: 0.080465091017438",
            "extra": "mean: 3.668262594166928 msec\nrounds: 1200"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 26621.840067271758,
            "unit": "iter/sec",
            "range": "stddev: 0.000022022978230362823",
            "extra": "mean: 37.56314354954659 usec\nrounds: 34246"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "1939362+davidhewitt@users.noreply.github.com",
            "name": "David Hewitt",
            "username": "davidhewitt"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6339ef58bba2171a3df38092cc40719b5dbb7d5d",
          "message": "fix memory leak in create_class (#58)",
          "timestamp": "2022-05-11T14:11:13+01:00",
          "tree_id": "e5861ba166c3eb8e13faf27a72d3db0ab804a4db",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/6339ef58bba2171a3df38092cc40719b5dbb7d5d"
        },
        "date": 1652274782216,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 56125.25611919934,
            "unit": "iter/sec",
            "range": "stddev: 0.000002154338167947614",
            "extra": "mean: 17.81729063073121 usec\nrounds: 57475"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 34160.19355471772,
            "unit": "iter/sec",
            "range": "stddev: 6.911352924156852e-7",
            "extra": "mean: 29.273838814707016 usec\nrounds: 34966"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 234944.5620142351,
            "unit": "iter/sec",
            "range": "stddev: 0.0000023605789465374043",
            "extra": "mean: 4.2563232424992705 usec\nrounds: 129887"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1024324.3809719951,
            "unit": "iter/sec",
            "range": "stddev: 4.427042584890591e-8",
            "extra": "mean: 976.2532441637159 nsec\nrounds: 103093"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 765535.4243188262,
            "unit": "iter/sec",
            "range": "stddev: 4.470620026427993e-8",
            "extra": "mean: 1.3062752790175831 usec\nrounds: 78132"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 10427.019403788283,
            "unit": "iter/sec",
            "range": "stddev: 0.00000194715001742517",
            "extra": "mean: 95.90468390580398 usec\nrounds: 10538"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 20393.04444770523,
            "unit": "iter/sec",
            "range": "stddev: 0.0000023794466795897523",
            "extra": "mean: 49.03632719304582 usec\nrounds: 21009"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3683.5354149789714,
            "unit": "iter/sec",
            "range": "stddev: 0.000002198173636732866",
            "extra": "mean: 271.47831833882583 usec\nrounds: 3757"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6820.068456141803,
            "unit": "iter/sec",
            "range": "stddev: 0.0000015401239190294125",
            "extra": "mean: 146.62609421456048 usec\nrounds: 6931"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 3014.1975643744013,
            "unit": "iter/sec",
            "range": "stddev: 0.0000020176237266691643",
            "extra": "mean: 331.7632566024419 usec\nrounds: 3067"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5382.471981961838,
            "unit": "iter/sec",
            "range": "stddev: 0.000001745317168273361",
            "extra": "mean: 185.78824067292473 usec\nrounds: 5468"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 929.9173587121096,
            "unit": "iter/sec",
            "range": "stddev: 0.0000033152685632784797",
            "extra": "mean: 1.0753643758031912 msec\nrounds: 934"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1850.5713348190318,
            "unit": "iter/sec",
            "range": "stddev: 0.000003248757742732661",
            "extra": "mean: 540.3736571429117 usec\nrounds: 1855"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1474.6477681124256,
            "unit": "iter/sec",
            "range": "stddev: 0.0010653025899753955",
            "extra": "mean: 678.1280395385652 usec\nrounds: 1821"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 935.8844736930756,
            "unit": "iter/sec",
            "range": "stddev: 0.001402963879981752",
            "extra": "mean: 1.0685079495484302 msec\nrounds: 1328"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 30496.26866248678,
            "unit": "iter/sec",
            "range": "stddev: 8.639986591884395e-7",
            "extra": "mean: 32.79089684929528 usec\nrounds: 32787"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "s@muelcolvin.com",
            "name": "Samuel Colvin",
            "username": "samuelcolvin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e50eecc6853047adc478d82b7c2992fdc1a6c3d3",
          "message": "prevent segfault on recursive schema (#67)\n\n* prevent segfault on recursive schema, fix #62\r\n\r\n* linting\r\n\r\n* use RecursionError",
          "timestamp": "2022-05-11T16:36:47+01:00",
          "tree_id": "d7f035fc984c71bb47dd761f2fc6fae0495e928c",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/e50eecc6853047adc478d82b7c2992fdc1a6c3d3"
        },
        "date": 1652283520286,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 55709.571435547376,
            "unit": "iter/sec",
            "range": "stddev: 6.481608376856237e-7",
            "extra": "mean: 17.950236812662254 usec\nrounds: 58143"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 34565.2795029435,
            "unit": "iter/sec",
            "range": "stddev: 8.692515433221771e-7",
            "extra": "mean: 28.930765623198337 usec\nrounds: 35716"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 241221.31519455774,
            "unit": "iter/sec",
            "range": "stddev: 0.0000016244103551572994",
            "extra": "mean: 4.1455706316560255 usec\nrounds: 129871"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 997906.3101180287,
            "unit": "iter/sec",
            "range": "stddev: 1.0637072850682701e-7",
            "extra": "mean: 1.0020980826162114 usec\nrounds: 106395"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 830043.0768864807,
            "unit": "iter/sec",
            "range": "stddev: 1.1702857145750747e-7",
            "extra": "mean: 1.2047567503978138 usec\nrounds: 90091"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 10442.104444550348,
            "unit": "iter/sec",
            "range": "stddev: 0.0000026421794288242446",
            "extra": "mean: 95.76613653983244 usec\nrounds: 10583"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 20855.224381190576,
            "unit": "iter/sec",
            "range": "stddev: 0.000002627183840884538",
            "extra": "mean: 47.94961596778142 usec\nrounds: 21368"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3663.0218690554293,
            "unit": "iter/sec",
            "range": "stddev: 0.000002769266335433024",
            "extra": "mean: 272.9986431279119 usec\nrounds: 3696"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6729.568454447578,
            "unit": "iter/sec",
            "range": "stddev: 0.0000018087208691076866",
            "extra": "mean: 148.59793860022316 usec\nrounds: 6873"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2989.564243855891,
            "unit": "iter/sec",
            "range": "stddev: 0.0000025352383145618274",
            "extra": "mean: 334.4969093924592 usec\nrounds: 3013"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5372.706200664851,
            "unit": "iter/sec",
            "range": "stddev: 0.000002161086198724889",
            "extra": "mean: 186.12594149969598 usec\nrounds: 5453"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 931.5784065113894,
            "unit": "iter/sec",
            "range": "stddev: 0.000005302604681338501",
            "extra": "mean: 1.073446950906514 msec\nrounds: 937"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1880.300090318752,
            "unit": "iter/sec",
            "range": "stddev: 0.000003566986056951173",
            "extra": "mean: 531.8300015772897 usec\nrounds: 1902"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1404.8927403231994,
            "unit": "iter/sec",
            "range": "stddev: 0.00133086584340705",
            "extra": "mean: 711.7981119113387 usec\nrounds: 1805"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 948.7103590527059,
            "unit": "iter/sec",
            "range": "stddev: 0.0016930162646991893",
            "extra": "mean: 1.0540624864668993 msec\nrounds: 1330"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 32360.621031643448,
            "unit": "iter/sec",
            "range": "stddev: 7.493183377878723e-7",
            "extra": "mean: 30.90175553250854 usec\nrounds: 32896"
          }
        ]
      },
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
          "id": "82fec734e74537c29153019d83acb29829e79c6c",
          "message": "parse strings as ints in benchmarks",
          "timestamp": "2022-05-24T11:47:30+01:00",
          "tree_id": "c2c0ed9db7b7ecc4804f25b40263455f48c22464",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/82fec734e74537c29153019d83acb29829e79c6c"
        },
        "date": 1653389430856,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 64086.09499215597,
            "unit": "iter/sec",
            "range": "stddev: 0.0000013476486408914612",
            "extra": "mean: 15.604008952681516 usec\nrounds: 65790"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 40153.137587415746,
            "unit": "iter/sec",
            "range": "stddev: 0.0000017862361528299602",
            "extra": "mean: 24.90465403414468 usec\nrounds: 43669"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 279512.08887102816,
            "unit": "iter/sec",
            "range": "stddev: 4.21640008115128e-7",
            "extra": "mean: 3.577662791040919 usec\nrounds: 142858"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1259386.6117416825,
            "unit": "iter/sec",
            "range": "stddev: 8.73830345989386e-8",
            "extra": "mean: 794.0373437961391 nsec\nrounds: 128206"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 1020671.9227982287,
            "unit": "iter/sec",
            "range": "stddev: 9.542437246556943e-8",
            "extra": "mean: 979.7467508062863 nsec\nrounds: 102041"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 12307.841938735806,
            "unit": "iter/sec",
            "range": "stddev: 0.0000036890542666971796",
            "extra": "mean: 81.24901221332344 usec\nrounds: 12691"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 24292.789814105196,
            "unit": "iter/sec",
            "range": "stddev: 0.0000022057831277830815",
            "extra": "mean: 41.16447751173342 usec\nrounds: 24391"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3040.1844579265758,
            "unit": "iter/sec",
            "range": "stddev: 0.000010293146530047925",
            "extra": "mean: 328.92741010918996 usec\nrounds: 3304"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 7451.954583948222,
            "unit": "iter/sec",
            "range": "stddev: 0.000005148469156432471",
            "extra": "mean: 134.192981013872 usec\nrounds: 8111"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 3378.4360368103967,
            "unit": "iter/sec",
            "range": "stddev: 0.000009029967955373785",
            "extra": "mean: 295.99494828503737 usec\nrounds: 3674"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 6140.793835723087,
            "unit": "iter/sec",
            "range": "stddev: 0.000005836808675612388",
            "extra": "mean: 162.84539535958035 usec\nrounds: 6680"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1071.3721165641903,
            "unit": "iter/sec",
            "range": "stddev: 0.00003069582720802522",
            "extra": "mean: 933.3825143843809 usec\nrounds: 1182"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 2064.3741253487347,
            "unit": "iter/sec",
            "range": "stddev: 0.000013973007226040953",
            "extra": "mean: 484.4083190739809 usec\nrounds: 2244"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1354.388385646058,
            "unit": "iter/sec",
            "range": "stddev: 0.00200552327167778",
            "extra": "mean: 738.3406492540092 usec\nrounds: 2278"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 931.1960432882586,
            "unit": "iter/sec",
            "range": "stddev: 0.0024575087700413937",
            "extra": "mean: 1.07388772451049 msec\nrounds: 1677"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 36854.50335359227,
            "unit": "iter/sec",
            "range": "stddev: 0.0000017147740033953777",
            "extra": "mean: 27.133726112267045 usec\nrounds: 38315"
          }
        ]
      },
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
          "id": "8f26e13100a91deadebafe58e7d7bf8d5244b9fc",
          "message": "parse longer strings as ints in benchmarks",
          "timestamp": "2022-05-24T11:55:47+01:00",
          "tree_id": "4af27a74b5479d969e010fbf02e8960488a73e8b",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/8f26e13100a91deadebafe58e7d7bf8d5244b9fc"
        },
        "date": 1653389846281,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 55691.651898372584,
            "unit": "iter/sec",
            "range": "stddev: 4.6122949876646734e-7",
            "extra": "mean: 17.95601254250499 usec\nrounds: 57804"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 34857.93643657332,
            "unit": "iter/sec",
            "range": "stddev: 5.352251937475953e-7",
            "extra": "mean: 28.68787146420949 usec\nrounds: 35461"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 247168.06790147963,
            "unit": "iter/sec",
            "range": "stddev: 1.4497463481118679e-7",
            "extra": "mean: 4.045830064094674 usec\nrounds: 126583"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1098799.7608589416,
            "unit": "iter/sec",
            "range": "stddev: 3.635833604369633e-8",
            "extra": "mean: 910.0839257712566 nsec\nrounds: 112360"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 892779.3150418202,
            "unit": "iter/sec",
            "range": "stddev: 3.2199636709558666e-8",
            "extra": "mean: 1.1200976357223995 usec\nrounds: 90091"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 10768.835327802735,
            "unit": "iter/sec",
            "range": "stddev: 0.0000015697880561416354",
            "extra": "mean: 92.8605526558868 usec\nrounds: 10882"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 21260.02314876261,
            "unit": "iter/sec",
            "range": "stddev: 9.087576133163245e-7",
            "extra": "mean: 47.03663740169552 usec\nrounds: 21368"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3642.481443061177,
            "unit": "iter/sec",
            "range": "stddev: 0.0000026963406455467546",
            "extra": "mean: 274.5381179374219 usec\nrounds: 3646"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6862.012965501256,
            "unit": "iter/sec",
            "range": "stddev: 0.0000014710134474737006",
            "extra": "mean: 145.72983248902273 usec\nrounds: 6907"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2999.013680789453,
            "unit": "iter/sec",
            "range": "stddev: 0.0000023197147019880625",
            "extra": "mean: 333.44296039915446 usec\nrounds: 3005"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5303.614705359805,
            "unit": "iter/sec",
            "range": "stddev: 0.000019881365910791523",
            "extra": "mean: 188.55064999148698 usec\nrounds: 5397"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 934.5958367385317,
            "unit": "iter/sec",
            "range": "stddev: 0.00003504622997816105",
            "extra": "mean: 1.0699812268474358 msec\nrounds: 961"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1803.5640571036258,
            "unit": "iter/sec",
            "range": "stddev: 0.000003232822699377603",
            "extra": "mean: 554.4577116966485 usec\nrounds: 1821"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1364.2742095173735,
            "unit": "iter/sec",
            "range": "stddev: 0.0014005945568061216",
            "extra": "mean: 732.990474366411 usec\nrounds: 1853"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 944.6270277142688,
            "unit": "iter/sec",
            "range": "stddev: 0.0018184115736810226",
            "extra": "mean: 1.0586188735460154 msec\nrounds: 1376"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 32076.68487655751,
            "unit": "iter/sec",
            "range": "stddev: 5.614303075727336e-7",
            "extra": "mean: 31.17529145696806 usec\nrounds: 32787"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "messense@icloud.com",
            "name": "messense",
            "username": "messense"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a8d7042df471e03ca64087b7b6e6533ded24b34b",
          "message": "Switch from setuptools-rust to maturin (#70)\n\n* Switch from setuptools-rust to maturin\r\n\r\n* Update project metadata\r\n\r\n* Fix CI lint job\r\n\r\n* Build wheels on CI\r\n\r\n* Change `mimalloc` to optional dependency\r\n\r\nEnabled by default.\r\n\r\n* fix clippy errors\r\n\r\nCo-authored-by: Samuel Colvin <s@muelcolvin.com>",
          "timestamp": "2022-06-08T09:54:54+01:00",
          "tree_id": "896b00dd95e2ff37f90e93bf28137b8e7805a5f4",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/a8d7042df471e03ca64087b7b6e6533ded24b34b"
        },
        "date": 1654678699778,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 55409.0363429414,
            "unit": "iter/sec",
            "range": "stddev: 5.542860656464437e-7",
            "extra": "mean: 18.047597756631816 usec\nrounds: 57147"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 33566.91287803917,
            "unit": "iter/sec",
            "range": "stddev: 7.642613967549552e-7",
            "extra": "mean: 29.791241262887787 usec\nrounds: 34365"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 255187.289782605,
            "unit": "iter/sec",
            "range": "stddev: 1.869520650607709e-7",
            "extra": "mean: 3.9186904678987102 usec\nrounds: 131597"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 936917.195705996,
            "unit": "iter/sec",
            "range": "stddev: 4.056023478080951e-8",
            "extra": "mean: 1.067330180920298 usec\nrounds: 95239"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 776848.2797704606,
            "unit": "iter/sec",
            "range": "stddev: 5.358308530974423e-8",
            "extra": "mean: 1.2872526412692764 usec\nrounds: 80000"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 10509.95984531618,
            "unit": "iter/sec",
            "range": "stddev: 0.000001325223228286849",
            "extra": "mean: 95.14784211527271 usec\nrounds: 10628"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 20275.501873252815,
            "unit": "iter/sec",
            "range": "stddev: 0.0000010082256779005542",
            "extra": "mean: 49.32060405958125 usec\nrounds: 20791"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3742.2875128440696,
            "unit": "iter/sec",
            "range": "stddev: 0.0000022728157543627434",
            "extra": "mean: 267.21624048602786 usec\nrounds: 3784"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6748.231900822012,
            "unit": "iter/sec",
            "range": "stddev: 0.0000018186302423010747",
            "extra": "mean: 148.1869643333076 usec\nrounds: 6841"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 3008.384377143372,
            "unit": "iter/sec",
            "range": "stddev: 0.0000022970080924944757",
            "extra": "mean: 332.4043322381416 usec\nrounds: 3043"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5346.290310810537,
            "unit": "iter/sec",
            "range": "stddev: 0.0000018753331278366446",
            "extra": "mean: 187.04558523092857 usec\nrounds: 5403"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 927.3437898913975,
            "unit": "iter/sec",
            "range": "stddev: 0.000005120801304529557",
            "extra": "mean: 1.07834873204587 msec\nrounds: 933"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1822.9066904548197,
            "unit": "iter/sec",
            "range": "stddev: 0.000003559616864633661",
            "extra": "mean: 548.5744307353974 usec\nrounds: 1848"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1327.7668268891325,
            "unit": "iter/sec",
            "range": "stddev: 0.0015211780575642656",
            "extra": "mean: 753.1442868948096 usec\nrounds: 1816"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 879.0550737639638,
            "unit": "iter/sec",
            "range": "stddev: 0.001988989551263764",
            "extra": "mean: 1.1375851523365546 msec\nrounds: 1326"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 32949.370797906005,
            "unit": "iter/sec",
            "range": "stddev: 8.634346056137345e-7",
            "extra": "mean: 30.34959320265842 usec\nrounds: 33899"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "s@muelcolvin.com",
            "name": "Samuel Colvin",
            "username": "samuelcolvin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a4afcb67701bf9765bf442bd850ad3d6b2572fed",
          "message": "improving coverage (#78)\n\n* improving coverage\r\n\r\n* improving to_loc and function coverage",
          "timestamp": "2022-06-08T15:09:42+01:00",
          "tree_id": "171e18fd5953e618bc0e3d73c3845363f0026824",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/a4afcb67701bf9765bf442bd850ad3d6b2572fed"
        },
        "date": 1654697485474,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 47490.01824546574,
            "unit": "iter/sec",
            "range": "stddev: 0.0000012883146343932272",
            "extra": "mean: 21.057056555152577 usec\nrounds: 48077"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 28905.881455235834,
            "unit": "iter/sec",
            "range": "stddev: 0.0000014820035519265808",
            "extra": "mean: 34.59503566942312 usec\nrounds: 31848"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 211417.78693842815,
            "unit": "iter/sec",
            "range": "stddev: 0.0000017243557680592826",
            "extra": "mean: 4.729970994783107 usec\nrounds: 117634"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 866611.0355725328,
            "unit": "iter/sec",
            "range": "stddev: 1.827038840024979e-7",
            "extra": "mean: 1.1539202236659067 usec\nrounds: 91744"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 741514.133008035,
            "unit": "iter/sec",
            "range": "stddev: 6.917137666394715e-8",
            "extra": "mean: 1.3485919626957594 usec\nrounds: 74627"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 9508.609003098773,
            "unit": "iter/sec",
            "range": "stddev: 0.000004512050466906279",
            "extra": "mean: 105.16785364442987 usec\nrounds: 10331"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 17450.85549193271,
            "unit": "iter/sec",
            "range": "stddev: 0.000001949816194837994",
            "extra": "mean: 57.30378092135862 usec\nrounds: 19121"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3011.0326386626316,
            "unit": "iter/sec",
            "range": "stddev: 0.0000071292317785244575",
            "extra": "mean: 332.1119761903863 usec\nrounds: 3276"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 5850.062775606873,
            "unit": "iter/sec",
            "range": "stddev: 0.000006243834966959953",
            "extra": "mean: 170.938336622595 usec\nrounds: 7195"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2502.9685014590727,
            "unit": "iter/sec",
            "range": "stddev: 0.00001527680174484164",
            "extra": "mean: 399.525603065745 usec\nrounds: 3066"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 4483.976622429334,
            "unit": "iter/sec",
            "range": "stddev: 0.000008554539027128086",
            "extra": "mean: 223.01632773861758 usec\nrounds: 5541"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 796.6257388035033,
            "unit": "iter/sec",
            "range": "stddev: 0.000021287208505593248",
            "extra": "mean: 1.2552946148864785 msec\nrounds: 927"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1420.3251044285128,
            "unit": "iter/sec",
            "range": "stddev: 0.00001867773564847241",
            "extra": "mean: 704.0641588901321 usec\nrounds: 1693"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1125.8181936665628,
            "unit": "iter/sec",
            "range": "stddev: 0.001772403411194497",
            "extra": "mean: 888.242884708766 usec\nrounds: 1648"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 771.4874019863694,
            "unit": "iter/sec",
            "range": "stddev: 0.002382660954010357",
            "extra": "mean: 1.296197445901609 msec\nrounds: 1220"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 27101.37960097393,
            "unit": "iter/sec",
            "range": "stddev: 0.0000014554689213091879",
            "extra": "mean: 36.898490583264014 usec\nrounds: 27398"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "s@muelcolvin.com",
            "name": "Samuel Colvin",
            "username": "samuelcolvin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1f0d57fdfd6668009c57dd78566f4f68a8d34456",
          "message": "Dates (#77)\n\n* starting date parsing\r\n\r\n* basic date parsing working\r\n\r\n* fix tests\r\n\r\n* improve coverage\r\n\r\n* datetime parsing working\r\n\r\n* working on datetime fallback for date errors\r\n\r\n* fixing date parsing from datetimes\r\n\r\n* adding datetime benchmarks\r\n\r\n* fix edge case, more tests\r\n\r\n* Dates rust type (#82)\r\n\r\n* using speedate types for all dates\r\n\r\n* using enum\r\n\r\n* more datetime tests\r\n\r\n* speedup by creating Date and DateTime from attributes\r\n\r\n* implementing time types\r\n\r\n* lax datetime tests and datetime from date\r\n\r\n* python types and alter error kinds\r\n\r\n* improving coverage\r\n\r\n* more coverage",
          "timestamp": "2022-06-14T18:28:27+01:00",
          "tree_id": "b2fe98d2114dd410c2cbe5e70d3428e821cc2377",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/1f0d57fdfd6668009c57dd78566f4f68a8d34456"
        },
        "date": 1655227963043,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 44951.408362400754,
            "unit": "iter/sec",
            "range": "stddev: 0.000029479797973792624",
            "extra": "mean: 22.24624403173187 usec\nrounds: 57804"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 25500.4026614016,
            "unit": "iter/sec",
            "range": "stddev: 0.00003624576244899851",
            "extra": "mean: 39.21506704337806 usec\nrounds: 33784"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 223015.27365318066,
            "unit": "iter/sec",
            "range": "stddev: 0.00000549868096340926",
            "extra": "mean: 4.483997816020158 usec\nrounds: 128206"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 820031.0191967003,
            "unit": "iter/sec",
            "range": "stddev: 0.000001246490052311279",
            "extra": "mean: 1.2194660648076434 usec\nrounds: 97088"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 669240.5590327448,
            "unit": "iter/sec",
            "range": "stddev: 0.0000014793451657822216",
            "extra": "mean: 1.494231015294955 usec\nrounds: 90910"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 8143.740795165615,
            "unit": "iter/sec",
            "range": "stddev: 0.00005626431981319232",
            "extra": "mean: 122.79369213146272 usec\nrounds: 10917"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 15729.633997955161,
            "unit": "iter/sec",
            "range": "stddev: 0.00002242866141570268",
            "extra": "mean: 63.57427007710408 usec\nrounds: 19724"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 2996.345627621753,
            "unit": "iter/sec",
            "range": "stddev: 0.00009872145468640298",
            "extra": "mean: 333.73986992071934 usec\nrounds: 4036"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 5535.879539445415,
            "unit": "iter/sec",
            "range": "stddev: 0.00010128785380913604",
            "extra": "mean: 180.63976878011695 usec\nrounds: 7508"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2456.976536114886,
            "unit": "iter/sec",
            "range": "stddev: 0.00011467720129090663",
            "extra": "mean: 407.0042938144041 usec\nrounds: 3104"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 4417.442295899923,
            "unit": "iter/sec",
            "range": "stddev: 0.00007786881858835192",
            "extra": "mean: 226.37533962314717 usec\nrounds: 5777"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1367.0264322280136,
            "unit": "iter/sec",
            "range": "stddev: 0.0001502986379644913",
            "extra": "mean: 731.5147508670884 usec\nrounds: 1730"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1476.6064290894626,
            "unit": "iter/sec",
            "range": "stddev: 0.00017572868234705215",
            "extra": "mean: 677.2285290784233 usec\nrounds: 1943"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1043.1719591552412,
            "unit": "iter/sec",
            "range": "stddev: 0.001538220183021385",
            "extra": "mean: 958.6147242778632 usec\nrounds: 1730"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 693.932431287944,
            "unit": "iter/sec",
            "range": "stddev: 0.0021421707146670197",
            "extra": "mean: 1.4410624938569194 msec\nrounds: 1221"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 28132.962162228625,
            "unit": "iter/sec",
            "range": "stddev: 0.000027210033430449346",
            "extra": "mean: 35.545492658522896 usec\nrounds: 37594"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 910725.2006097652,
            "unit": "iter/sec",
            "range": "stddev: 0.0000015348990111109133",
            "extra": "mean: 1.0980260558623292 usec\nrounds: 106383"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 677885.9405396375,
            "unit": "iter/sec",
            "range": "stddev: 0.0000018684984381216238",
            "extra": "mean: 1.4751744212365938 usec\nrounds: 94340"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 7208150.970974403,
            "unit": "iter/sec",
            "range": "stddev: 1.4747707705914138e-7",
            "extra": "mean: 138.7318334517043 nsec\nrounds: 99010"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2516557.184810979,
            "unit": "iter/sec",
            "range": "stddev: 0.000001515203299405589",
            "extra": "mean: 397.3682799801138 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 977956.1817558334,
            "unit": "iter/sec",
            "range": "stddev: 0.0000013857404973896535",
            "extra": "mean: 1.0225407013682466 usec\nrounds: 126583"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2547902.8890081085,
            "unit": "iter/sec",
            "range": "stddev: 9.297636623643342e-7",
            "extra": "mean: 392.47963661145496 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 7203519.166717624,
            "unit": "iter/sec",
            "range": "stddev: 2.442148310776175e-7",
            "extra": "mean: 138.8210368926597 nsec\nrounds: 89286"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 3904754.0355954147,
            "unit": "iter/sec",
            "range": "stddev: 1.4901339338383852e-7",
            "extra": "mean: 256.0980770835058 nsec\nrounds: 45249"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1550993.544031086,
            "unit": "iter/sec",
            "range": "stddev: 7.205232143637735e-7",
            "extra": "mean: 644.7480093314961 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2432198.248969916,
            "unit": "iter/sec",
            "range": "stddev: 7.085106721808482e-7",
            "extra": "mean: 411.1506948185708 nsec\nrounds: 185186"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 2914079.2096738177,
            "unit": "iter/sec",
            "range": "stddev: 0.0000010265311071243363",
            "extra": "mean: 343.1615711338875 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 3747770.7283358853,
            "unit": "iter/sec",
            "range": "stddev: 1.976300792771448e-7",
            "extra": "mean: 266.82528694705644 nsec\nrounds: 48075"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "dharmasw@outlook.com",
            "name": "dswij",
            "username": "dswij"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2b46ec563945e526541d95c131e92932cbf038df",
          "message": "Add `bytes` type validator (#80)\n\n* init bytes type\r\n\r\n* single quote lint\r\n\r\n* Remove int and float coercion to bytes\r\n\r\n* Finish tests\r\n\r\n* fix json string test\r\n\r\n* Remove config setting for `BytesValidator`\r\n\r\n* Add `bytes` case to `test_typing`\r\n\r\n* Add benchmark for bytes type\r\n\r\n* use slice for validation logic\r\n\r\n* using enum for bytes\r\n\r\n* use IntoPy\r\n\r\nCo-authored-by: Samuel Colvin <s@muelcolvin.com>",
          "timestamp": "2022-06-14T20:58:04+01:00",
          "tree_id": "f8b973b16ef405a602b606c10ee564ac1c4a3a3b",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/2b46ec563945e526541d95c131e92932cbf038df"
        },
        "date": 1655236804325,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 55908.00536143587,
            "unit": "iter/sec",
            "range": "stddev: 0.000002761368336719595",
            "extra": "mean: 17.886526151937776 usec\nrounds: 57472"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 34140.52101934881,
            "unit": "iter/sec",
            "range": "stddev: 7.075140600490311e-7",
            "extra": "mean: 29.290707058432403 usec\nrounds: 34966"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 260578.8809894506,
            "unit": "iter/sec",
            "range": "stddev: 1.7625084532100696e-7",
            "extra": "mean: 3.837609541505724 usec\nrounds: 135136"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 928995.0815584206,
            "unit": "iter/sec",
            "range": "stddev: 3.947602553279822e-8",
            "extra": "mean: 1.0764319637973827 usec\nrounds: 95239"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 797988.0358731027,
            "unit": "iter/sec",
            "range": "stddev: 4.4192983818854214e-8",
            "extra": "mean: 1.2531516201318327 usec\nrounds: 81968"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 9981.128295067549,
            "unit": "iter/sec",
            "range": "stddev: 0.00000515169941088928",
            "extra": "mean: 100.18907386394159 usec\nrounds: 10560"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 19374.725748471148,
            "unit": "iter/sec",
            "range": "stddev: 0.000003961967751069529",
            "extra": "mean: 51.61363381254104 usec\nrounds: 20921"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3795.3488446080582,
            "unit": "iter/sec",
            "range": "stddev: 0.0000020120988484837086",
            "extra": "mean: 263.48039164322694 usec\nrounds: 3853"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6752.745066799651,
            "unit": "iter/sec",
            "range": "stddev: 0.0000017608101936053257",
            "extra": "mean: 148.0879242601014 usec\nrounds: 6826"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 3047.920087597972,
            "unit": "iter/sec",
            "range": "stddev: 0.0000021964883013783746",
            "extra": "mean: 328.09259142620357 usec\nrounds: 3079"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5369.085361702856,
            "unit": "iter/sec",
            "range": "stddev: 0.0000017409258694767312",
            "extra": "mean: 186.25146233153583 usec\nrounds: 5429"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1553.9542077963158,
            "unit": "iter/sec",
            "range": "stddev: 0.000003888497158823085",
            "extra": "mean: 643.5196062940065 usec\nrounds: 1557"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1868.7413954459792,
            "unit": "iter/sec",
            "range": "stddev: 0.0000037168765276226506",
            "extra": "mean: 535.1195207838524 usec\nrounds: 1684"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1272.0619440895748,
            "unit": "iter/sec",
            "range": "stddev: 0.001424342147400412",
            "extra": "mean: 786.1252391413283 usec\nrounds: 1819"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 889.0118583472067,
            "unit": "iter/sec",
            "range": "stddev: 0.0018575473630782497",
            "extra": "mean: 1.1248443883067378 msec\nrounds: 1334"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 32530.483003902053,
            "unit": "iter/sec",
            "range": "stddev: 0.000006033329287272015",
            "extra": "mean: 30.74039816377916 usec\nrounds: 33004"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 9603721.450682122,
            "unit": "iter/sec",
            "range": "stddev: 4.254317722547356e-9",
            "extra": "mean: 104.12630198976201 nsec\nrounds: 99010"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1103757.73505572,
            "unit": "iter/sec",
            "range": "stddev: 5.342671668282513e-8",
            "extra": "mean: 905.995915805805 nsec\nrounds: 112360"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 759140.4059324171,
            "unit": "iter/sec",
            "range": "stddev: 4.487090271157516e-8",
            "extra": "mean: 1.3172793757063248 usec\nrounds: 76342"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 8888042.58184235,
            "unit": "iter/sec",
            "range": "stddev: 3.748733068424146e-9",
            "extra": "mean: 112.51071209354647 nsec\nrounds: 89286"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2752157.4676707764,
            "unit": "iter/sec",
            "range": "stddev: 2.123816169095272e-8",
            "extra": "mean: 363.3513022956639 nsec\nrounds: 185186"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 1093790.4872649496,
            "unit": "iter/sec",
            "range": "stddev: 7.480481357781747e-8",
            "extra": "mean: 914.251871489903 nsec\nrounds: 105264"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2633781.134181071,
            "unit": "iter/sec",
            "range": "stddev: 2.793412336528609e-8",
            "extra": "mean: 379.68227011061396 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 8985337.949797373,
            "unit": "iter/sec",
            "range": "stddev: 4.8490241997716985e-9",
            "extra": "mean: 111.29241944909953 nsec\nrounds: 91744"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 4528653.665170486,
            "unit": "iter/sec",
            "range": "stddev: 5.842717727014262e-9",
            "extra": "mean: 220.8161793628617 nsec\nrounds: 45249"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1784931.453420881,
            "unit": "iter/sec",
            "range": "stddev: 3.700340768884946e-8",
            "extra": "mean: 560.2456038768779 nsec\nrounds: 185186"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2765328.633991798,
            "unit": "iter/sec",
            "range": "stddev: 1.9331573602642435e-8",
            "extra": "mean: 361.6206723889814 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 3417624.5892452653,
            "unit": "iter/sec",
            "range": "stddev: 1.50478137514685e-8",
            "extra": "mean: 292.6008910244365 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 4476375.109031075,
            "unit": "iter/sec",
            "range": "stddev: 6.3346250107729105e-9",
            "extra": "mean: 223.39504077367326 nsec\nrounds: 45249"
          }
        ]
      },
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
          "id": "b551729922df2187d36c756505e9befe7c7fdbba",
          "message": "add basic string benchmark",
          "timestamp": "2022-06-14T21:37:13+01:00",
          "tree_id": "a17936782c19ee131f0ea3086ef299f235721fdc",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/b551729922df2187d36c756505e9befe7c7fdbba"
        },
        "date": 1655239171508,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 51769.41327059836,
            "unit": "iter/sec",
            "range": "stddev: 0.000015175247882486426",
            "extra": "mean: 19.31642521758953 usec\nrounds: 54946"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 30679.2056392186,
            "unit": "iter/sec",
            "range": "stddev: 0.00001920624233161118",
            "extra": "mean: 32.59536807307864 usec\nrounds: 33113"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 232498.3344354321,
            "unit": "iter/sec",
            "range": "stddev: 0.000005701217049116291",
            "extra": "mean: 4.3011060807307135 usec\nrounds: 131579"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 932160.0099341867,
            "unit": "iter/sec",
            "range": "stddev: 9.96912036184853e-7",
            "extra": "mean: 1.072777194196857 usec\nrounds: 94340"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 4580258.1013105335,
            "unit": "iter/sec",
            "range": "stddev: 5.132935920884571e-8",
            "extra": "mean: 218.32830768063803 nsec\nrounds: 41842"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 846120.8687381326,
            "unit": "iter/sec",
            "range": "stddev: 4.268643632792098e-7",
            "extra": "mean: 1.1818642429789012 usec\nrounds: 89286"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 9076.661730958873,
            "unit": "iter/sec",
            "range": "stddev: 0.000051501967524699586",
            "extra": "mean: 110.17266365553522 usec\nrounds: 9853"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 18253.58543623469,
            "unit": "iter/sec",
            "range": "stddev: 0.00004403230397974539",
            "extra": "mean: 54.78375760714536 usec\nrounds: 20409"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3286.9362300293633,
            "unit": "iter/sec",
            "range": "stddev: 0.00015378419062271774",
            "extra": "mean: 304.23468239633803 usec\nrounds: 3539"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6358.333732903298,
            "unit": "iter/sec",
            "range": "stddev: 0.00003382095455050798",
            "extra": "mean: 157.27390885841203 usec\nrounds: 6649"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2696.96372172031,
            "unit": "iter/sec",
            "range": "stddev: 0.000039975985627828996",
            "extra": "mean: 370.7873383488195 usec\nrounds: 2858"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 4815.383262123888,
            "unit": "iter/sec",
            "range": "stddev: 0.00002965654613328545",
            "extra": "mean: 207.66778998997825 usec\nrounds: 5095"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1390.021978669138,
            "unit": "iter/sec",
            "range": "stddev: 0.00021363977766459622",
            "extra": "mean: 719.4130850775753 usec\nrounds: 1481"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1566.378027871163,
            "unit": "iter/sec",
            "range": "stddev: 0.00018503885415610874",
            "extra": "mean: 638.4154924332554 usec\nrounds: 1718"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1237.7433532389593,
            "unit": "iter/sec",
            "range": "stddev: 0.0014121413943261219",
            "extra": "mean: 807.9219309747643 usec\nrounds: 1753"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 871.1735346686917,
            "unit": "iter/sec",
            "range": "stddev: 0.0019324942763609767",
            "extra": "mean: 1.1478769271615916 msec\nrounds: 1318"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 29036.772699658508,
            "unit": "iter/sec",
            "range": "stddev: 0.00003208198197804892",
            "extra": "mean: 34.439089024923234 usec\nrounds: 30396"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 7964734.164328085,
            "unit": "iter/sec",
            "range": "stddev: 2.9359414094971765e-7",
            "extra": "mean: 125.55346849854952 nsec\nrounds: 78126"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1153046.559814882,
            "unit": "iter/sec",
            "range": "stddev: 4.1143375064119983e-7",
            "extra": "mean: 867.2676671104142 nsec\nrounds: 117648"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 823556.8094687536,
            "unit": "iter/sec",
            "range": "stddev: 0.0000014953665935230568",
            "extra": "mean: 1.2142453179943749 usec\nrounds: 88488"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 8199350.318573936,
            "unit": "iter/sec",
            "range": "stddev: 8.795041023797243e-8",
            "extra": "mean: 121.96088240490174 nsec\nrounds: 85471"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2832197.8350074114,
            "unit": "iter/sec",
            "range": "stddev: 4.086346091478673e-7",
            "extra": "mean: 353.08267933810606 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 1052576.9691920548,
            "unit": "iter/sec",
            "range": "stddev: 7.280896075376725e-7",
            "extra": "mean: 950.0492878609228 nsec\nrounds: 120482"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2782902.852958954,
            "unit": "iter/sec",
            "range": "stddev: 1.3838034503268624e-7",
            "extra": "mean: 359.33701348472584 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 7769689.957872687,
            "unit": "iter/sec",
            "range": "stddev: 1.066707401930174e-7",
            "extra": "mean: 128.7052643568983 nsec\nrounds: 76336"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 4067114.946789692,
            "unit": "iter/sec",
            "range": "stddev: 2.1882022813943772e-7",
            "extra": "mean: 245.8745359014931 nsec\nrounds: 44053"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1660830.5807344061,
            "unit": "iter/sec",
            "range": "stddev: 7.669255270646515e-7",
            "extra": "mean: 602.1083737256628 nsec\nrounds: 181819"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2624802.5686123767,
            "unit": "iter/sec",
            "range": "stddev: 4.3660540059213864e-7",
            "extra": "mean: 380.9810352820438 nsec\nrounds: 185186"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 3144600.0633962727,
            "unit": "iter/sec",
            "range": "stddev: 8.342807369472395e-7",
            "extra": "mean: 318.0054632833735 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 4051006.2352373535,
            "unit": "iter/sec",
            "range": "stddev: 4.027575685700822e-7",
            "extra": "mean: 246.85224902925248 nsec\nrounds: 37594"
          }
        ]
      },
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
          "id": "4c062dc352cce5dab43dcd5e37120453985fbb1d",
          "message": "linting",
          "timestamp": "2022-06-15T10:49:21+01:00",
          "tree_id": "8b86c25fc31abad45c7a30cdd704a04e15ed463d",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/4c062dc352cce5dab43dcd5e37120453985fbb1d"
        },
        "date": 1655286782115,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 56419.447091023016,
            "unit": "iter/sec",
            "range": "stddev: 4.91905111010753e-7",
            "extra": "mean: 17.724384969364785 usec\nrounds: 58480"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 33986.87409607218,
            "unit": "iter/sec",
            "range": "stddev: 0.0000038583401446571475",
            "extra": "mean: 29.42312367925501 usec\nrounds: 35212"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 259147.85206880092,
            "unit": "iter/sec",
            "range": "stddev: 1.5187115831435967e-7",
            "extra": "mean: 3.8588010358446296 usec\nrounds: 131579"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 949537.7585320113,
            "unit": "iter/sec",
            "range": "stddev: 3.525978393147724e-8",
            "extra": "mean: 1.0531440071917129 usec\nrounds: 95239"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 5087587.305746694,
            "unit": "iter/sec",
            "range": "stddev: 5.530593257194579e-9",
            "extra": "mean: 196.55682348108482 nsec\nrounds: 51547"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 775517.6519292436,
            "unit": "iter/sec",
            "range": "stddev: 3.326841670883074e-7",
            "extra": "mean: 1.289461300477658 usec\nrounds: 80001"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 10036.104122289888,
            "unit": "iter/sec",
            "range": "stddev: 0.000005057078012318647",
            "extra": "mean: 99.64025759547769 usec\nrounds: 10730"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 18063.279579660975,
            "unit": "iter/sec",
            "range": "stddev: 0.000002812117338519014",
            "extra": "mean: 55.36093241484163 usec\nrounds: 20833"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3812.6397693156264,
            "unit": "iter/sec",
            "range": "stddev: 0.0000019502406644598124",
            "extra": "mean: 262.28546637111253 usec\nrounds: 3836"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6735.999058942983,
            "unit": "iter/sec",
            "range": "stddev: 0.0000015624098454054286",
            "extra": "mean: 148.4560777472734 usec\nrounds: 6817"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 3071.334385415889,
            "unit": "iter/sec",
            "range": "stddev: 0.000002251373555757314",
            "extra": "mean: 325.59137967798654 usec\nrounds: 3100"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5323.732229530065,
            "unit": "iter/sec",
            "range": "stddev: 0.000001578451938295082",
            "extra": "mean: 187.83814754114178 usec\nrounds: 5368"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1585.099708818505,
            "unit": "iter/sec",
            "range": "stddev: 0.000005203052988202751",
            "extra": "mean: 630.8751395490293 usec\nrounds: 1598"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1866.34223695924,
            "unit": "iter/sec",
            "range": "stddev: 0.0000030516950913894454",
            "extra": "mean: 535.8074099149477 usec\nrounds: 1876"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1286.6682557194795,
            "unit": "iter/sec",
            "range": "stddev: 0.0014008815913797653",
            "extra": "mean: 777.2011126837197 usec\nrounds: 1837"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 892.948513946424,
            "unit": "iter/sec",
            "range": "stddev: 0.001956229079280775",
            "extra": "mean: 1.1198853958336943 msec\nrounds: 1344"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 33132.682099314836,
            "unit": "iter/sec",
            "range": "stddev: 6.840163317028668e-7",
            "extra": "mean: 30.181679738528604 usec\nrounds: 34247"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 9526607.134311978,
            "unit": "iter/sec",
            "range": "stddev: 3.5108011202331664e-9",
            "extra": "mean: 104.96916540182367 nsec\nrounds: 96154"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1065758.542188734,
            "unit": "iter/sec",
            "range": "stddev: 4.320743144746741e-8",
            "extra": "mean: 938.2988363819621 nsec\nrounds: 105275"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 730099.6788580811,
            "unit": "iter/sec",
            "range": "stddev: 9.364735322912825e-8",
            "extra": "mean: 1.3696759893992903 usec\nrounds: 73530"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 7903256.672607624,
            "unit": "iter/sec",
            "range": "stddev: 3.79745130868683e-9",
            "extra": "mean: 126.53011807973353 nsec\nrounds: 81301"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2650824.7610715823,
            "unit": "iter/sec",
            "range": "stddev: 1.812102808805782e-8",
            "extra": "mean: 377.24108160040595 nsec\nrounds: 192271"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 951787.5578817505,
            "unit": "iter/sec",
            "range": "stddev: 4.82932396136862e-8",
            "extra": "mean: 1.0506546253095428 usec\nrounds: 112360"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2237583.8860920114,
            "unit": "iter/sec",
            "range": "stddev: 2.724415481548168e-8",
            "extra": "mean: 446.91061918014265 nsec\nrounds: 185186"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 9109271.942393636,
            "unit": "iter/sec",
            "range": "stddev: 3.7169151889489023e-9",
            "extra": "mean: 109.77825739799142 nsec\nrounds: 90910"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 4589221.951515126,
            "unit": "iter/sec",
            "range": "stddev: 4.996427598464531e-9",
            "extra": "mean: 217.90186017701905 nsec\nrounds: 45872"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1749566.9931377617,
            "unit": "iter/sec",
            "range": "stddev: 3.27041235432484e-8",
            "extra": "mean: 571.5699964176675 nsec\nrounds: 178572"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2773594.704153626,
            "unit": "iter/sec",
            "range": "stddev: 1.6749476787089845e-8",
            "extra": "mean: 360.5429439647809 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 2881789.4459652025,
            "unit": "iter/sec",
            "range": "stddev: 1.77001029225182e-8",
            "extra": "mean: 347.00661472676643 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 4532338.633318441,
            "unit": "iter/sec",
            "range": "stddev: 5.251575158791685e-9",
            "extra": "mean: 220.6366471934108 nsec\nrounds: 45872"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "s@muelcolvin.com",
            "name": "Samuel Colvin",
            "username": "samuelcolvin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a76e783249fd25330fc96b72bd012860eebca1a4",
          "message": "Add rust bench marks (#87)\n\n* rust benchmarks\r\n\r\n* tweaking model dict creation\r\n\r\n* run the rust bencharmks on CI",
          "timestamp": "2022-06-15T15:34:09+01:00",
          "tree_id": "1b43fc63dda0ddca9d82dbd91a414675bf8005f7",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/a76e783249fd25330fc96b72bd012860eebca1a4"
        },
        "date": 1655303809791,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 51336.21540904401,
            "unit": "iter/sec",
            "range": "stddev: 5.630299437091309e-7",
            "extra": "mean: 19.479425821168498 usec\nrounds: 53189"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 34806.93993268898,
            "unit": "iter/sec",
            "range": "stddev: 0.00003624601366032269",
            "extra": "mean: 28.729902770362436 usec\nrounds: 35843"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 554461.5116243543,
            "unit": "iter/sec",
            "range": "stddev: 7.342931482114175e-8",
            "extra": "mean: 1.8035516966189833 usec\nrounds: 57143"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1241435.3779105393,
            "unit": "iter/sec",
            "range": "stddev: 4.74057391146352e-8",
            "extra": "mean: 805.5191738483685 nsec\nrounds: 128206"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 4644057.447576591,
            "unit": "iter/sec",
            "range": "stddev: 4.58286117732519e-9",
            "extra": "mean: 215.32894700124567 nsec\nrounds: 47170"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 1009488.3561613795,
            "unit": "iter/sec",
            "range": "stddev: 4.344795625389642e-8",
            "extra": "mean: 990.6008265442987 nsec\nrounds: 104167"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 14123.45022318781,
            "unit": "iter/sec",
            "range": "stddev: 0.0000033296241897646673",
            "extra": "mean: 70.80422872579712 usec\nrounds: 14266"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 28245.042975987883,
            "unit": "iter/sec",
            "range": "stddev: 8.534621005512962e-7",
            "extra": "mean: 35.40444250165013 usec\nrounds: 28653"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3473.5451660931785,
            "unit": "iter/sec",
            "range": "stddev: 0.0000016803927682459187",
            "extra": "mean: 287.8903115357317 usec\nrounds: 3502"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6591.4557931598865,
            "unit": "iter/sec",
            "range": "stddev: 0.0000015965713298341313",
            "extra": "mean: 151.7115537720399 usec\nrounds: 6667"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2885.1302745635835,
            "unit": "iter/sec",
            "range": "stddev: 0.0000022249113944773913",
            "extra": "mean: 346.60479937990465 usec\nrounds: 2901"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5176.320085091475,
            "unit": "iter/sec",
            "range": "stddev: 0.0000024020382939926764",
            "extra": "mean: 193.187435004288 usec\nrounds: 5108"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1509.1425580337059,
            "unit": "iter/sec",
            "range": "stddev: 0.00000734850160202607",
            "extra": "mean: 662.6279238343934 usec\nrounds: 1523"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1771.7540483060016,
            "unit": "iter/sec",
            "range": "stddev: 0.0000036951270271263467",
            "extra": "mean: 564.4124256163624 usec\nrounds: 1788"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1564.5373769152272,
            "unit": "iter/sec",
            "range": "stddev: 0.0016816326355398187",
            "extra": "mean: 639.1665771332889 usec\nrounds: 2379"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 1023.6088298456068,
            "unit": "iter/sec",
            "range": "stddev: 0.0020695773241903533",
            "extra": "mean: 976.9356914895236 usec\nrounds: 1598"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 27692.65338433483,
            "unit": "iter/sec",
            "range": "stddev: 8.270537429410865e-7",
            "extra": "mean: 36.11066033006717 usec\nrounds: 28248"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 8161814.324260263,
            "unit": "iter/sec",
            "range": "stddev: 3.231789983387325e-9",
            "extra": "mean: 122.52177766739251 nsec\nrounds: 83334"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1284924.082933113,
            "unit": "iter/sec",
            "range": "stddev: 3.408704821048778e-8",
            "extra": "mean: 778.2560956570377 nsec\nrounds: 129871"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 924694.3950427432,
            "unit": "iter/sec",
            "range": "stddev: 3.909451064422613e-8",
            "extra": "mean: 1.0814383707321338 usec\nrounds: 96154"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 8032034.703662466,
            "unit": "iter/sec",
            "range": "stddev: 3.426109477518152e-9",
            "extra": "mean: 124.50145410145383 nsec\nrounds: 81301"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2877434.6948560122,
            "unit": "iter/sec",
            "range": "stddev: 1.6394297066452894e-8",
            "extra": "mean: 347.5317795353298 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 1149054.3833069156,
            "unit": "iter/sec",
            "range": "stddev: 4.100195578466945e-8",
            "extra": "mean: 870.2808278945513 nsec\nrounds: 119048"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2720919.296554317,
            "unit": "iter/sec",
            "range": "stddev: 1.7536870298099354e-8",
            "extra": "mean: 367.52284467483173 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 7388235.176528162,
            "unit": "iter/sec",
            "range": "stddev: 3.33039402235875e-9",
            "extra": "mean: 135.3503206256345 nsec\nrounds: 75758"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 4123705.233029786,
            "unit": "iter/sec",
            "range": "stddev: 2.1326071773391066e-7",
            "extra": "mean: 242.50035914068283 nsec\nrounds: 42017"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1653570.1802693803,
            "unit": "iter/sec",
            "range": "stddev: 3.911804335989332e-8",
            "extra": "mean: 604.7520764050872 nsec\nrounds: 172414"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2445228.481419185,
            "unit": "iter/sec",
            "range": "stddev: 9.747404557145793e-7",
            "extra": "mean: 408.9597383634339 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 3184912.594904888,
            "unit": "iter/sec",
            "range": "stddev: 2.607924691628572e-8",
            "extra": "mean: 313.9803590204424 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 3931575.21920986,
            "unit": "iter/sec",
            "range": "stddev: 1.207308174926975e-8",
            "extra": "mean: 254.3509774694697 nsec\nrounds: 188680"
          }
        ]
      },
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
          "id": "9f8570a85029dd89547d4b384ae3331749389f53",
          "message": "remove bounds checks from recursive validators",
          "timestamp": "2022-06-15T18:02:57+01:00",
          "tree_id": "03911af4a3918b1a5d6078590559bd0822a12224",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/9f8570a85029dd89547d4b384ae3331749389f53"
        },
        "date": 1655312763874,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 54302.9189706591,
            "unit": "iter/sec",
            "range": "stddev: 6.157963383097183e-7",
            "extra": "mean: 18.415216326406302 usec\nrounds: 56498"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 34210.1144137315,
            "unit": "iter/sec",
            "range": "stddev: 8.295802510426481e-7",
            "extra": "mean: 29.231121179723765 usec\nrounds: 35336"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 569858.2200037848,
            "unit": "iter/sec",
            "range": "stddev: 6.355678686428327e-8",
            "extra": "mean: 1.754822453896246 usec\nrounds: 56498"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1174825.9979493194,
            "unit": "iter/sec",
            "range": "stddev: 3.856258160391826e-8",
            "extra": "mean: 851.1898798163131 nsec\nrounds: 120482"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 4744265.522753339,
            "unit": "iter/sec",
            "range": "stddev: 6.2568532586496256e-9",
            "extra": "mean: 210.7807826530035 nsec\nrounds: 48310"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 941028.9643229999,
            "unit": "iter/sec",
            "range": "stddev: 4.621322877345621e-8",
            "extra": "mean: 1.06266654684711 usec\nrounds: 97088"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 13662.929138252062,
            "unit": "iter/sec",
            "range": "stddev: 0.0000012798370369689607",
            "extra": "mean: 73.19074774385697 usec\nrounds: 13851"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 26026.88406064439,
            "unit": "iter/sec",
            "range": "stddev: 9.73789866625618e-7",
            "extra": "mean: 38.42181022015286 usec\nrounds: 26810"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3546.3281504688257,
            "unit": "iter/sec",
            "range": "stddev: 0.0000024773264780459415",
            "extra": "mean: 281.98180133663027 usec\nrounds: 3589"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6504.639879184889,
            "unit": "iter/sec",
            "range": "stddev: 0.000001820686986229424",
            "extra": "mean: 153.73641255683356 usec\nrounds: 6610"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2910.3010821060307,
            "unit": "iter/sec",
            "range": "stddev: 0.0000028379722320734715",
            "extra": "mean: 343.60706050260376 usec\nrounds: 2942"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5116.127362154299,
            "unit": "iter/sec",
            "range": "stddev: 0.0000022358521425942536",
            "extra": "mean: 195.46034123335824 usec\nrounds: 5190"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1447.4034930926762,
            "unit": "iter/sec",
            "range": "stddev: 0.000004538729766452413",
            "extra": "mean: 690.8923494880434 usec\nrounds: 1465"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1811.078572461979,
            "unit": "iter/sec",
            "range": "stddev: 0.000004847368534420569",
            "extra": "mean: 552.1571593885078 usec\nrounds: 1832"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1370.9879808830967,
            "unit": "iter/sec",
            "range": "stddev: 0.002216530839363131",
            "extra": "mean: 729.4009969043408 usec\nrounds: 2261"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 884.5657491891639,
            "unit": "iter/sec",
            "range": "stddev: 0.0028162969719980517",
            "extra": "mean: 1.1304982144251559 msec\nrounds: 1539"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 29585.413760579613,
            "unit": "iter/sec",
            "range": "stddev: 8.684898265422686e-7",
            "extra": "mean: 33.8004399090888 usec\nrounds: 30304"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 8926874.794400148,
            "unit": "iter/sec",
            "range": "stddev: 5.07255075375532e-9",
            "extra": "mean: 112.02128662409648 nsec\nrounds: 91744"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1225471.0620623166,
            "unit": "iter/sec",
            "range": "stddev: 4.035388985558546e-8",
            "extra": "mean: 816.0127406988754 nsec\nrounds: 125001"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 771903.281810523,
            "unit": "iter/sec",
            "range": "stddev: 5.2059837155840234e-8",
            "extra": "mean: 1.2954990910964965 usec\nrounds: 80646"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 8675640.068061715,
            "unit": "iter/sec",
            "range": "stddev: 4.820339656793053e-9",
            "extra": "mean: 115.2652705915853 nsec\nrounds: 89286"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2393828.998449791,
            "unit": "iter/sec",
            "range": "stddev: 3.331022245690731e-8",
            "extra": "mean: 417.7407829248868 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 954152.5681405844,
            "unit": "iter/sec",
            "range": "stddev: 5.091622097769235e-8",
            "extra": "mean: 1.0480504202266439 usec\nrounds: 98040"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2295564.2743630256,
            "unit": "iter/sec",
            "range": "stddev: 2.5521230460622298e-8",
            "extra": "mean: 435.62274041666643 nsec\nrounds: 200000"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 7708858.062014087,
            "unit": "iter/sec",
            "range": "stddev: 4.8436315068262636e-9",
            "extra": "mean: 129.72089925066243 nsec\nrounds: 79366"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 3701500.5716486042,
            "unit": "iter/sec",
            "range": "stddev: 1.986168240179439e-8",
            "extra": "mean: 270.16070392078495 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1560246.2752789778,
            "unit": "iter/sec",
            "range": "stddev: 4.577678805949464e-8",
            "extra": "mean: 640.924459070728 nsec\nrounds: 163935"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2200455.085298168,
            "unit": "iter/sec",
            "range": "stddev: 2.750439151391651e-8",
            "extra": "mean: 454.4514481032227 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 3003427.3656147514,
            "unit": "iter/sec",
            "range": "stddev: 1.988842307931578e-8",
            "extra": "mean: 332.9529495031809 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 3642970.419505216,
            "unit": "iter/sec",
            "range": "stddev: 3.096722097782345e-8",
            "extra": "mean: 274.5012681532403 nsec\nrounds: 196079"
          }
        ]
      },
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
          "id": "17d4a54f03b39968cc7ed2bf462325bf44a4df41",
          "message": "remove bounds checks from recursive validators",
          "timestamp": "2022-06-15T18:04:55+01:00",
          "tree_id": "30e6058e7f23c8e34a03f7de165fdc297ffa805c",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/17d4a54f03b39968cc7ed2bf462325bf44a4df41"
        },
        "date": 1655312851296,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 54473.830666907044,
            "unit": "iter/sec",
            "range": "stddev: 5.146115475966343e-7",
            "extra": "mean: 18.357438567423934 usec\nrounds: 55866"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 34248.94661979434,
            "unit": "iter/sec",
            "range": "stddev: 6.051937465959462e-7",
            "extra": "mean: 29.1979782940841 usec\nrounds: 35336"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 570405.0743766411,
            "unit": "iter/sec",
            "range": "stddev: 4.467585889118238e-8",
            "extra": "mean: 1.7531400839882532 usec\nrounds: 58824"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1170479.3799462316,
            "unit": "iter/sec",
            "range": "stddev: 3.600559011670169e-8",
            "extra": "mean: 854.3508045785269 nsec\nrounds: 120497"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 4766722.245692469,
            "unit": "iter/sec",
            "range": "stddev: 5.089416300554276e-9",
            "extra": "mean: 209.78776367830196 nsec\nrounds: 48544"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 940865.554438489,
            "unit": "iter/sec",
            "range": "stddev: 3.8582668744839027e-8",
            "extra": "mean: 1.0628511111739811 usec\nrounds: 98049"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 13541.375428492942,
            "unit": "iter/sec",
            "range": "stddev: 0.0000018750451464365071",
            "extra": "mean: 73.84774207617497 usec\nrounds: 13756"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 25852.18748730651,
            "unit": "iter/sec",
            "range": "stddev: 9.180560856079898e-7",
            "extra": "mean: 38.68144622156259 usec\nrounds: 26386"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3536.0896897140674,
            "unit": "iter/sec",
            "range": "stddev: 0.0000021634191047964174",
            "extra": "mean: 282.7982567605239 usec\nrounds: 3587"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6478.631078736878,
            "unit": "iter/sec",
            "range": "stddev: 0.0000014084656423586937",
            "extra": "mean: 154.35359535782788 usec\nrounds: 6549"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2893.6833558284557,
            "unit": "iter/sec",
            "range": "stddev: 0.0000023353496027157518",
            "extra": "mean: 345.5803130587182 usec\nrounds: 2910"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5139.092931979906,
            "unit": "iter/sec",
            "range": "stddev: 0.0000017441128960701855",
            "extra": "mean: 194.58686839016477 usec\nrounds: 5182"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1456.3307375197403,
            "unit": "iter/sec",
            "range": "stddev: 0.0000034585709639550195",
            "extra": "mean: 686.6572092703943 usec\nrounds: 1467"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1812.9938737880545,
            "unit": "iter/sec",
            "range": "stddev: 0.000004533614167364163",
            "extra": "mean: 551.5738439372707 usec\nrounds: 1839"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1651.484179884104,
            "unit": "iter/sec",
            "range": "stddev: 0.0011782694368788688",
            "extra": "mean: 605.5159426778021 usec\nrounds: 2233"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 1074.6476187898409,
            "unit": "iter/sec",
            "range": "stddev: 0.0015597044841624717",
            "extra": "mean: 930.5375850793756 usec\nrounds: 1528"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 29447.429089483972,
            "unit": "iter/sec",
            "range": "stddev: 6.791422619092252e-7",
            "extra": "mean: 33.95882190466372 usec\nrounds: 30304"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 8906143.873869829,
            "unit": "iter/sec",
            "range": "stddev: 3.574124877431792e-9",
            "extra": "mean: 112.28203969787742 nsec\nrounds: 91744"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1240836.6662055613,
            "unit": "iter/sec",
            "range": "stddev: 2.7632104031389842e-8",
            "extra": "mean: 805.9078420517195 nsec\nrounds: 128206"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 764210.8252277699,
            "unit": "iter/sec",
            "range": "stddev: 4.361921991648289e-8",
            "extra": "mean: 1.3085394330838522 usec\nrounds: 78741"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 8661471.980266096,
            "unit": "iter/sec",
            "range": "stddev: 3.378624130958621e-9",
            "extra": "mean: 115.45381688917244 nsec\nrounds: 88504"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2557332.1473032716,
            "unit": "iter/sec",
            "range": "stddev: 1.7493103767613652e-8",
            "extra": "mean: 391.0325066906021 nsec\nrounds: 188715"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 952606.982547436,
            "unit": "iter/sec",
            "range": "stddev: 4.2473176221293395e-8",
            "extra": "mean: 1.0497508608701414 usec\nrounds: 98040"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2480049.5397761855,
            "unit": "iter/sec",
            "range": "stddev: 2.1338668547210204e-8",
            "extra": "mean: 403.21775188832424 nsec\nrounds: 196117"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 7674667.3782583,
            "unit": "iter/sec",
            "range": "stddev: 4.113608353402918e-9",
            "extra": "mean: 130.2988065428369 nsec\nrounds: 79366"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 3684403.3510326473,
            "unit": "iter/sec",
            "range": "stddev: 1.2993146494821095e-8",
            "extra": "mean: 271.4143661061357 nsec\nrounds: 188715"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1559378.7172248217,
            "unit": "iter/sec",
            "range": "stddev: 3.787671146175233e-8",
            "extra": "mean: 641.2810364501634 nsec\nrounds: 163962"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2212366.182450746,
            "unit": "iter/sec",
            "range": "stddev: 2.326229887266129e-8",
            "extra": "mean: 452.0047395101264 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 3013473.833042231,
            "unit": "iter/sec",
            "range": "stddev: 1.5356661624730637e-8",
            "extra": "mean: 331.84293456779653 nsec\nrounds: 192345"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 3637241.311390926,
            "unit": "iter/sec",
            "range": "stddev: 1.2301784894628778e-8",
            "extra": "mean: 274.9336418419471 nsec\nrounds: 200000"
          }
        ]
      },
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
          "id": "bbf54fbef3da20c7e9c1f5c499c35abd96b1f1e9",
          "message": "move unboxing outside loop for dicts",
          "timestamp": "2022-06-15T22:50:46+01:00",
          "tree_id": "f5130492f4be41c925c2acf0db42a79c3ce61e14",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/bbf54fbef3da20c7e9c1f5c499c35abd96b1f1e9"
        },
        "date": 1655329995695,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 52573.702724830335,
            "unit": "iter/sec",
            "range": "stddev: 0.000003456257857975183",
            "extra": "mean: 19.0209163169271 usec\nrounds: 64099"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 36000.12350941187,
            "unit": "iter/sec",
            "range": "stddev: 0.000004826973555851521",
            "extra": "mean: 27.777682477632613 usec\nrounds: 42734"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 557086.185376222,
            "unit": "iter/sec",
            "range": "stddev: 3.3319191062687074e-7",
            "extra": "mean: 1.7950543852112355 usec\nrounds: 65356"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1321513.5770344744,
            "unit": "iter/sec",
            "range": "stddev: 1.7706834540316501e-7",
            "extra": "mean: 756.7080788108202 nsec\nrounds: 163935"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 4468809.777655313,
            "unit": "iter/sec",
            "range": "stddev: 3.9841830408140926e-8",
            "extra": "mean: 223.77323040246722 nsec\nrounds: 55863"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 990805.4580156873,
            "unit": "iter/sec",
            "range": "stddev: 2.1917748968402668e-7",
            "extra": "mean: 1.0092798661028712 usec\nrounds: 119048"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 14030.808361431082,
            "unit": "iter/sec",
            "range": "stddev: 0.00001032846572611284",
            "extra": "mean: 71.27173105356306 usec\nrounds: 17985"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 27204.13834126991,
            "unit": "iter/sec",
            "range": "stddev: 0.000005596108740970845",
            "extra": "mean: 36.75911317076912 usec\nrounds: 34364"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3249.079129769623,
            "unit": "iter/sec",
            "range": "stddev: 0.00003454372383002721",
            "extra": "mean: 307.77951538253404 usec\nrounds: 4193"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6419.528904215998,
            "unit": "iter/sec",
            "range": "stddev: 0.000022172154321252704",
            "extra": "mean: 155.7746705281215 usec\nrounds: 7855"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2907.343548107511,
            "unit": "iter/sec",
            "range": "stddev: 0.000048219436056715855",
            "extra": "mean: 343.9565993674652 usec\nrounds: 3477"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5263.473538227345,
            "unit": "iter/sec",
            "range": "stddev: 0.00002769417566710107",
            "extra": "mean: 189.98860595331962 usec\nrounds: 6215"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1527.906942386949,
            "unit": "iter/sec",
            "range": "stddev: 0.00009170135782005476",
            "extra": "mean: 654.4901212620747 usec\nrounds: 1806"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1783.6814125513827,
            "unit": "iter/sec",
            "range": "stddev: 0.0000769601435994608",
            "extra": "mean: 560.6382355970159 usec\nrounds: 2135"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1608.4361013578175,
            "unit": "iter/sec",
            "range": "stddev: 0.0015543927315424839",
            "extra": "mean: 621.7219317297188 usec\nrounds: 2827"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 1041.8422380891288,
            "unit": "iter/sec",
            "range": "stddev: 0.0019282516677016873",
            "extra": "mean: 959.8382206447371 usec\nrounds: 1831"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 27829.335105146652,
            "unit": "iter/sec",
            "range": "stddev: 0.000006184658281268646",
            "extra": "mean: 35.93330549298908 usec\nrounds: 33333"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 8068381.999729011,
            "unit": "iter/sec",
            "range": "stddev: 2.5501100701794656e-8",
            "extra": "mean: 123.9405868529453 nsec\nrounds: 99991"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1308866.214587023,
            "unit": "iter/sec",
            "range": "stddev: 1.7312408818430932e-7",
            "extra": "mean: 764.0200265354739 nsec\nrounds: 161291"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 897344.4377719385,
            "unit": "iter/sec",
            "range": "stddev: 2.2985880976095083e-7",
            "extra": "mean: 1.1143992851651336 usec\nrounds: 108696"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 7752196.05404945,
            "unit": "iter/sec",
            "range": "stddev: 2.567374458588374e-8",
            "extra": "mean: 128.99570560753787 nsec\nrounds: 82645"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2994199.650651525,
            "unit": "iter/sec",
            "range": "stddev: 8.236272512735142e-8",
            "extra": "mean: 333.9790650841514 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 1073439.1398543532,
            "unit": "iter/sec",
            "range": "stddev: 2.0161064784277444e-7",
            "extra": "mean: 931.5851852910946 nsec\nrounds: 135136"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2855820.9653613395,
            "unit": "iter/sec",
            "range": "stddev: 8.643544247551885e-8",
            "extra": "mean: 350.1620066974751 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 7562357.804404749,
            "unit": "iter/sec",
            "range": "stddev: 2.602725423284555e-8",
            "extra": "mean: 132.2338913158633 nsec\nrounds: 89286"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 4078227.2412517625,
            "unit": "iter/sec",
            "range": "stddev: 4.087385583521545e-8",
            "extra": "mean: 245.20458053066284 nsec\nrounds: 49503"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1608118.8163276443,
            "unit": "iter/sec",
            "range": "stddev: 1.5490081493242793e-7",
            "extra": "mean: 621.8445986990605 nsec\nrounds: 181819"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2535166.4148734,
            "unit": "iter/sec",
            "range": "stddev: 9.792409382879224e-8",
            "extra": "mean: 394.45142304393363 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 3185615.190934034,
            "unit": "iter/sec",
            "range": "stddev: 7.849457368989228e-8",
            "extra": "mean: 313.9111098056885 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 4030914.8118825913,
            "unit": "iter/sec",
            "range": "stddev: 4.3990928042404855e-8",
            "extra": "mean: 248.08264294050994 nsec\nrounds: 48779"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "beltowski.t@gmail.com",
            "name": "Tom",
            "username": "czotomo"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d6ee7b4d50759198cd4e7294324e3c98f5a184d9",
          "message": "Tuple validator (#73)\n\n* add well-known vscode config to gitignore\r\n\r\n* force removal of profdata to avoid shell error\r\n\r\n* add tuple validation\r\n\r\n* split tuple types\r\n\r\n* implement fixed size tuple validator\r\n\r\n* restore strict tuple validator\r\n\r\n* fix tuple error context\r\n\r\n* add fix length tuple error tests\r\n\r\n* add fix length tuple test for json\r\n\r\n* mark tuple -> string json input non-covered for obvious reasons\r\n\r\n* restore build-coverage make target\r\n\r\n* bump general coverage for json inputs by covering error match arms\r\n\r\n* fix post-merge set validation error\r\n\r\n* remove py unused import\r\n\r\n* refine validator name\r\n\r\nCo-authored-by: Samuel Colvin <samcolvin@gmail.com>\r\n\r\n* use validators' collection length instead of input length\r\n\r\nCo-authored-by: Samuel Colvin <samcolvin@gmail.com>\r\n\r\n* fix field name after GH suggestion commit\r\n\r\n* address review comments\r\n\r\n* cleanup\r\n\r\n* improve coverage\r\n\r\n* improve coverage\r\n\r\n* cover tuple schema with nonsense items attempt\r\n\r\nCo-authored-by: Samuel Colvin <s@muelcolvin.com>\r\nCo-authored-by: Samuel Colvin <samcolvin@gmail.com>",
          "timestamp": "2022-06-16T21:13:44+01:00",
          "tree_id": "29f93e1edd5a7860c1dc5a582275ffab2504c47b",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/d6ee7b4d50759198cd4e7294324e3c98f5a184d9"
        },
        "date": 1655410696274,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 45126.28808900799,
            "unit": "iter/sec",
            "range": "stddev: 0.00001526564818629136",
            "extra": "mean: 22.160032263845412 usec\nrounds: 51544"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 28690.38531322294,
            "unit": "iter/sec",
            "range": "stddev: 0.000025747143495621516",
            "extra": "mean: 34.854882187278115 usec\nrounds: 32679"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 483144.5989632918,
            "unit": "iter/sec",
            "range": "stddev: 0.0000012492580947979936",
            "extra": "mean: 2.069773732637711 usec\nrounds: 54345"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1224017.871872753,
            "unit": "iter/sec",
            "range": "stddev: 0.0000012258664456569472",
            "extra": "mean: 816.9815351388694 nsec\nrounds: 140846"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 3845843.333342673,
            "unit": "iter/sec",
            "range": "stddev: 3.0123159274422534e-7",
            "extra": "mean: 260.0209923608332 nsec\nrounds: 44248"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 877172.5162055353,
            "unit": "iter/sec",
            "range": "stddev: 0.0000010186552179614482",
            "extra": "mean: 1.1400265985599929 usec\nrounds: 97088"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 11696.40662129578,
            "unit": "iter/sec",
            "range": "stddev: 0.00003359628740715285",
            "extra": "mean: 85.4963436530403 usec\nrounds: 12920"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 23388.938933875837,
            "unit": "iter/sec",
            "range": "stddev: 0.000043290307661240296",
            "extra": "mean: 42.75525293503716 usec\nrounds: 27173"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 2945.7169947389966,
            "unit": "iter/sec",
            "range": "stddev: 0.00006595113830186472",
            "extra": "mean: 339.4759244645646 usec\nrounds: 3270"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 5626.65482732707,
            "unit": "iter/sec",
            "range": "stddev: 0.00006040084792891963",
            "extra": "mean: 177.7254924441576 usec\nrounds: 6419"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2448.879176589852,
            "unit": "iter/sec",
            "range": "stddev: 0.00007764262343841424",
            "extra": "mean: 408.35007686762816 usec\nrounds: 2758"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 4288.789957037076,
            "unit": "iter/sec",
            "range": "stddev: 0.00012699585404795307",
            "extra": "mean: 233.16600020460154 usec\nrounds: 4866"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1185.5484370272977,
            "unit": "iter/sec",
            "range": "stddev: 0.00009760409063690138",
            "extra": "mean: 843.4914751416222 usec\nrounds: 1408"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1377.0043500498696,
            "unit": "iter/sec",
            "range": "stddev: 0.00020452089468354313",
            "extra": "mean: 726.2141183241607 usec\nrounds: 1648"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1334.5463252931877,
            "unit": "iter/sec",
            "range": "stddev: 0.001706448983826796",
            "extra": "mean: 749.3183121839619 usec\nrounds: 2175"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 836.2347597665051,
            "unit": "iter/sec",
            "range": "stddev: 0.0024526509835704796",
            "extra": "mean: 1.1958364422440675 msec\nrounds: 1515"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 24533.949325739395,
            "unit": "iter/sec",
            "range": "stddev: 0.00002505798100862602",
            "extra": "mean: 40.75984615126217 usec\nrounds: 27397"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 7002695.098279312,
            "unit": "iter/sec",
            "range": "stddev: 1.903709004361188e-7",
            "extra": "mean: 142.8021620198329 nsec\nrounds: 78126"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1090931.1111756405,
            "unit": "iter/sec",
            "range": "stddev: 9.034043589917103e-7",
            "extra": "mean: 916.6481638994749 nsec\nrounds: 128206"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 805208.4579320785,
            "unit": "iter/sec",
            "range": "stddev: 0.000003166459176097168",
            "extra": "mean: 1.241914426194866 usec\nrounds: 98030"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 7026313.869604199,
            "unit": "iter/sec",
            "range": "stddev: 9.521004291965031e-8",
            "extra": "mean: 142.32213626637798 nsec\nrounds: 76924"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2513462.2342337575,
            "unit": "iter/sec",
            "range": "stddev: 0.0000013629019824290985",
            "extra": "mean: 397.8575792310157 nsec\nrounds: 185186"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 963948.9615389254,
            "unit": "iter/sec",
            "range": "stddev: 9.217424354659701e-7",
            "extra": "mean: 1.0373993228887746 usec\nrounds: 108696"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2438274.060436807,
            "unit": "iter/sec",
            "range": "stddev: 4.4810427431463276e-7",
            "extra": "mean: 410.1261692546281 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 6468889.528444296,
            "unit": "iter/sec",
            "range": "stddev: 2.840772107250287e-7",
            "extra": "mean: 154.58603761943715 nsec\nrounds: 70922"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 3477652.3304696297,
            "unit": "iter/sec",
            "range": "stddev: 7.347085649417453e-7",
            "extra": "mean: 287.55030836100815 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1467232.7575134886,
            "unit": "iter/sec",
            "range": "stddev: 6.609545823705152e-7",
            "extra": "mean: 681.5551212849308 nsec\nrounds: 166667"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2087559.4719301236,
            "unit": "iter/sec",
            "range": "stddev: 5.0797356698513e-7",
            "extra": "mean: 479.0282688696664 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 2620058.545376767,
            "unit": "iter/sec",
            "range": "stddev: 4.857315436884752e-7",
            "extra": "mean: 381.6708606624662 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 2920384.7352528097,
            "unit": "iter/sec",
            "range": "stddev: 8.523994617318151e-7",
            "extra": "mean: 342.42063654474845 nsec\nrounds: 185186"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "s@muelcolvin.com",
            "name": "Samuel Colvin",
            "username": "samuelcolvin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "fc26fb4e5f29c7cba7a9791a51ec08e29048605a",
          "message": "assorted tweaks (#89)\n\n* fix heterogenious error context, ref #73\r\n\r\n* linting and correct trait order",
          "timestamp": "2022-06-16T21:49:47+01:00",
          "tree_id": "269ab4d8e4f158ee636541dc1aae18f7671a97d3",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/fc26fb4e5f29c7cba7a9791a51ec08e29048605a"
        },
        "date": 1655412747246,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 43894.695565645285,
            "unit": "iter/sec",
            "range": "stddev: 0.000002134111874466817",
            "extra": "mean: 22.78179600322054 usec\nrounds: 62501"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 29172.170279943242,
            "unit": "iter/sec",
            "range": "stddev: 0.0000026279285308463856",
            "extra": "mean: 34.27924595269247 usec\nrounds: 36629"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 469237.9251356582,
            "unit": "iter/sec",
            "range": "stddev: 1.547490386241784e-7",
            "extra": "mean: 2.1311150408632566 usec\nrounds: 68028"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1186064.99175628,
            "unit": "iter/sec",
            "range": "stddev: 1.1598188677333161e-7",
            "extra": "mean: 843.1241179449485 nsec\nrounds: 149254"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 3907232.332004931,
            "unit": "iter/sec",
            "range": "stddev: 3.610454108895115e-8",
            "extra": "mean: 255.9356380752502 nsec\nrounds: 56498"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 924759.3442163705,
            "unit": "iter/sec",
            "range": "stddev: 1.3652058702361408e-7",
            "extra": "mean: 1.0813624174269465 usec\nrounds: 103093"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 12873.601867794041,
            "unit": "iter/sec",
            "range": "stddev: 0.000004345911345793736",
            "extra": "mean: 77.67833822030069 usec\nrounds: 18553"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 24089.858577614094,
            "unit": "iter/sec",
            "range": "stddev: 0.0000030058881873537933",
            "extra": "mean: 41.511244110385384 usec\nrounds: 34722"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 2868.3212925798584,
            "unit": "iter/sec",
            "range": "stddev: 0.0000072856771822468135",
            "extra": "mean: 348.6359783288324 usec\nrounds: 4153"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 5508.878333606037,
            "unit": "iter/sec",
            "range": "stddev: 0.000006780782755375517",
            "extra": "mean: 181.5251562009745 usec\nrounds: 7023"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2441.8921748487924,
            "unit": "iter/sec",
            "range": "stddev: 0.000021108086545850878",
            "extra": "mean: 409.51849156153764 usec\nrounds: 2785"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 4308.754564657352,
            "unit": "iter/sec",
            "range": "stddev: 0.000009225423092334814",
            "extra": "mean: 232.08562590278885 usec\nrounds: 6231"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1234.628312984717,
            "unit": "iter/sec",
            "range": "stddev: 0.000023872252652331857",
            "extra": "mean: 809.9603657901686 usec\nrounds: 1520"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1462.4840751394765,
            "unit": "iter/sec",
            "range": "stddev: 0.00001634225349473664",
            "extra": "mean: 683.7681291706581 usec\nrounds: 2098"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1332.7242996358561,
            "unit": "iter/sec",
            "range": "stddev: 0.0019692765560834044",
            "extra": "mean: 750.3427380090786 usec\nrounds: 2481"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 863.8542735149197,
            "unit": "iter/sec",
            "range": "stddev: 0.002481282522876363",
            "extra": "mean: 1.1576026543587261 msec\nrounds: 1652"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_optional_core",
            "value": 23526.472574369403,
            "unit": "iter/sec",
            "range": "stddev: 0.0000028324711414280216",
            "extra": "mean: 42.50530957579406 usec\nrounds: 29240"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 6773885.591755826,
            "unit": "iter/sec",
            "range": "stddev: 1.8869267357986565e-8",
            "extra": "mean: 147.62575872510098 nsec\nrounds: 71429"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1203925.5154349823,
            "unit": "iter/sec",
            "range": "stddev: 1.1388781232425072e-7",
            "extra": "mean: 830.6161695047612 nsec\nrounds: 156251"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 805893.8148680731,
            "unit": "iter/sec",
            "range": "stddev: 1.4793831743389186e-7",
            "extra": "mean: 1.2408582638939243 usec\nrounds: 103093"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 6490279.604583775,
            "unit": "iter/sec",
            "range": "stddev: 1.491486001808248e-8",
            "extra": "mean: 154.07656694697567 nsec\nrounds: 83334"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2411449.4784601564,
            "unit": "iter/sec",
            "range": "stddev: 6.491584786825901e-8",
            "extra": "mean: 414.6883477889533 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 882740.6843830113,
            "unit": "iter/sec",
            "range": "stddev: 1.189197552674847e-7",
            "extra": "mean: 1.1328355174872118 usec\nrounds: 113637"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2349114.6852266523,
            "unit": "iter/sec",
            "range": "stddev: 6.939350220720422e-8",
            "extra": "mean: 425.6922858166652 nsec\nrounds: 185186"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 6239771.401370993,
            "unit": "iter/sec",
            "range": "stddev: 1.528489928737379e-8",
            "extra": "mean: 160.26228136819336 nsec\nrounds: 68494"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 3487530.997610888,
            "unit": "iter/sec",
            "range": "stddev: 2.3704973251437066e-8",
            "extra": "mean: 286.73580268822616 nsec\nrounds: 38315"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1243223.0379456559,
            "unit": "iter/sec",
            "range": "stddev: 1.2490764259778924e-7",
            "extra": "mean: 804.3608986306663 nsec\nrounds: 158731"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2136368.254748998,
            "unit": "iter/sec",
            "range": "stddev: 6.759436215755851e-8",
            "extra": "mean: 468.08409447999867 nsec\nrounds: 149254"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 2695126.3701774287,
            "unit": "iter/sec",
            "range": "stddev: 7.104969436275826e-8",
            "extra": "mean: 371.04011561973607 nsec\nrounds: 178572"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 3433848.523190131,
            "unit": "iter/sec",
            "range": "stddev: 2.9021235633208408e-8",
            "extra": "mean: 291.2184370528966 nsec\nrounds: 42919"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "em.jolibois@gmail.com",
            "name": "Eric Jolibois",
            "username": "PrettyWood"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "187860c6d353ee392e8a1f2c921b20301994543c",
          "message": "refactor: rename optional into nullable (#91)",
          "timestamp": "2022-06-18T21:22:52+01:00",
          "tree_id": "a1f864b1c65cb18af060378ae8ab8c888b7dea2a",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/187860c6d353ee392e8a1f2c921b20301994543c"
        },
        "date": 1655583998790,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 59309.06411529751,
            "unit": "iter/sec",
            "range": "stddev: 0.000006347718498490044",
            "extra": "mean: 16.860829198990366 usec\nrounds: 65790"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 39821.50787168686,
            "unit": "iter/sec",
            "range": "stddev: 0.0000011740481215165119",
            "extra": "mean: 25.11205761525171 usec\nrounds: 43669"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 592063.207281762,
            "unit": "iter/sec",
            "range": "stddev: 8.577798279808955e-8",
            "extra": "mean: 1.6890088553064877 usec\nrounds: 67564"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1555038.6906734353,
            "unit": "iter/sec",
            "range": "stddev: 8.059593111888412e-8",
            "extra": "mean: 643.0708161774194 nsec\nrounds: 172414"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 4724799.945299319,
            "unit": "iter/sec",
            "range": "stddev: 1.6825697976739374e-8",
            "extra": "mean: 211.6491727856165 nsec\nrounds: 48077"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 1221470.8956254388,
            "unit": "iter/sec",
            "range": "stddev: 9.179165305415135e-8",
            "extra": "mean: 818.6850817174129 nsec\nrounds: 125001"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 16035.452483086794,
            "unit": "iter/sec",
            "range": "stddev: 0.0000027378173438695956",
            "extra": "mean: 62.36181991463842 usec\nrounds: 17575"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 31968.222391611424,
            "unit": "iter/sec",
            "range": "stddev: 0.0000019614592903862264",
            "extra": "mean: 31.28106366847609 usec\nrounds: 35088"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3831.5497398978505,
            "unit": "iter/sec",
            "range": "stddev: 0.000007810302869375497",
            "extra": "mean: 260.99100047874106 usec\nrounds: 4176"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 7392.723909465751,
            "unit": "iter/sec",
            "range": "stddev: 0.0000035059517970137193",
            "extra": "mean: 135.26813827303704 usec\nrounds: 8071"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 3187.7758897406184,
            "unit": "iter/sec",
            "range": "stddev: 0.000007062139046944295",
            "extra": "mean: 313.69833846173157 usec\nrounds: 3445"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5891.782449069838,
            "unit": "iter/sec",
            "range": "stddev: 0.000005816231879923779",
            "extra": "mean: 169.72792336517355 usec\nrounds: 6407"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1653.0990828834351,
            "unit": "iter/sec",
            "range": "stddev: 0.00000944484703926808",
            "extra": "mean: 604.9244176312407 usec\nrounds: 1815"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1980.8540100640123,
            "unit": "iter/sec",
            "range": "stddev: 0.000009713073216751689",
            "extra": "mean: 504.832761485378 usec\nrounds: 2046"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1598.1309066141785,
            "unit": "iter/sec",
            "range": "stddev: 0.0019219299912028253",
            "extra": "mean: 625.73096850909 usec\nrounds: 2858"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 1035.055103020216,
            "unit": "iter/sec",
            "range": "stddev: 0.002442713236921276",
            "extra": "mean: 966.1321383586944 usec\nrounds: 1937"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_nullable_core",
            "value": 31906.585693959045,
            "unit": "iter/sec",
            "range": "stddev: 0.0000018152808953930515",
            "extra": "mean: 31.34149199139576 usec\nrounds: 35212"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 9123575.1671163,
            "unit": "iter/sec",
            "range": "stddev: 1.143868314592463e-8",
            "extra": "mean: 109.60615566633666 nsec\nrounds: 101011"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1573524.7361842385,
            "unit": "iter/sec",
            "range": "stddev: 4.103760994681907e-8",
            "extra": "mean: 635.5159070613045 nsec\nrounds: 172414"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 1016923.8420482742,
            "unit": "iter/sec",
            "range": "stddev: 1.003627190926481e-7",
            "extra": "mean: 983.3578077839802 nsec\nrounds: 114943"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 9044713.972796764,
            "unit": "iter/sec",
            "range": "stddev: 1.2682231381619362e-8",
            "extra": "mean: 110.56181577528103 nsec\nrounds: 97088"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 3088820.761602798,
            "unit": "iter/sec",
            "range": "stddev: 5.049598264974602e-8",
            "extra": "mean: 323.7481476521737 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 1228164.3316820806,
            "unit": "iter/sec",
            "range": "stddev: 7.701676336094219e-8",
            "extra": "mean: 814.2232877179545 nsec\nrounds: 129871"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2999341.9896817734,
            "unit": "iter/sec",
            "range": "stddev: 4.809583651790912e-8",
            "extra": "mean: 333.40646163040685 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 8552973.083591484,
            "unit": "iter/sec",
            "range": "stddev: 1.249487307647546e-8",
            "extra": "mean: 116.91840839752192 nsec\nrounds: 91744"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 4397622.280220543,
            "unit": "iter/sec",
            "range": "stddev: 1.727974725291742e-8",
            "extra": "mean: 227.39560978163686 nsec\nrounds: 47394"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1835636.163549719,
            "unit": "iter/sec",
            "range": "stddev: 4.842005794125582e-8",
            "extra": "mean: 544.770265402698 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2737946.513219342,
            "unit": "iter/sec",
            "range": "stddev: 4.2667883937699276e-8",
            "extra": "mean: 365.2372298628296 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 3646649.4665190983,
            "unit": "iter/sec",
            "range": "stddev: 2.7058152211893324e-8",
            "extra": "mean: 274.22432816229514 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 4332185.111002608,
            "unit": "iter/sec",
            "range": "stddev: 1.079345068854848e-8",
            "extra": "mean: 230.83039491093913 nsec\nrounds: 44643"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "beltowski.t@gmail.com",
            "name": "Tom",
            "username": "czotomo"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8faeed750b7dad3f2a6129fff0515da73eec5ac6",
          "message": "reuse error kinds where appropriate (#81)\n\n* gather comparison errors under unified enumerations\r\n\r\n* use unified comparison error kind for int and float\r\n\r\n* unify  and  errors\r\n\r\n* make str and bytes toolong/tooshort error kinds standalone\r\n\r\n* remove unnecessary variables from errors\r\n\r\n* linting\r\n\r\nCo-authored-by: Samuel Colvin <s@muelcolvin.com>",
          "timestamp": "2022-06-20T13:43:51+01:00",
          "tree_id": "36797af0cd48f74c0d7ea7856d6783504c83f09a",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/8faeed750b7dad3f2a6129fff0515da73eec5ac6"
        },
        "date": 1655729265004,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 51772.4269472669,
            "unit": "iter/sec",
            "range": "stddev: 6.888295403498451e-7",
            "extra": "mean: 19.315300807098644 usec\nrounds: 52911"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 34535.18420641012,
            "unit": "iter/sec",
            "range": "stddev: 6.975833112639021e-7",
            "extra": "mean: 28.95597701240547 usec\nrounds: 35715"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 558260.5106217816,
            "unit": "iter/sec",
            "range": "stddev: 4.390097301509847e-8",
            "extra": "mean: 1.791278410300485 usec\nrounds: 57143"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1398783.0384997411,
            "unit": "iter/sec",
            "range": "stddev: 4.985162290522199e-8",
            "extra": "mean: 714.9071532011951 nsec\nrounds: 144928"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 4704179.042437619,
            "unit": "iter/sec",
            "range": "stddev: 4.5169368261981795e-9",
            "extra": "mean: 212.5769429646017 nsec\nrounds: 47847"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 1064161.3395231147,
            "unit": "iter/sec",
            "range": "stddev: 3.550977596947719e-8",
            "extra": "mean: 939.707131672704 nsec\nrounds: 111124"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 13996.670729862582,
            "unit": "iter/sec",
            "range": "stddev: 0.0000010290193782129407",
            "extra": "mean: 71.44556154103498 usec\nrounds: 14145"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 28478.87924576215,
            "unit": "iter/sec",
            "range": "stddev: 7.977020883752198e-7",
            "extra": "mean: 35.11374135795062 usec\nrounds: 28986"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3440.5053920916794,
            "unit": "iter/sec",
            "range": "stddev: 0.000001830990792891697",
            "extra": "mean: 290.6549724638109 usec\nrounds: 3450"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6566.332074862109,
            "unit": "iter/sec",
            "range": "stddev: 0.0000019691025888922132",
            "extra": "mean: 152.29202370503012 usec\nrounds: 6623"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2881.3667510929163,
            "unit": "iter/sec",
            "range": "stddev: 0.0000027476251000732846",
            "extra": "mean: 347.0575204009331 usec\nrounds: 2892"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5083.215948962027,
            "unit": "iter/sec",
            "range": "stddev: 0.0000024293007761729663",
            "extra": "mean: 196.7258542703062 usec\nrounds: 5222"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1479.3348478429223,
            "unit": "iter/sec",
            "range": "stddev: 0.0000057409651746522686",
            "extra": "mean: 675.9794791950857 usec\nrounds: 1490"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1754.0269663003332,
            "unit": "iter/sec",
            "range": "stddev: 0.0000047752368983045535",
            "extra": "mean: 570.1166625215811 usec\nrounds: 1769"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1445.7939655042512,
            "unit": "iter/sec",
            "range": "stddev: 0.002150172120634905",
            "extra": "mean: 691.6614841805823 usec\nrounds: 2402"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 923.4352766980749,
            "unit": "iter/sec",
            "range": "stddev: 0.002728964416400621",
            "extra": "mean: 1.0829129287498063 msec\nrounds: 1600"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_nullable_core",
            "value": 28444.010500015116,
            "unit": "iter/sec",
            "range": "stddev: 8.679965830239219e-7",
            "extra": "mean: 35.15678634696991 usec\nrounds: 28902"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 8082299.742073702,
            "unit": "iter/sec",
            "range": "stddev: 5.284350098995377e-8",
            "extra": "mean: 123.72716082198079 nsec\nrounds: 83334"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1342950.7292981423,
            "unit": "iter/sec",
            "range": "stddev: 3.3005752234127105e-8",
            "extra": "mean: 744.6289563600195 nsec\nrounds: 138909"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 897390.0291783238,
            "unit": "iter/sec",
            "range": "stddev: 3.88140547301346e-8",
            "extra": "mean: 1.114342668723104 usec\nrounds: 91744"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 7618172.104548739,
            "unit": "iter/sec",
            "range": "stddev: 4.598515863715718e-9",
            "extra": "mean: 131.26508383860636 nsec\nrounds: 78126"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2809736.5509344796,
            "unit": "iter/sec",
            "range": "stddev: 2.3789744103366482e-8",
            "extra": "mean: 355.90525370338145 nsec\nrounds: 185186"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 1100599.4541759305,
            "unit": "iter/sec",
            "range": "stddev: 5.2898535097236375e-8",
            "extra": "mean: 908.595762250894 nsec\nrounds: 114943"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2831571.977099312,
            "unit": "iter/sec",
            "range": "stddev: 1.5784672715799817e-8",
            "extra": "mean: 353.1607206490403 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 7313757.049133086,
            "unit": "iter/sec",
            "range": "stddev: 4.3501816587903384e-9",
            "extra": "mean: 136.72863253215678 nsec\nrounds: 74627"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 4147217.007994853,
            "unit": "iter/sec",
            "range": "stddev: 4.737585937957262e-9",
            "extra": "mean: 241.12555433503155 nsec\nrounds: 41843"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1522550.7829450541,
            "unit": "iter/sec",
            "range": "stddev: 3.903443668914981e-8",
            "extra": "mean: 656.7925426207578 nsec\nrounds: 166667"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2516260.2525824574,
            "unit": "iter/sec",
            "range": "stddev: 1.8548087647733423e-8",
            "extra": "mean: 397.41517157229447 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 3196082.9676603293,
            "unit": "iter/sec",
            "range": "stddev: 1.828418691907255e-8",
            "extra": "mean: 312.8829914978768 nsec\nrounds: 192345"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 3935432.1385580017,
            "unit": "iter/sec",
            "range": "stddev: 1.2381389620376212e-8",
            "extra": "mean: 254.1017008533947 nsec\nrounds: 188715"
          }
        ]
      },
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
          "id": "4a1992254805da4aadfa8ad537a0309cd27d9b4f",
          "message": "benchmarks for list[any] and dict[any, any]",
          "timestamp": "2022-06-20T15:30:09+01:00",
          "tree_id": "5116edd12bbcf44d5f711a871f85a4880e766326",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/4a1992254805da4aadfa8ad537a0309cd27d9b4f"
        },
        "date": 1655735561289,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 51287.988815998346,
            "unit": "iter/sec",
            "range": "stddev: 6.476523251050498e-7",
            "extra": "mean: 19.49774251409266 usec\nrounds: 52632"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 34931.64675813734,
            "unit": "iter/sec",
            "range": "stddev: 7.474685416887214e-7",
            "extra": "mean: 28.627336321241412 usec\nrounds: 35588"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 547859.8630219089,
            "unit": "iter/sec",
            "range": "stddev: 4.855812388665389e-8",
            "extra": "mean: 1.8252842880002151 usec\nrounds: 56180"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1398801.4146568696,
            "unit": "iter/sec",
            "range": "stddev: 3.8037715761243675e-8",
            "extra": "mean: 714.8977614133285 nsec\nrounds: 142858"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 4629132.688396197,
            "unit": "iter/sec",
            "range": "stddev: 5.046241051637532e-9",
            "extra": "mean: 216.02318777917426 nsec\nrounds: 46512"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 1060472.7107099986,
            "unit": "iter/sec",
            "range": "stddev: 4.209281481758325e-8",
            "extra": "mean: 942.9757030993264 nsec\nrounds: 112360"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 14027.661187197798,
            "unit": "iter/sec",
            "range": "stddev: 0.0000010187419372783292",
            "extra": "mean: 71.287721214185 usec\nrounds: 14165"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 28056.9333722619,
            "unit": "iter/sec",
            "range": "stddev: 7.56991183323341e-7",
            "extra": "mean: 35.64181397631418 usec\nrounds: 28491"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3456.967010991912,
            "unit": "iter/sec",
            "range": "stddev: 0.000002302767590122544",
            "extra": "mean: 289.2709119931893 usec\nrounds: 3477"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_any_core_py",
            "value": 20260.4798282428,
            "unit": "iter/sec",
            "range": "stddev: 8.613716840064434e-7",
            "extra": "mean: 49.35717260782815 usec\nrounds: 20619"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6560.705133051149,
            "unit": "iter/sec",
            "range": "stddev: 0.0000018911751243801346",
            "extra": "mean: 152.42264051195605 usec\nrounds: 6640"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2859.1786167381,
            "unit": "iter/sec",
            "range": "stddev: 0.0000022596777860918034",
            "extra": "mean: 349.75079701066454 usec\nrounds: 2877"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5133.917496716663,
            "unit": "iter/sec",
            "range": "stddev: 0.000001963370384400201",
            "extra": "mean: 194.7830288740593 usec\nrounds: 5195"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1482.7009909659023,
            "unit": "iter/sec",
            "range": "stddev: 0.0000037974223940841493",
            "extra": "mean: 674.4448179997182 usec\nrounds: 1500"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_any_core",
            "value": 7061.047382234633,
            "unit": "iter/sec",
            "range": "stddev: 0.0000015536282311074833",
            "extra": "mean: 141.62204923252148 usec\nrounds: 7231"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1690.799689406897,
            "unit": "iter/sec",
            "range": "stddev: 0.000004329968165878472",
            "extra": "mean: 591.4361152684992 usec\nrounds: 1657"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1575.2613120117521,
            "unit": "iter/sec",
            "range": "stddev: 0.0017748439618664729",
            "extra": "mean: 634.8153111961526 usec\nrounds: 2349"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 950.5521432820427,
            "unit": "iter/sec",
            "range": "stddev: 0.0024399389006858783",
            "extra": "mean: 1.0520201412067989 msec\nrounds: 1558"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_nullable_core",
            "value": 28471.835225680443,
            "unit": "iter/sec",
            "range": "stddev: 7.036970792129335e-7",
            "extra": "mean: 35.122428606149015 usec\nrounds: 28819"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 7934317.320898362,
            "unit": "iter/sec",
            "range": "stddev: 4.472049647430104e-9",
            "extra": "mean: 126.03478781537778 nsec\nrounds: 80001"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1376700.2863826072,
            "unit": "iter/sec",
            "range": "stddev: 3.972031683024279e-8",
            "extra": "mean: 726.3745129502443 nsec\nrounds: 147059"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 889699.5225854672,
            "unit": "iter/sec",
            "range": "stddev: 4.6654946685166313e-8",
            "extra": "mean: 1.1239749765109863 usec\nrounds: 92593"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 7482657.045556723,
            "unit": "iter/sec",
            "range": "stddev: 4.36102891582483e-9",
            "extra": "mean: 133.6423671314152 nsec\nrounds: 77520"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2736784.3327446016,
            "unit": "iter/sec",
            "range": "stddev: 2.0514444660856154e-8",
            "extra": "mean: 365.39232852050986 nsec\nrounds: 185186"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 1057586.9582768332,
            "unit": "iter/sec",
            "range": "stddev: 5.673759228375364e-8",
            "extra": "mean: 945.5487250232499 nsec\nrounds: 114943"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2794717.5689499904,
            "unit": "iter/sec",
            "range": "stddev: 1.9702153634529536e-8",
            "extra": "mean: 357.8179101572615 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 7147720.867241858,
            "unit": "iter/sec",
            "range": "stddev: 4.491622656862024e-9",
            "extra": "mean: 139.9047358694743 nsec\nrounds: 72464"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 4074330.0852368176,
            "unit": "iter/sec",
            "range": "stddev: 6.21879299970837e-9",
            "extra": "mean: 245.4391222801057 nsec\nrounds: 41323"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1578367.749081167,
            "unit": "iter/sec",
            "range": "stddev: 3.9685616420582324e-8",
            "extra": "mean: 633.5659104680453 nsec\nrounds: 166667"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2535187.8627838013,
            "unit": "iter/sec",
            "range": "stddev: 2.0004337008913238e-8",
            "extra": "mean: 394.4480859509242 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 3173953.6387892296,
            "unit": "iter/sec",
            "range": "stddev: 1.6814497922419294e-8",
            "extra": "mean: 315.064463379692 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 3878865.4978233054,
            "unit": "iter/sec",
            "range": "stddev: 1.1084045317449952e-8",
            "extra": "mean: 257.8073409766509 nsec\nrounds: 188680"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "s@muelcolvin.com",
            "name": "Samuel Colvin",
            "username": "samuelcolvin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "77b7ae6a4f5dc2e7bb539aff7719439027c0bfe8",
          "message": "stop using `dyn Input` and other related changes (#93)\n\n* working on impl input instead of dyn\r\n\r\n* sequences working\r\n\r\n* more types working\r\n\r\n* dict working, simplify sequence generators\r\n\r\n* Working!!!\r\n\r\n* improving list, set & tuple validators\r\n\r\n* stop using \"dyn Input\"\r\n\r\n* remove to_py\r\n\r\n* add more rust benchmarks\r\n\r\n* improve how InputValue works\r\n\r\n* reverse some input merges\r\n\r\n* cleanup\r\n\r\n* tweak literals errors\r\n\r\n* use \"impl Input\" instead of generic param\r\n\r\n* use dedicated function macros",
          "timestamp": "2022-06-20T17:25:35+01:00",
          "tree_id": "b9d4b077b68a7cc6b799579a09156cc9b8a64b13",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/77b7ae6a4f5dc2e7bb539aff7719439027c0bfe8"
        },
        "date": 1655742482836,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 54382.6910005072,
            "unit": "iter/sec",
            "range": "stddev: 6.256719044052835e-7",
            "extra": "mean: 18.388203702363192 usec\nrounds: 56180"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 35744.914979725836,
            "unit": "iter/sec",
            "range": "stddev: 7.606873941830084e-7",
            "extra": "mean: 27.976007232558533 usec\nrounds: 36364"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 536774.1759600013,
            "unit": "iter/sec",
            "range": "stddev: 5.6455524170405665e-8",
            "extra": "mean: 1.862980830274908 usec\nrounds: 54946"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1274512.7839823563,
            "unit": "iter/sec",
            "range": "stddev: 1.4046513705019194e-7",
            "extra": "mean: 784.6135500303116 nsec\nrounds: 131579"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 3465756.939284924,
            "unit": "iter/sec",
            "range": "stddev: 2.0358958616885298e-8",
            "extra": "mean: 288.53725680163336 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 1030528.3556115003,
            "unit": "iter/sec",
            "range": "stddev: 1.6742883767621446e-7",
            "extra": "mean: 970.3760159097089 nsec\nrounds: 106383"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 14750.193881963369,
            "unit": "iter/sec",
            "range": "stddev: 0.0000022349548106818094",
            "extra": "mean: 67.79571902595845 usec\nrounds: 14948"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 28966.76549102354,
            "unit": "iter/sec",
            "range": "stddev: 8.248933706871259e-7",
            "extra": "mean: 34.52232180737916 usec\nrounds: 29325"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3533.0219422624727,
            "unit": "iter/sec",
            "range": "stddev: 0.000009579503521021767",
            "extra": "mean: 283.0438124478845 usec\nrounds: 3599"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_any_core_py",
            "value": 28412.630361511034,
            "unit": "iter/sec",
            "range": "stddev: 0.000001098869551594215",
            "extra": "mean: 35.19561502319204 usec\nrounds: 29155"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6796.07307855974,
            "unit": "iter/sec",
            "range": "stddev: 0.0000016893401846605878",
            "extra": "mean: 147.14379737245633 usec\nrounds: 6850"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2855.0851950914243,
            "unit": "iter/sec",
            "range": "stddev: 0.000014150934434455833",
            "extra": "mean: 350.2522452637279 usec\nrounds: 2956"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5316.582494321223,
            "unit": "iter/sec",
            "range": "stddev: 0.000002731818952660282",
            "extra": "mean: 188.09075210779207 usec\nrounds: 5337"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1440.4916404478074,
            "unit": "iter/sec",
            "range": "stddev: 0.000004745695966283584",
            "extra": "mean: 694.2074302417533 usec\nrounds: 1455"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_any_core",
            "value": 7354.069329991387,
            "unit": "iter/sec",
            "range": "stddev: 0.0000019551143634402948",
            "extra": "mean: 135.9791368734854 usec\nrounds: 7474"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1729.048652055179,
            "unit": "iter/sec",
            "range": "stddev: 0.000004647166505298884",
            "extra": "mean: 578.3527252465577 usec\nrounds: 1707"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1461.010598788558,
            "unit": "iter/sec",
            "range": "stddev: 0.002265126381131059",
            "extra": "mean: 684.4577314012513 usec\nrounds: 2379"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 893.1599906198898,
            "unit": "iter/sec",
            "range": "stddev: 0.002946724518320824",
            "extra": "mean: 1.1196202365781733 msec\nrounds: 1602"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_nullable_core",
            "value": 32257.54961429725,
            "unit": "iter/sec",
            "range": "stddev: 7.973287466869466e-7",
            "extra": "mean: 31.00049482855877 usec\nrounds: 32680"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 5218615.05474972,
            "unit": "iter/sec",
            "range": "stddev: 5.493024086689111e-9",
            "extra": "mean: 191.62172137795073 nsec\nrounds: 52911"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1337848.0542348104,
            "unit": "iter/sec",
            "range": "stddev: 4.5638251842630975e-8",
            "extra": "mean: 747.4690394271146 nsec\nrounds: 136987"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 904797.6819723003,
            "unit": "iter/sec",
            "range": "stddev: 4.6330209775210346e-8",
            "extra": "mean: 1.1052194539441327 usec\nrounds: 92593"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 5037657.464613583,
            "unit": "iter/sec",
            "range": "stddev: 5.5061619578061234e-9",
            "extra": "mean: 198.5049612889506 nsec\nrounds: 51021"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2168399.5911029647,
            "unit": "iter/sec",
            "range": "stddev: 2.5143736268526542e-8",
            "extra": "mean: 461.16961288079284 nsec\nrounds: 181819"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 971530.3306027894,
            "unit": "iter/sec",
            "range": "stddev: 7.93718250705855e-8",
            "extra": "mean: 1.0293039429644406 usec\nrounds: 100001"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2121355.5317162327,
            "unit": "iter/sec",
            "range": "stddev: 2.622622816322468e-8",
            "extra": "mean: 471.3967013307616 nsec\nrounds: 178572"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 4991296.716718094,
            "unit": "iter/sec",
            "range": "stddev: 6.064178738981522e-9",
            "extra": "mean: 200.34873836503343 nsec\nrounds: 50506"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 3192598.1820028797,
            "unit": "iter/sec",
            "range": "stddev: 2.092836869614435e-8",
            "extra": "mean: 313.2245096284918 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1467886.3165356943,
            "unit": "iter/sec",
            "range": "stddev: 4.707959245648393e-8",
            "extra": "mean: 681.251666927041 nsec\nrounds: 153847"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2151759.5507646394,
            "unit": "iter/sec",
            "range": "stddev: 2.7407716958620113e-8",
            "extra": "mean: 464.7359411721021 nsec\nrounds: 185186"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 2610719.977867945,
            "unit": "iter/sec",
            "range": "stddev: 2.0674744055787406e-8",
            "extra": "mean: 383.0361005689851 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 3094770.48726448,
            "unit": "iter/sec",
            "range": "stddev: 2.103475929235945e-8",
            "extra": "mean: 323.1257387624778 nsec\nrounds: 185186"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "dswijj@gmail.com",
            "name": "dswij",
            "username": "dswij"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e4f19a0980967d53268c9697f25a4b9324ca2c2a",
          "message": "Add `frozenset` type (#86)\n\n* Add `frozenset` type\r\n\r\n* Use total=false\r\n\r\n* Add dict keys and simple values testcase for frozenset, set\r\n\r\n* Remove string case from frozenset\r\n\r\n* fix test\r\n\r\n* Fix error message for frozenset JSON\r\n\r\n* fix lint\r\n\r\n* fix rust lint\r\n\r\n* `frozenset` test check for instance type\r\n\r\n* update to match changes, more tests\r\n\r\nCo-authored-by: Samuel Colvin <s@muelcolvin.com>",
          "timestamp": "2022-06-20T18:12:38+01:00",
          "tree_id": "68e7eebe515159dd534d4d19a8026d2fda6a4634",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/e4f19a0980967d53268c9697f25a4b9324ca2c2a"
        },
        "date": 1655745334266,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 45522.33572081712,
            "unit": "iter/sec",
            "range": "stddev: 0.0000671792988385436",
            "extra": "mean: 21.96723837135416 usec\nrounds: 63292"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 28059.7514948192,
            "unit": "iter/sec",
            "range": "stddev: 0.000017369861133846098",
            "extra": "mean: 35.638234365141635 usec\nrounds: 37736"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 444817.1337682659,
            "unit": "iter/sec",
            "range": "stddev: 0.0000015180987754937506",
            "extra": "mean: 2.2481148410995897 usec\nrounds: 59881"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1207633.4544914782,
            "unit": "iter/sec",
            "range": "stddev: 8.146019970581837e-7",
            "extra": "mean: 828.0658309694985 nsec\nrounds: 163935"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 3702599.4333861144,
            "unit": "iter/sec",
            "range": "stddev: 3.136581292101736e-7",
            "extra": "mean: 270.0805253150274 nsec\nrounds: 43669"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 869884.9065234987,
            "unit": "iter/sec",
            "range": "stddev: 0.0000037425997555231725",
            "extra": "mean: 1.1495773665006053 usec\nrounds: 112360"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 10594.477957889563,
            "unit": "iter/sec",
            "range": "stddev: 0.00008698852710813386",
            "extra": "mean: 94.38879423552095 usec\nrounds: 14225"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 23854.695338308873,
            "unit": "iter/sec",
            "range": "stddev: 0.000020311872031424236",
            "extra": "mean: 41.92046831107812 usec\nrounds: 30121"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 2924.532081561905,
            "unit": "iter/sec",
            "range": "stddev: 0.00010741528450013635",
            "extra": "mean: 341.93504195239666 usec\nrounds: 3647"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_any_core_py",
            "value": 21825.46315826076,
            "unit": "iter/sec",
            "range": "stddev: 0.00003826668377022053",
            "extra": "mean: 45.818042565640035 usec\nrounds: 29155"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 5538.375564531471,
            "unit": "iter/sec",
            "range": "stddev: 0.000046121728497603985",
            "extra": "mean: 180.55835837571928 usec\nrounds: 7068"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2395.2681138250587,
            "unit": "iter/sec",
            "range": "stddev: 0.0000634702262272316",
            "extra": "mean: 417.48979758390266 usec\nrounds: 3063"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 4100.601223883015,
            "unit": "iter/sec",
            "range": "stddev: 0.00022974006098533742",
            "extra": "mean: 243.86667842162467 usec\nrounds: 5728"
          },
          {
            "name": "tests/test_benchmarks.py::test_frozenset_of_ints_core",
            "value": 13073.512140881561,
            "unit": "iter/sec",
            "range": "stddev: 0.00003146298220362934",
            "extra": "mean: 76.49053974355883 usec\nrounds: 16921"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1132.9783389529898,
            "unit": "iter/sec",
            "range": "stddev: 0.00028837738673541364",
            "extra": "mean: 882.6294074819841 usec\nrounds: 1497"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_any_core",
            "value": 5750.048297538242,
            "unit": "iter/sec",
            "range": "stddev: 0.0001744172038432915",
            "extra": "mean: 173.9115826954233 usec\nrounds: 7316"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1385.3824387313614,
            "unit": "iter/sec",
            "range": "stddev: 0.00013912967160656863",
            "extra": "mean: 721.8223445330602 usec\nrounds: 1756"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1296.8435112067834,
            "unit": "iter/sec",
            "range": "stddev: 0.00198814390256591",
            "extra": "mean: 771.1030601290094 usec\nrounds: 2478"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 815.6127500181874,
            "unit": "iter/sec",
            "range": "stddev: 0.002546827230512279",
            "extra": "mean: 1.2260720543881896 msec\nrounds: 1618"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_nullable_core",
            "value": 27499.683534551554,
            "unit": "iter/sec",
            "range": "stddev: 0.00006503544505532703",
            "extra": "mean: 36.36405483516076 usec\nrounds: 38023"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 6894428.85460778,
            "unit": "iter/sec",
            "range": "stddev: 1.9961246626906367e-7",
            "extra": "mean: 145.0446470749205 nsec\nrounds: 82652"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1140546.701998121,
            "unit": "iter/sec",
            "range": "stddev: 0.0000015518777336300952",
            "extra": "mean: 876.7725146616151 nsec\nrounds: 140846"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 748755.8281862232,
            "unit": "iter/sec",
            "range": "stddev: 0.0000021874338417085798",
            "extra": "mean: 1.3355488696794893 usec\nrounds: 95239"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 6317391.1431716,
            "unit": "iter/sec",
            "range": "stddev: 1.5740023265572448e-7",
            "extra": "mean: 158.29319054924534 nsec\nrounds: 72464"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2137567.79904359,
            "unit": "iter/sec",
            "range": "stddev: 0.00000169014092548082",
            "extra": "mean: 467.82141855216423 nsec\nrounds: 181819"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 843558.5194015031,
            "unit": "iter/sec",
            "range": "stddev: 0.0000025598899963348994",
            "extra": "mean: 1.1854542121268177 usec\nrounds: 106395"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2159055.5423561204,
            "unit": "iter/sec",
            "range": "stddev: 8.507070349865207e-7",
            "extra": "mean: 463.16548156457884 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 6427423.541115861,
            "unit": "iter/sec",
            "range": "stddev: 1.315340334744739e-7",
            "extra": "mean: 155.58333655833692 nsec\nrounds: 77520"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 3239571.888017368,
            "unit": "iter/sec",
            "range": "stddev: 0.0000010994147222075187",
            "extra": "mean: 308.68276258916444 nsec\nrounds: 196117"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1146632.7032911694,
            "unit": "iter/sec",
            "range": "stddev: 0.000001112098966145321",
            "extra": "mean: 872.1188547384833 nsec\nrounds: 161291"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2135540.1795884,
            "unit": "iter/sec",
            "range": "stddev: 5.274176476294438e-7",
            "extra": "mean: 468.2655983521 nsec\nrounds: 178572"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 2100311.2339884746,
            "unit": "iter/sec",
            "range": "stddev: 0.0000024897988550636148",
            "extra": "mean: 476.11991204799097 nsec\nrounds: 175439"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 3247886.947305957,
            "unit": "iter/sec",
            "range": "stddev: 5.783018885728283e-7",
            "extra": "mean: 307.8924901709564 nsec\nrounds: 192308"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "s@muelcolvin.com",
            "name": "Samuel Colvin",
            "username": "samuelcolvin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b39cad87a9e0086566a38b63095bce90dcea6877",
          "message": "changes to benchmarks (#95)",
          "timestamp": "2022-06-20T23:15:22+01:00",
          "tree_id": "aefed726fa55fc67f0535378460317940b8dd497",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/b39cad87a9e0086566a38b63095bce90dcea6877"
        },
        "date": 1655763496139,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 44880.20193232513,
            "unit": "iter/sec",
            "range": "stddev: 9.760005079626325e-7",
            "extra": "mean: 22.281539675509933 usec\nrounds: 45872"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 29419.350981946496,
            "unit": "iter/sec",
            "range": "stddev: 9.094082316758209e-7",
            "extra": "mean: 33.99123252629403 usec\nrounds: 30031"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 464956.12433494977,
            "unit": "iter/sec",
            "range": "stddev: 5.9449252458745745e-8",
            "extra": "mean: 2.1507405702642166 usec\nrounds: 47170"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1092186.8029576985,
            "unit": "iter/sec",
            "range": "stddev: 4.344777817370121e-8",
            "extra": "mean: 915.5942896326271 nsec\nrounds: 116280"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 851699.4326454893,
            "unit": "iter/sec",
            "range": "stddev: 5.3438837543511565e-8",
            "extra": "mean: 1.1741231256824147 usec\nrounds: 89286"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 3335199.4726999197,
            "unit": "iter/sec",
            "range": "stddev: 1.7026635996951483e-8",
            "extra": "mean: 299.83214143066317 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 12804.361043592642,
            "unit": "iter/sec",
            "range": "stddev: 0.000001420188054281956",
            "extra": "mean: 78.09839136802569 usec\nrounds: 13021"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 23679.427922644947,
            "unit": "iter/sec",
            "range": "stddev: 8.856559281486821e-7",
            "extra": "mean: 42.23074996857027 usec\nrounds: 23809"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 2941.493205114277,
            "unit": "iter/sec",
            "range": "stddev: 0.0000030363278545057685",
            "extra": "mean: 339.96338943137215 usec\nrounds: 2971"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 5644.377547701486,
            "unit": "iter/sec",
            "range": "stddev: 0.000001704222859550817",
            "extra": "mean: 177.1674540813135 usec\nrounds: 5673"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_any_core_py",
            "value": 24114.219214849672,
            "unit": "iter/sec",
            "range": "stddev: 8.180629484193477e-7",
            "extra": "mean: 41.469308671797855 usec\nrounds: 26316"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2383.3249829455403,
            "unit": "iter/sec",
            "range": "stddev: 0.00022697692352943797",
            "extra": "mean: 419.5818896523732 usec\nrounds: 2474"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 4470.49907678307,
            "unit": "iter/sec",
            "range": "stddev: 0.00000486030784443194",
            "extra": "mean: 223.6886716280435 usec\nrounds: 4501"
          },
          {
            "name": "tests/test_benchmarks.py::test_frozenset_of_ints_core",
            "value": 13120.382364531833,
            "unit": "iter/sec",
            "range": "stddev: 0.00000196649022351444",
            "extra": "mean: 76.2172909459779 usec\nrounds: 13298"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1178.201974782879,
            "unit": "iter/sec",
            "range": "stddev: 0.000004331705398388418",
            "extra": "mean: 848.7509114761767 usec\nrounds: 1220"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_any_core",
            "value": 6019.012059957244,
            "unit": "iter/sec",
            "range": "stddev: 0.000002444031186446163",
            "extra": "mean: 166.1402220229317 usec\nrounds: 6139"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1450.053378839003,
            "unit": "iter/sec",
            "range": "stddev: 0.00000465459467787321",
            "extra": "mean: 689.6297850777453 usec\nrounds: 1461"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1356.764461377121,
            "unit": "iter/sec",
            "range": "stddev: 0.0021027546772514035",
            "extra": "mean: 737.047607353303 usec\nrounds: 2040"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 828.0437290602646,
            "unit": "iter/sec",
            "range": "stddev: 0.002734240111885968",
            "extra": "mean: 1.2076656883023391 msec\nrounds: 1325"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_nullable_core",
            "value": 26616.618637171756,
            "unit": "iter/sec",
            "range": "stddev: 9.75408262661365e-7",
            "extra": "mean: 37.57051237918847 usec\nrounds: 26738"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 5852713.289942466,
            "unit": "iter/sec",
            "range": "stddev: 5.739966419719448e-9",
            "extra": "mean: 170.8609238929018 nsec\nrounds: 59877"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1098592.2002568168,
            "unit": "iter/sec",
            "range": "stddev: 4.7481784045111584e-8",
            "extra": "mean: 910.2558708916921 nsec\nrounds: 113637"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 773852.6245409813,
            "unit": "iter/sec",
            "range": "stddev: 6.433072730171182e-8",
            "extra": "mean: 1.2922357155448194 usec\nrounds: 79366"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 5494816.075856233,
            "unit": "iter/sec",
            "range": "stddev: 7.82746695289085e-9",
            "extra": "mean: 181.98971288487314 nsec\nrounds: 55866"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2017964.12318541,
            "unit": "iter/sec",
            "range": "stddev: 3.4010903176334205e-8",
            "extra": "mean: 495.54894881985194 nsec\nrounds: 185186"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 849791.7875204985,
            "unit": "iter/sec",
            "range": "stddev: 5.605414338243725e-8",
            "extra": "mean: 1.176758842207489 usec\nrounds: 88496"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 1972277.1076092124,
            "unit": "iter/sec",
            "range": "stddev: 3.3557245214803654e-8",
            "extra": "mean: 507.02814332886254 nsec\nrounds: 185186"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 5491181.750000466,
            "unit": "iter/sec",
            "range": "stddev: 5.500642594996529e-9",
            "extra": "mean: 182.11016235263494 nsec\nrounds: 55866"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 3105693.101650852,
            "unit": "iter/sec",
            "range": "stddev: 1.8728426382288803e-8",
            "extra": "mean: 321.9893168020755 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1246923.2882864743,
            "unit": "iter/sec",
            "range": "stddev: 5.421411488084916e-8",
            "extra": "mean: 801.9739541270202 nsec\nrounds: 133334"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2031122.7947783165,
            "unit": "iter/sec",
            "range": "stddev: 3.282241142903609e-8",
            "extra": "mean: 492.3385245699808 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 2430675.0419516815,
            "unit": "iter/sec",
            "range": "stddev: 2.867424402351371e-8",
            "extra": "mean: 411.40834654584773 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 3003713.882869138,
            "unit": "iter/sec",
            "range": "stddev: 2.10395569876613e-8",
            "extra": "mean: 332.92118989868743 nsec\nrounds: 192308"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "s@muelcolvin.com",
            "name": "Samuel Colvin",
            "username": "samuelcolvin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d17f70094ac18e47ea0a6c701bd55ff991f8fde6",
          "message": "String enum (#96)\n\n* implement EitherString\r\n\r\n* linting",
          "timestamp": "2022-06-20T23:41:39+01:00",
          "tree_id": "dea3acba44fa291d59a29f49a20abe7b43bc31c0",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/d17f70094ac18e47ea0a6c701bd55ff991f8fde6"
        },
        "date": 1655765064168,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 71881.59374825376,
            "unit": "iter/sec",
            "range": "stddev: 0.000018372903997315166",
            "extra": "mean: 13.911767225170815 usec\nrounds: 84034"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 30372.546873891748,
            "unit": "iter/sec",
            "range": "stddev: 0.00001583913704944447",
            "extra": "mean: 32.92446972432201 usec\nrounds: 37175"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 562352.1872253555,
            "unit": "iter/sec",
            "range": "stddev: 0.0000010720517427333814",
            "extra": "mean: 1.7782450619316776 usec\nrounds: 63699"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1477396.7433889078,
            "unit": "iter/sec",
            "range": "stddev: 8.81997596431041e-7",
            "extra": "mean: 676.8662544267976 nsec\nrounds: 181852"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 1151603.2880214981,
            "unit": "iter/sec",
            "range": "stddev: 5.369706960097923e-7",
            "extra": "mean: 868.354589120532 nsec\nrounds: 136987"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 8197107.519959055,
            "unit": "iter/sec",
            "range": "stddev: 8.27927878390669e-8",
            "extra": "mean: 121.9942519437912 nsec\nrounds: 88496"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 11618.016715928105,
            "unit": "iter/sec",
            "range": "stddev: 0.00003625692487028315",
            "extra": "mean: 86.07321063921495 usec\nrounds: 14926"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 25485.84476907428,
            "unit": "iter/sec",
            "range": "stddev: 0.00005642335050965902",
            "extra": "mean: 39.23746727098671 usec\nrounds: 31547"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3414.183797307697,
            "unit": "iter/sec",
            "range": "stddev: 0.00014595433141202372",
            "extra": "mean: 292.89577227463974 usec\nrounds: 3917"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6450.248793938138,
            "unit": "iter/sec",
            "range": "stddev: 0.00005234317801949257",
            "extra": "mean: 155.03277965646646 usec\nrounds: 7452"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_any_core_py",
            "value": 25437.65903559388,
            "unit": "iter/sec",
            "range": "stddev: 0.000028833582719609297",
            "extra": "mean: 39.3117935341747 usec\nrounds: 29942"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2766.1797794582435,
            "unit": "iter/sec",
            "range": "stddev: 0.00009251482919942851",
            "extra": "mean: 361.50940276045617 usec\nrounds: 3188"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 4985.490028154428,
            "unit": "iter/sec",
            "range": "stddev: 0.00005704886288725194",
            "extra": "mean: 200.5820880901829 usec\nrounds: 5676"
          },
          {
            "name": "tests/test_benchmarks.py::test_frozenset_of_ints_core",
            "value": 16166.904873090036,
            "unit": "iter/sec",
            "range": "stddev: 0.000019610359911977005",
            "extra": "mean: 61.85475870922635 usec\nrounds: 18658"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1406.9942707375437,
            "unit": "iter/sec",
            "range": "stddev: 0.0001045244463237739",
            "extra": "mean: 710.7349481073593 usec\nrounds: 1638"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_any_core",
            "value": 6951.41206671967,
            "unit": "iter/sec",
            "range": "stddev: 0.000056580613444042305",
            "extra": "mean: 143.85566420203514 usec\nrounds: 7844"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1627.0801755161986,
            "unit": "iter/sec",
            "range": "stddev: 0.00013670684139994776",
            "extra": "mean: 614.5978637363371 usec\nrounds: 1820"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1516.6584663166036,
            "unit": "iter/sec",
            "range": "stddev: 0.0015571823939317538",
            "extra": "mean: 659.3442242989789 usec\nrounds: 2461"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 1050.236499781747,
            "unit": "iter/sec",
            "range": "stddev: 0.0018778293507658414",
            "extra": "mean: 952.1664884126701 usec\nrounds: 1726"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_nullable_core",
            "value": 33485.60380038402,
            "unit": "iter/sec",
            "range": "stddev: 0.000010465875682272956",
            "extra": "mean: 29.863579762851156 usec\nrounds: 37881"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 8487401.059351847,
            "unit": "iter/sec",
            "range": "stddev: 6.590720086269008e-8",
            "extra": "mean: 117.821697479318 nsec\nrounds: 92602"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1419944.840615543,
            "unit": "iter/sec",
            "range": "stddev: 0.0000011351144678440226",
            "extra": "mean: 704.2527085535859 nsec\nrounds: 163962"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 911513.7119283936,
            "unit": "iter/sec",
            "range": "stddev: 8.821040317581647e-7",
            "extra": "mean: 1.0970762007347525 usec\nrounds: 103104"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 7638902.758882041,
            "unit": "iter/sec",
            "range": "stddev: 5.631240785769188e-8",
            "extra": "mean: 130.90885321684107 nsec\nrounds: 86965"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2871531.552935317,
            "unit": "iter/sec",
            "range": "stddev: 2.1426891024229106e-7",
            "extra": "mean: 348.24621689322646 nsec\nrounds: 192345"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 1097321.0112172337,
            "unit": "iter/sec",
            "range": "stddev: 6.835632114915392e-7",
            "extra": "mean: 911.3103547437446 nsec\nrounds: 126599"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2767310.1634279313,
            "unit": "iter/sec",
            "range": "stddev: 5.248598522915388e-7",
            "extra": "mean: 361.3617342991567 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 6677207.045630155,
            "unit": "iter/sec",
            "range": "stddev: 9.418861505282791e-8",
            "extra": "mean: 149.7632158425643 nsec\nrounds: 86957"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 3603359.829043906,
            "unit": "iter/sec",
            "range": "stddev: 3.331413024810016e-7",
            "extra": "mean: 277.51877343465605 nsec\nrounds: 46512"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1595282.734196531,
            "unit": "iter/sec",
            "range": "stddev: 5.965998843707167e-7",
            "extra": "mean: 626.8481307822176 nsec\nrounds: 188715"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2684015.0203689737,
            "unit": "iter/sec",
            "range": "stddev: 9.211709078649324e-7",
            "extra": "mean: 372.5761563966692 nsec\nrounds: 185220"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 3121214.114269941,
            "unit": "iter/sec",
            "range": "stddev: 2.6336732168804225e-7",
            "extra": "mean: 320.3881449298108 nsec\nrounds: 192345"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 4121437.460251743,
            "unit": "iter/sec",
            "range": "stddev: 1.2547343817109378e-7",
            "extra": "mean: 242.63379212822642 nsec\nrounds: 45249"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "em.jolibois@gmail.com",
            "name": "Eric Jolibois",
            "username": "PrettyWood"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f7b6ea7fbb3464b44331002b81b76f554977d663",
          "message": "add model field (#97)\n\n* add model field\r\n\r\n* add test for missing schema key\r\n\r\n* forgot some files\r\n\r\n* fix merge with main\r\n\r\n* remove demo.py\r\n\r\n* tweak\r\n\r\n* use get_as_req",
          "timestamp": "2022-06-20T23:49:12+01:00",
          "tree_id": "c212e5526d0a99b2a0eac4e725ffb73a9497fe68",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/f7b6ea7fbb3464b44331002b81b76f554977d663"
        },
        "date": 1655765535803,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 59872.91471910649,
            "unit": "iter/sec",
            "range": "stddev: 0.00002414046877206228",
            "extra": "mean: 16.70204306390453 usec\nrounds: 89286"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 29796.46423687011,
            "unit": "iter/sec",
            "range": "stddev: 0.000041166097018284396",
            "extra": "mean: 33.56102898821804 usec\nrounds: 42017"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 466202.7434479409,
            "unit": "iter/sec",
            "range": "stddev: 0.000002497284613198873",
            "extra": "mean: 2.144989522378807 usec\nrounds: 58477"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1334374.2311054661,
            "unit": "iter/sec",
            "range": "stddev: 0.0000016448274230547683",
            "extra": "mean: 749.4149517346918 nsec\nrounds: 169492"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 878858.4308077239,
            "unit": "iter/sec",
            "range": "stddev: 0.0000051219844864126474",
            "extra": "mean: 1.1378396849206038 usec\nrounds: 106383"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 6997366.9501227485,
            "unit": "iter/sec",
            "range": "stddev: 2.1577736404443719e-7",
            "extra": "mean: 142.91089878923435 nsec\nrounds: 76336"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 13049.387024695916,
            "unit": "iter/sec",
            "range": "stddev: 0.00005531569569013089",
            "extra": "mean: 76.63195199188313 usec\nrounds: 18726"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 23809.480898358517,
            "unit": "iter/sec",
            "range": "stddev: 0.00005055377882202784",
            "extra": "mean: 42.000075695431995 usec\nrounds: 35088"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3020.5770135340117,
            "unit": "iter/sec",
            "range": "stddev: 0.00015472354231487544",
            "extra": "mean: 331.0625736471526 usec\nrounds: 4250"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 5779.800298203017,
            "unit": "iter/sec",
            "range": "stddev: 0.0000905808972481176",
            "extra": "mean: 173.0163584217447 usec\nrounds: 8211"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_any_core_py",
            "value": 24725.31633944959,
            "unit": "iter/sec",
            "range": "stddev: 0.00001875770831975204",
            "extra": "mean: 40.44437637404404 usec\nrounds: 35842"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2491.2727737035193,
            "unit": "iter/sec",
            "range": "stddev: 0.00018620805238061996",
            "extra": "mean: 401.4012478101315 usec\nrounds: 3539"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 4323.686221939585,
            "unit": "iter/sec",
            "range": "stddev: 0.00018569863862862157",
            "extra": "mean: 231.28412855810907 usec\nrounds: 6464"
          },
          {
            "name": "tests/test_benchmarks.py::test_frozenset_of_ints_core",
            "value": 13126.711123379344,
            "unit": "iter/sec",
            "range": "stddev: 0.00010835665235595265",
            "extra": "mean: 76.18054443347572 usec\nrounds: 19231"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1193.6041411865504,
            "unit": "iter/sec",
            "range": "stddev: 0.0002699593010378962",
            "extra": "mean: 837.7987018425637 usec\nrounds: 1791"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_any_core",
            "value": 6126.440604505508,
            "unit": "iter/sec",
            "range": "stddev: 0.0001483265065234847",
            "extra": "mean: 163.22691503196484 usec\nrounds: 8921"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1500.96488800103,
            "unit": "iter/sec",
            "range": "stddev: 0.00017167441707434622",
            "extra": "mean: 666.2381032322414 usec\nrounds: 2073"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1381.3416682260574,
            "unit": "iter/sec",
            "range": "stddev: 0.001980595506768823",
            "extra": "mean: 723.9338557593915 usec\nrounds: 2891"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 784.0115005717113,
            "unit": "iter/sec",
            "range": "stddev: 0.0027781436157178654",
            "extra": "mean: 1.2754914937737356 msec\nrounds: 1847"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_nullable_core",
            "value": 26204.565774310424,
            "unit": "iter/sec",
            "range": "stddev: 0.000030429783909302162",
            "extra": "mean: 38.16128870871607 usec\nrounds: 39216"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 6816920.0284229405,
            "unit": "iter/sec",
            "range": "stddev: 2.7648257395356964e-7",
            "extra": "mean: 146.6938141903555 nsec\nrounds: 70922"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1050525.8681206875,
            "unit": "iter/sec",
            "range": "stddev: 0.0000025228296734606358",
            "extra": "mean: 951.9042132574434 nsec\nrounds: 163935"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 772395.517355718,
            "unit": "iter/sec",
            "range": "stddev: 0.0000015131478395417597",
            "extra": "mean: 1.2946734898506376 usec\nrounds: 116280"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 5913020.471236135,
            "unit": "iter/sec",
            "range": "stddev: 1.8667237746252897e-7",
            "extra": "mean: 169.11830508017786 nsec\nrounds: 76924"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2413470.945571845,
            "unit": "iter/sec",
            "range": "stddev: 5.213684430560166e-7",
            "extra": "mean: 414.34101447707127 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 870015.5270511034,
            "unit": "iter/sec",
            "range": "stddev: 0.0000015853326242466533",
            "extra": "mean: 1.1494047737165063 usec\nrounds: 133334"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2272595.7831329755,
            "unit": "iter/sec",
            "range": "stddev: 0.0000011642353293027467",
            "extra": "mean: 440.0254578583752 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 5864811.376775368,
            "unit": "iter/sec",
            "range": "stddev: 4.225138106254651e-7",
            "extra": "mean: 170.50846749475718 nsec\nrounds: 90910"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 3421193.1931149135,
            "unit": "iter/sec",
            "range": "stddev: 3.227939312958255e-7",
            "extra": "mean: 292.2956826911891 nsec\nrounds: 50506"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1307507.2469089276,
            "unit": "iter/sec",
            "range": "stddev: 0.0000018989615939219656",
            "extra": "mean: 764.8141166055772 nsec\nrounds: 172414"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2213719.1748740324,
            "unit": "iter/sec",
            "range": "stddev: 3.683795146341519e-7",
            "extra": "mean: 451.72848089760845 nsec\nrounds: 161291"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 2511698.545764671,
            "unit": "iter/sec",
            "range": "stddev: 0.000002096819053025126",
            "extra": "mean: 398.1369506652512 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 3152370.276060701,
            "unit": "iter/sec",
            "range": "stddev: 0.000001533446789991574",
            "extra": "mean: 317.22161815623707 nsec\nrounds: 196079"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "s@muelcolvin.com",
            "name": "Samuel Colvin",
            "username": "samuelcolvin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e1b75cf6c27a5f87a8127f27c9be85c940342fc5",
          "message": "improvements to `with_prefix_location` (#99)\n\n* improvements to with_prefix_location as suggested in #35\r\n\r\n* avoid clone and better benches",
          "timestamp": "2022-06-21T14:11:00+01:00",
          "tree_id": "02383881bde4eb2e20576beb1763175d2ac2c233",
          "url": "https://github.com/samuelcolvin/pydantic-core/commit/e1b75cf6c27a5f87a8127f27c9be85c940342fc5"
        },
        "date": 1655817314944,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_python",
            "value": 71782.31931666483,
            "unit": "iter/sec",
            "range": "stddev: 5.258123753152076e-7",
            "extra": "mean: 13.931007099234842 usec\nrounds: 73530"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkSimpleModel::test_core_json",
            "value": 35358.96203932916,
            "unit": "iter/sec",
            "range": "stddev: 8.525259489626276e-7",
            "extra": "mean: 28.281373160437155 usec\nrounds: 35336"
          },
          {
            "name": "tests/test_benchmarks.py::test_bool_core",
            "value": 549309.5515397391,
            "unit": "iter/sec",
            "range": "stddev: 5.561737073328622e-8",
            "extra": "mean: 1.8204671613606858 usec\nrounds: 55249"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_dict",
            "value": 1481080.8225757044,
            "unit": "iter/sec",
            "range": "stddev: 4.4687993147992045e-8",
            "extra": "mean: 675.1825995969067 nsec\nrounds: 153847"
          },
          {
            "name": "tests/test_benchmarks.py::test_small_class_core_model",
            "value": 1138250.7006222692,
            "unit": "iter/sec",
            "range": "stddev: 4.384551495947014e-8",
            "extra": "mean: 878.541080144877 nsec\nrounds: 116280"
          },
          {
            "name": "tests/test_benchmarks.py::test_core_string_lax",
            "value": 8295646.708399177,
            "unit": "iter/sec",
            "range": "stddev: 3.8988658001036806e-9",
            "extra": "mean: 120.54515279529437 nsec\nrounds: 84034"
          },
          {
            "name": "tests/test_benchmarks.py::test_recursive_model_core",
            "value": 14568.191404093039,
            "unit": "iter/sec",
            "range": "stddev: 0.0000012576366411132122",
            "extra": "mean: 68.64270054270723 usec\nrounds: 14750"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_dict_models_core",
            "value": 28742.643209612983,
            "unit": "iter/sec",
            "range": "stddev: 9.536960769187894e-7",
            "extra": "mean: 34.79151143850089 usec\nrounds: 29069"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_py",
            "value": 3562.708674868575,
            "unit": "iter/sec",
            "range": "stddev: 0.0000020852500295460695",
            "extra": "mean: 280.6853131309955 usec\nrounds: 3564"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_ints_core_json",
            "value": 6741.4463984824515,
            "unit": "iter/sec",
            "range": "stddev: 0.0000020979221632703643",
            "extra": "mean: 148.3361197123969 usec\nrounds: 6808"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_any_core_py",
            "value": 28390.045249786916,
            "unit": "iter/sec",
            "range": "stddev: 8.592013453643294e-7",
            "extra": "mean: 35.22361416481031 usec\nrounds: 28818"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core",
            "value": 2933.5429849237853,
            "unit": "iter/sec",
            "range": "stddev: 0.000003686897641495027",
            "extra": "mean: 340.8847271504973 usec\nrounds: 2965"
          },
          {
            "name": "tests/test_benchmarks.py::test_set_of_ints_core_json",
            "value": 5275.1516009271545,
            "unit": "iter/sec",
            "range": "stddev: 0.000002154270929912324",
            "extra": "mean: 189.56801162344624 usec\nrounds: 5334"
          },
          {
            "name": "tests/test_benchmarks.py::test_frozenset_of_ints_core",
            "value": 16003.332649261109,
            "unit": "iter/sec",
            "range": "stddev: 0.000001236517849431389",
            "extra": "mean: 62.48698454981945 usec\nrounds: 16181"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core",
            "value": 1429.964067658337,
            "unit": "iter/sec",
            "range": "stddev: 0.00002303622347680204",
            "extra": "mean: 699.3182714287135 usec\nrounds: 1470"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_any_core",
            "value": 7318.787273025418,
            "unit": "iter/sec",
            "range": "stddev: 0.0000015497206999764797",
            "extra": "mean: 136.63465854318005 usec\nrounds: 7386"
          },
          {
            "name": "tests/test_benchmarks.py::test_dict_of_ints_core_json",
            "value": 1712.5875963878136,
            "unit": "iter/sec",
            "range": "stddev: 0.000004744482314057904",
            "extra": "mean: 583.911738067704 usec\nrounds: 1718"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_dict",
            "value": 1490.0566637731768,
            "unit": "iter/sec",
            "range": "stddev: 0.0022000834019697806",
            "extra": "mean: 671.1154174954413 usec\nrounds: 2412"
          },
          {
            "name": "tests/test_benchmarks.py::test_many_models_core_model",
            "value": 891.3535128919323,
            "unit": "iter/sec",
            "range": "stddev: 0.0029617820502682627",
            "extra": "mean: 1.1218893351926913 msec\nrounds: 1614"
          },
          {
            "name": "tests/test_benchmarks.py::test_list_of_nullable_core",
            "value": 31252.856411406527,
            "unit": "iter/sec",
            "range": "stddev: 7.230895015107945e-7",
            "extra": "mean: 31.997075302052217 usec\nrounds: 32363"
          },
          {
            "name": "tests/test_benchmarks.py::test_bytes_core",
            "value": 8155989.17557863,
            "unit": "iter/sec",
            "range": "stddev: 5.195545586448954e-9",
            "extra": "mean: 122.60928484232845 nsec\nrounds: 81968"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_python",
            "value": 1374426.565346783,
            "unit": "iter/sec",
            "range": "stddev: 3.786531533509473e-8",
            "extra": "mean: 727.5761580957151 nsec\nrounds: 149254"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_model_core_json",
            "value": 891957.6528854895,
            "unit": "iter/sec",
            "range": "stddev: 5.016452537640527e-8",
            "extra": "mean: 1.1211294580689883 usec\nrounds: 91744"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_raw",
            "value": 7587887.942117869,
            "unit": "iter/sec",
            "range": "stddev: 4.5671407279058675e-9",
            "extra": "mean: 131.78897838618562 nsec\nrounds: 74627"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_str",
            "value": 2602768.792862925,
            "unit": "iter/sec",
            "range": "stddev: 3.264569167145999e-8",
            "extra": "mean: 384.2062355839113 nsec\nrounds: 196079"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future",
            "value": 1087520.5240446003,
            "unit": "iter/sec",
            "range": "stddev: 5.517637602241757e-8",
            "extra": "mean: 919.5228760193029 nsec\nrounds: 111112"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateTime::test_core_future_str",
            "value": 2434777.9679799033,
            "unit": "iter/sec",
            "range": "stddev: 3.006494653684521e-8",
            "extra": "mean: 410.7150685407429 nsec\nrounds: 192308"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_date",
            "value": 7528558.0907035675,
            "unit": "iter/sec",
            "range": "stddev: 4.709529567601815e-9",
            "extra": "mean: 132.8275597999717 nsec\nrounds: 76924"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_str",
            "value": 4155755.6536433133,
            "unit": "iter/sec",
            "range": "stddev: 8.483984267290388e-9",
            "extra": "mean: 240.63012442119398 nsec\nrounds: 42195"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime",
            "value": 1662527.8628988431,
            "unit": "iter/sec",
            "range": "stddev: 5.006345828101765e-8",
            "extra": "mean: 601.4936785815565 nsec\nrounds: 172414"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_date_from_datetime_str",
            "value": 2578044.162362456,
            "unit": "iter/sec",
            "range": "stddev: 2.2087158543159644e-8",
            "extra": "mean: 387.8909502790683 nsec\nrounds: 188680"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future",
            "value": 3084682.7335881195,
            "unit": "iter/sec",
            "range": "stddev: 1.65763247707562e-8",
            "extra": "mean: 324.18244803885733 nsec\nrounds: 185186"
          },
          {
            "name": "tests/test_benchmarks.py::TestBenchmarkDateX::test_core_future_str",
            "value": 3905322.8705559107,
            "unit": "iter/sec",
            "range": "stddev: 1.6268422764451082e-8",
            "extra": "mean: 256.06077477991 nsec\nrounds: 192308"
          }
        ]
      }
    ]
  }
}