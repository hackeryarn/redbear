# Redbear

This is a test of an idea inspired by https://github.com/cgdeboer/blackbear.

`blackbear` shows that running many operations on small data sets can perform better in raw python than in the popular data science libraries. This inspired the theory that using the simplest conversion of types from Python to Rust would yield similar results.

After some heavy optimizations, we can heavily improve performance at the cost of some code complexity. None of the complexity gets exposed to the end user, so it makes a good tradeoff.

## Benchmarks

``` sh
# Merging dict derived from the same original calculation (best case scenario)
Redbear 100000 X 5 Element-wise ops on collection of 10:
  0.058 seconds
Redbear 1000000 X 5 Element-wise ops on collection of 10:
  0.600 seconds
Redbear 10000 X 5 Element-wise ops on collection of 1000:
  0.017 seconds
Redbear 100000 X 5 Element-wise ops on collection of 1000:
  0.149 seconds
-----------
# Merging dicts with different key order (worst case scenario)
Redbear 100000 X 5 Element-wise ops on collection of 10:
  0.111 seconds
Redbear 1000000 X 5 Element-wise ops on collection of 10:
  1.078 seconds
Redbear 10000 X 5 Element-wise ops on collection of 1000:
  0.578 seconds
Redbear 100000 X 5 Element-wise ops on collection of 1000:
  5.884 seconds
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

