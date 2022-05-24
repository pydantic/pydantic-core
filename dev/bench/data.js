window.BENCHMARK_DATA = {
  "lastUpdate": 1653389846718,
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
      }
    ]
  }
}