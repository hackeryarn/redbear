# Redbear

This was a quick test of an idea inspired by https://github.com/cgdeboer/blackbear.

`blackbear` shows that running many operations on small data sets can perform better in raw python than in the popular data science libraries. This inspired the theory that using the simplest conversion of types from Python to Rust would yield similar results.

Running tests with a few `blackbear` inspired functions proved the simple approach ends up not reaching any meaningful optimization. When running many operations on a small set, the performance was better than `polars` and `pandas`, but far behind `blackbear`. And when running on large sets the performance became very bad over all.

## Benchmarks

``` sh
Redbear 100000 X 5 Element-wise ops on collection of 10:
  0.170 seconds
Redbear 1000000 X 5 Element-wise ops on collection of 10:
  1.742 seconds
Redbear 10000 X 5 Element-wise ops on collection of 1000:
  1.862 seconds
Redbear 100000 X 5 Element-wise ops on collection of 1000:
  18.331 seconds
-----------
Polars 100000 X 5 Element-wise ops on collection of 10:
  5.903 seconds
Polars 1000000 X 5 Element-wise ops on collection of 10:
  56.002 seconds
Polars 10000 X 5 Element-wise ops on collection of 1000:
  0.541 seconds
Polars 100000 X 5 Element-wise ops on collection of 1000:
  5.383 seconds
-----------
Pandas 100000 X 5 Element-wise ops on collection of 10:
  18.414 seconds
Pandas 1000000 X 5 Element-wise ops on collection of 10:
  182.312 seconds
Pandas 10000 X 5 Element-wise ops on collection of 1000:
  2.021 seconds
Pandas 100000 X 5 Element-wise ops on collection of 1000:
  23.436 seconds
-----------
Numpy 100000 X 5 Element-wise ops on collection of 10:
  0.205 seconds
Numpy 1000000 X 5 Element-wise ops on collection of 10:
  1.997 seconds
Numpy 10000 X 5 Element-wise ops on collection of 1000:
  0.037 seconds
Numpy 100000 X 5 Element-wise ops on collection of 1000:
  0.348 seconds
-----------
Blackbear 100000 X 5 Element-wise ops on collection of 10:
  0.313 seconds
Blackbear 1000000 X 5 Element-wise ops on collection of 10:
  3.072 seconds
Blackbear 10000 X 5 Element-wise ops on collection of 1000:
  2.637 seconds
Blackbear 100000 X 5 Element-wise ops on collection of 1000:
  27.970 seconds
```

