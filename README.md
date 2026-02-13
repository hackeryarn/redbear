# Redbear
[![Build Status](https://img.shields.io/github/actions/workflow/status/hackeryarn/redbear/CI.yml)](https://github.com/hackeryarn/redbear/actions/workflows/CI.yml)
[![PyPI - Version](https://img.shields.io/pypi/v/redbear)](https://pypi.org/project/redbear/)
[![GitHub License](https://img.shields.io/github/license/hackeryarn/redbear)](https://opensource.org/license/gpl-3-0)
[![PyPI - Downloads](https://img.shields.io/pypi/dm/redbear)](https://pypi.org/project/redbear/)


A high-performance Python extension for fast element-wise dictionary operations. Redbear provides a minimal, purpose-built `RedDict` class that delivers significant performance improvements over traditional data science libraries for specific use cases involving numeric dictionaries.

## Purpose

Redbear is inspired by [blackbear](https://github.com/cgdeboer/blackbear) and builds on the concepts introduced there. By moving the implementation to Rust while keeping things as simple as possible, Redbear achieves far better performance across the board.

The library is designed for scenarios where:
- You work with numeric dictionaries (string keys, float values)
- You need fast element-wise operations (add, subtract, multiply)
- Performance matters more than generality

After heavy optimizations using PyO3 and Arc-based internal sharing, Redbear achieves excellent performance while hiding all the complexity from the library users.

## Installation

```bash
pip install redbear
```

## Usage

```python
import redbear as rb

# Create a RedDict from a regular Python dict
data = {"a": 1.0, "b": 2.0, "c": 3.0}
rd = rb.RedDict(data)

# Scalar operations (creates a new RedDict)
rd_plus_5 = rd.add_scalar(5.0)  # {"a": 6.0, "b": 7.0, "c": 8.0}
rd_minus_2 = rd.subtract_scalar(2.0)  # {"a": -1.0, "b": 0.0, "c": 1.0}

# Element-wise operations between two RedDicts
other = rb.RedDict({"a": 10.0, "b": 20.0, "c": 30.0})
result = rd.add(other)  # {"a": 11.0, "b": 22.0, "c": 33.0}
result = rd.subtract(other)  # {"a": -9.0, "b": -18.0, "c": -27.0}
result = rd.multiply(other)  # {"a": 10.0, "b": 40.0, "c": 90.0}

# Get the underlying dict back
plain_dict = rd.to_dict  # {"a": 1.0, "b": 2.0, "c": 3.0}
```

## Benchmarks

Run benchmarks with:

```bash
pip install -e ".[dev]"
python -m benchmarks.redbear_way
python -m benchmarks.numpy_way
python -m benchmarks.blackbear_way
python -m benchmarks.polars_way
python -m benchmarks.pandas_way
```

### Best Case (keys identical)

| Library | Operations | Collection Size | Time (seconds) |
|---------|------------|-----------------|----------------|
| Redbear | 100000 x 5 | 10 | 0.056 |
| Redbear | 1000000 x 5 | 10 | 0.583 |
| Redbear | 10000 x 5 | 1000 | 0.015 |
| Redbear | 100000 x 5 | 1000 | 0.145 |
| Numpy | 100000 x 5 | 10 | 0.285 |
| Numpy | 1000000 x 5 | 10 | 2.436 |
| Numpy | 10000 x 5 | 1000 | 0.039 |
| Numpy | 100000 x 5 | 1000 | 0.371 |
| Blackbear | 100000 x 5 | 10 | 0.371 |
| Blackbear | 1000000 x 5 | 10 | 3.755 |
| Blackbear | 10000 x 5 | 1000 | 3.643 |
| Blackbear | 100000 x 5 | 1000 | 36.307 |
| Polars | 100000 x 5 | 10 | 1.072 |
| Polars | 1000000 x 5 | 10 | 11.147 |
| Polars | 10000 x 5 | 1000 | 0.132 |
| Polars | 100000 x 5 | 1000 | 1.219 |
| Pandas | 100000 x 5 | 10 | 13.466 |
| Pandas | 1000000 x 5 | 10 | 130.837 |
| Pandas | 10000 x 5 | 1000 | 1.339 |
| Pandas | 100000 x 5 | 1000 | 13.432 |

### Worst Case (different key order)

| Library | Operations | Collection Size | Time (seconds) |
|---------|------------|-----------------|----------------|
| Redbear | 100000 x 5 | 10 | 0.561 |
| Redbear | 1000000 x 5 | 10 | 5.432 |
| Redbear | 10000 x 5 | 1000 | 0.283 |
| Redbear | 100000 x 5 | 1000 | 2.842 |

## Development

Start developing with [maturin](https://www.maturin.rs/):

```bash
maturin develop
```
