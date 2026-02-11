//! A high-performance dictionary of numeric values with aligned operations.
//!
//! # Architecture
//!
//! Keys and values are stored in separate parallel arrays for cache efficiency.
//! An index maps keys to their positions in the values array.
//!
//! # Immutability
//!
//! All operations return new instances. Internal data uses `Arc` for cheap cloning
//! with copy-on-write semantics via `Arc::make_mut`.
use std::collections::HashMap;
use std::sync::Arc;

use pyo3::{prelude::*, types::PyDict};

#[pyclass]
#[derive(Clone)]
struct RedDict {
    /// Mapping from key -> index into `values`.
    index: Arc<HashMap<String, usize>>,
    /// Packed numeric values, aligned with `keys`.
    values: Arc<Vec<f64>>,
}

#[pymethods]
impl RedDict {
    /// Creates a new `RedDict` from a Python dictionary.
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> import redbear as rb
    /// >>> d = rb.RedDict({"x": 1.0, "y": 2.0})
    /// >>> d.to_dict
    /// {'x': 1.0, 'y': 2.0}
    /// ```
    #[new]
    fn new(dict: &Bound<PyDict>) -> PyResult<Self> {
        // First extract into a temporary HashMap via PyO3, then
        // build our split key/index/value representation.
        let extracted: HashMap<String, f64> = dict.extract()?;

        let mut values = Vec::with_capacity(extracted.len());
        let mut index = HashMap::with_capacity(extracted.len());

        for (pos, (k, v)) in extracted.into_iter().enumerate() {
            values.push(v);
            index.insert(k, pos);
        }

        Ok(Self {
            index: Arc::new(index),
            values: Arc::new(values),
        })
    }

    /// Adds a scalar value (single value) to every value in the dictionary.
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> d = rb.RedDict({"a": 1.0, "b": 2.0})
    /// >>> d.add_scalar(10.0).to_dict
    /// {'a': 11.0, 'b': 12.0}
    /// ```
    #[must_use]
    fn add_scalar(&self, value: f64) -> Self {
        let mut new = self.clone();
        Arc::make_mut(&mut new.values)
            .iter_mut()
            .for_each(|val| *val += value);
        new
    }

    /// Adds values (d1 + d2), aligned on d1s keys. Only keys from d1 are
    /// considered, if key from d1 is absent from d2, a fill value can optionally
    /// be used as the argument for +.
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> d1 = rb.RedDict({"a": 1.0, "b": 2.0})
    /// >>> d2 = rb.RedDict({"b": 10.0})
    /// >>> d1.add(d2).to_dict
    /// {'a': 1.0, 'b': 12.0}
    /// >>> d1.add(d2, fill=5.0).to_dict
    /// {'a': 6.0, 'b': 12.0}
    /// ```
    #[pyo3(signature = (other, fill=0.0))]
    fn add(&self, other: &Bound<Self>, fill: f64) -> PyResult<Self> {
        let other_ref = other.borrow();
        Ok(merge(self, &other_ref, fill, |a, b| a + b))
    }

    /// Subtracts a scalar value (single value) to every value in the dictionary.
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> d = rb.RedDict({"a": 5.0, "b": 10.0})
    /// >>> d.subtract_scalar(3.0).to_dict
    /// {'a': 2.0, 'b': 7.0}
    /// ```
    #[must_use]
    fn subtract_scalar(&self, value: f64) -> Self {
        let mut new = self.clone();
        Arc::make_mut(&mut new.values)
            .iter_mut()
            .for_each(|val| *val -= value);
        new
    }

    /// Subtracts values (d1 - d2), aligned on d1s keys. Only keys from d1 are
    /// considered, if key from d1 is absent from d2, a fill value can optionally
    /// be used as the argument for -.
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> d1 = rb.RedDict({"a": 10.0, "b": 5.0})
    /// >>> d2 = rb.RedDict({"b": 2.0})
    /// >>> d1.subtract(d2).to_dict
    /// {'a': 10.0, 'b': 3.0}
    /// ```
    #[pyo3(signature = (other, fill=0.0))]
    fn subtract(&self, other: &Bound<Self>, fill: f64) -> PyResult<Self> {
        let other_ref = other.borrow();
        Ok(merge(self, &other_ref, fill, |a, b| a - b))
    }

    /// Multiplies a scalar value (single value) to every value in the dictionary.
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> d = rb.RedDict({"a": 2.0, "b": 5.0})
    /// >>> d.multiply_scalar(3.0).to_dict
    /// {'a': 6.0, 'b': 15.0}
    /// ```
    #[must_use]
    fn multiply_scalar(&self, value: f64) -> Self {
        let mut new = self.clone();
        Arc::make_mut(&mut new.values)
            .iter_mut()
            .for_each(|val| *val *= value);
        new
    }

    /// Multiplies values (d1 * d2), aligned on d1s keys. Only keys from d1 are
    /// considered, if key from d1 is absent from d2, a fill value can optionally
    /// be used as the argument for *.
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> d1 = rb.RedDict({"a": 2.0, "b": 3.0})
    /// >>> d2 = rb.RedDict({"b": 10.0})
    /// >>> d1.multiply(d2).to_dict
    /// {'a': 2.0, 'b': 30.0}
    /// ```
    #[pyo3(signature = (other, fill=1.0))]
    fn multiply(&self, other: &Bound<Self>, fill: f64) -> PyResult<Self> {
        let other_ref = other.borrow();
        Ok(merge(self, &other_ref, fill, |a, b| a * b))
    }

    /// Divides a scalar value (single value) to every value in the dictionary.
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> d = rb.RedDict({"a": 10.0, "b": 6.0})
    /// >>> d.divide_scalar(2.0).to_dict
    /// {'a': 5.0, 'b': 3.0}
    /// ```
    #[must_use]
    fn divide_scalar(&self, value: f64) -> Self {
        let mut new = self.clone();
        Arc::make_mut(&mut new.values)
            .iter_mut()
            .for_each(|val| *val /= value);
        new
    }

    /// Divides values (d1 / d2), aligned on d1s keys. Only keys from d1 are
    /// considered, if key from d1 is absent from d2, a fill value can optionally
    /// be used as the argument for /.
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> d1 = rb.RedDict({"a": 10.0, "b": 6.0})
    /// >>> d2 = rb.RedDict({"b": 2.0})
    /// >>> d1.divide(d2).to_dict
    /// {'a': 10.0, 'b': 3.0}
    /// ```
    #[pyo3(signature = (other, fill=1.0))]
    fn divide(&self, other: &Bound<Self>, fill: f64) -> PyResult<Self> {
        let other_ref = other.borrow();
        Ok(merge(self, &other_ref, fill, |a, b| a / b))
    }

    /// Sum of values.
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> d = rb.RedDict({"a": 1.0, "b": 2.0, "c": 3.0})
    /// >>> d.sum()
    /// 6.0
    /// ```
    fn sum(&self) -> f64 {
        self.values.iter().sum()
    }

    /// Product of values.
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> d = rb.RedDict({"a": 2.0, "b": 3.0, "c": 4.0})
    /// >>> d.product()
    /// 24.0
    /// ```
    fn product(&self) -> f64 {
        self.values.iter().product()
    }

    /// Sets all values to passed in value
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> d = rb.RedDict({"a": 1.0, "b": 2.0})
    /// >>> d.reset(99.0).to_dict
    /// {'a': 99.0, 'b': 99.0}
    /// ```
    #[must_use]
    fn reset(&self, value: f64) -> Self {
        let mut new = self.clone();
        Arc::make_mut(&mut new.values)
            .iter_mut()
            .for_each(|val| *val = value);
        new
    }

    #[getter]
    /// Returns the underlying dictionary.
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> d = rb.RedDict({"x": 42.0})
    /// >>> d.to_dict
    /// {'x': 42.0}
    /// ```
    fn to_dict(&self) -> HashMap<String, f64> {
        let mut map = HashMap::with_capacity(self.values.len());
        for (k, v) in self.index.iter() {
            map.insert(k.clone(), self.values[*v]);
        }
        map
    }
}

/// Shared implementation for binary element-wise operations.
///
/// `fill` is the value used when `other` is missing a key present in `self`.
fn merge<F>(this: &RedDict, other: &RedDict, fill: f64, f: F) -> RedDict
where
    F: Fn(&f64, &f64) -> f64,
{
    let mut new = this.clone();
    let new_vals = Arc::make_mut(&mut new.values);

    if Arc::ptr_eq(&this.index, &other.index)
        || (this.index.len() == other.index.len() && this.index == other.index)
    {
        for (nv, ov) in new_vals.iter_mut().zip(other.values.iter()) {
            *nv = f(nv, ov);
        }
    } else {
        for (key, &i) in this.index.iter() {
            let rhs = other
                .index
                .get(key)
                .map(|&j| other.values[j])
                .unwrap_or(fill);
            new_vals[i] = f(&new_vals[i], &rhs);
        }
    }

    new
}

/// A Python module implemented in Rust.
#[pymodule]
fn redbear(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<RedDict>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::{types::PyDict, Py, Python};

    fn make_dict(py: Python<'_>, entries: &[(&str, f64)]) -> RedDict {
        let dict = PyDict::new(py);
        for (k, v) in entries {
            dict.set_item(*k, *v).unwrap();
        }
        RedDict::new(&dict).unwrap()
    }

    #[test]
    fn test_new_from_empty_dict() {
        Python::initialize();
        Python::attach(|py| {
            let dict = PyDict::new(py);
            let rd = RedDict::new(&dict).unwrap();
            assert_eq!(rd.to_dict().len(), 0);
        });
    }

    #[test]
    fn test_new_from_single_entry() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("x", 42.0)]);
            let map = rd.to_dict();
            assert_eq!(map.get("x"), Some(&42.0));
            assert_eq!(map.len(), 1);
        });
    }

    #[test]
    fn test_new_from_multiple_entries() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("a", 1.0), ("b", 2.0), ("c", 3.0)]);
            let map = rd.to_dict();
            assert_eq!(map.get("a"), Some(&1.0));
            assert_eq!(map.get("b"), Some(&2.0));
            assert_eq!(map.get("c"), Some(&3.0));
        });
    }

    #[test]
    fn test_add_scalar() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("a", 1.0), ("b", -2.0)]);
            let result = rd.add_scalar(3.0);
            assert_eq!(result.to_dict().get("a"), Some(&4.0));
            assert_eq!(result.to_dict().get("b"), Some(&1.0));
        });
    }

    #[test]
    fn test_subtract_scalar() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("a", 5.0), ("b", 3.0)]);
            let result = rd.subtract_scalar(2.0);
            assert_eq!(result.to_dict().get("a"), Some(&3.0));
            assert_eq!(result.to_dict().get("b"), Some(&1.0));
        });
    }

    #[test]
    fn test_add_scalar_negative() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("a", 10.0)]);
            let result = rd.add_scalar(-5.0);
            assert_eq!(result.to_dict().get("a"), Some(&5.0));
        });
    }

    #[test]
    fn test_operations_return_new_instance() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("a", 1.0)]);
            let added = rd.add_scalar(1.0);
            let subtracted = rd.subtract_scalar(1.0);
            assert_eq!(rd.to_dict().get("a"), Some(&1.0));
            assert_eq!(added.to_dict().get("a"), Some(&2.0));
            assert_eq!(subtracted.to_dict().get("a"), Some(&0.0));
        });
    }

    #[test]
    fn test_add_uses_fill_for_missing_keys() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 1.0), ("b", 2.0)]);
            let right = make_dict(py, &[("b", 10.0), ("c", 100.0)]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let result = left.add(py_right.bind(py), 5.0).unwrap();
            assert_eq!(result.to_dict().get("a"), Some(&6.0)); // fill used
            assert_eq!(result.to_dict().get("b"), Some(&12.0)); // right value used
            assert!(!result.to_dict().contains_key("c"));
        });
    }

    #[test]
    fn test_subtract_uses_fill_for_missing_keys() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 10.0), ("b", 5.0)]);
            let right = make_dict(py, &[("b", 2.0)]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let result = left.subtract(py_right.bind(py), 3.0).unwrap();
            assert_eq!(result.to_dict().get("a"), Some(&7.0)); // fill used
            assert_eq!(result.to_dict().get("b"), Some(&3.0)); // right value used
        });
    }

    #[test]
    fn test_multiply_uses_fill_for_missing_keys() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 2.0), ("b", 3.0)]);
            let right = make_dict(py, &[("b", 10.0)]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let result = left.multiply(py_right.bind(py), 1.0).unwrap();
            assert_eq!(result.to_dict().get("a"), Some(&2.0)); // fill used
            assert_eq!(result.to_dict().get("b"), Some(&30.0)); // right value used
        });
    }

    #[test]
    fn test_add_default_fill_is_zero() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 1.0)]);
            let right = make_dict(py, &[]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let result = left.add(py_right.bind(py), 0.0).unwrap();
            assert_eq!(result.to_dict().get("a"), Some(&1.0));
        });
    }

    #[test]
    fn test_subtract_default_fill_is_zero() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 5.0)]);
            let right = make_dict(py, &[]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let result = left.subtract(py_right.bind(py), 0.0).unwrap();
            assert_eq!(result.to_dict().get("a"), Some(&5.0));
        });
    }

    #[test]
    fn test_multiply_default_fill_is_one() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 7.0)]);
            let right = make_dict(py, &[]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let result = left.multiply(py_right.bind(py), 1.0).unwrap();
            assert_eq!(result.to_dict().get("a"), Some(&7.0));
        });
    }

    #[test]
    fn test_add_fast_path_identical_keys() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 1.0), ("b", 2.0)]);
            let right = make_dict(py, &[("a", 10.0), ("b", 20.0)]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let result = left.add(py_right.bind(py), 0.0).unwrap();
            assert_eq!(result.to_dict().get("a"), Some(&11.0));
            assert_eq!(result.to_dict().get("b"), Some(&22.0));
        });
    }

    #[test]
    fn test_subtract_fast_path_identical_keys() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 10.0), ("b", 20.0)]);
            let right = make_dict(py, &[("a", 3.0), ("b", 5.0)]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let result = left.subtract(py_right.bind(py), 0.0).unwrap();
            assert_eq!(result.to_dict().get("a"), Some(&7.0));
            assert_eq!(result.to_dict().get("b"), Some(&15.0));
        });
    }

    #[test]
    fn test_multiply_fast_path_identical_keys() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 2.0), ("b", 3.0)]);
            let right = make_dict(py, &[("a", 5.0), ("b", 4.0)]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let result = left.multiply(py_right.bind(py), 1.0).unwrap();
            assert_eq!(result.to_dict().get("a"), Some(&10.0));
            assert_eq!(result.to_dict().get("b"), Some(&12.0));
        });
    }

    #[test]
    fn test_does_not_modify_operands() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 1.0), ("b", 2.0)]);
            let right = make_dict(py, &[("b", 10.0)]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let _ = left.add(py_right.bind(py), 5.0).unwrap();
            let _ = left.subtract(py_right.bind(py), 0.0).unwrap();
            let _ = left.multiply(py_right.bind(py), 1.0).unwrap();
            assert_eq!(left.to_dict().get("a"), Some(&1.0));
            assert_eq!(left.to_dict().get("b"), Some(&2.0));
            assert_eq!(right.to_dict().get("b"), Some(&10.0));
        });
    }

    #[test]
    fn test_chained_operations() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("x", 1.0)]);
            let py_rd = Py::new(py, rd.clone()).unwrap();
            let result = rd
                .add_scalar(2.0)
                .subtract_scalar(1.0)
                .add(py_rd.bind(py), 0.0)
                .unwrap();
            assert_eq!(result.to_dict().get("x"), Some(&3.0));
        });
    }

    #[test]
    fn test_multiply_scalar() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("a", 2.0), ("b", 5.0)]);
            let result = rd.multiply_scalar(3.0);
            assert_eq!(result.to_dict().get("a"), Some(&6.0));
            assert_eq!(result.to_dict().get("b"), Some(&15.0));
        });
    }

    #[test]
    fn test_multiply_scalar_by_zero() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("x", 42.0)]);
            let result = rd.multiply_scalar(0.0);
            assert_eq!(result.to_dict().get("x"), Some(&0.0));
        });
    }

    #[test]
    fn test_divide_scalar() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("a", 10.0), ("b", 6.0)]);
            let result = rd.divide_scalar(2.0);
            assert_eq!(result.to_dict().get("a"), Some(&5.0));
            assert_eq!(result.to_dict().get("b"), Some(&3.0));
        });
    }

    #[test]
    fn test_divide_scalar_by_fraction() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("x", 1.0)]);
            let result = rd.divide_scalar(0.5);
            assert_eq!(result.to_dict().get("x"), Some(&2.0));
        });
    }

    #[test]
    fn test_divide() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 10.0), ("b", 6.0)]);
            let right = make_dict(py, &[("b", 2.0)]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let result = left.divide(py_right.bind(py), 1.0).unwrap();
            assert_eq!(result.to_dict().get("a"), Some(&10.0));
            assert_eq!(result.to_dict().get("b"), Some(&3.0));
        });
    }

    #[test]
    fn test_divide_default_fill_is_one() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 7.0)]);
            let right = make_dict(py, &[]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let result = left.divide(py_right.bind(py), 1.0).unwrap();
            assert_eq!(result.to_dict().get("a"), Some(&7.0));
        });
    }

    #[test]
    fn test_sum() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("a", 1.0), ("b", 2.0), ("c", 3.0)]);
            assert_eq!(rd.sum(), 6.0);
        });
    }

    #[test]
    fn test_sum_empty() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[]);
            assert_eq!(rd.sum(), 0.0);
        });
    }

    #[test]
    fn test_product() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("a", 2.0), ("b", 3.0), ("c", 4.0)]);
            assert_eq!(rd.product(), 24.0);
        });
    }

    #[test]
    fn test_product_single_element() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("x", 5.0)]);
            assert_eq!(rd.product(), 5.0);
        });
    }

    #[test]
    fn test_reset() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("a", 1.0), ("b", 2.0)]);
            let result = rd.reset(99.0);
            assert_eq!(result.to_dict().get("a"), Some(&99.0));
            assert_eq!(result.to_dict().get("b"), Some(&99.0));
        });
    }

    #[test]
    fn test_reset_to_zero() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("x", 42.0)]);
            let result = rd.reset(0.0);
            assert_eq!(result.to_dict().get("x"), Some(&0.0));
        });
    }

    #[test]
    fn test_original_unchanged_after_reset() {
        Python::initialize();
        Python::attach(|py| {
            let rd = make_dict(py, &[("a", 1.0)]);
            let _ = rd.reset(100.0);
            assert_eq!(rd.to_dict().get("a"), Some(&1.0));
        });
    }

    #[test]
    fn test_multiply_does_not_modify_operands() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 1.0), ("b", 2.0)]);
            let right = make_dict(py, &[("b", 10.0)]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let _ = left.multiply(py_right.bind(py), 1.0).unwrap();
            assert_eq!(left.to_dict().get("a"), Some(&1.0));
            assert_eq!(left.to_dict().get("b"), Some(&2.0));
            assert_eq!(right.to_dict().get("b"), Some(&10.0));
        });
    }

    #[test]
    fn test_divide_does_not_modify_operands() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 10.0), ("b", 6.0)]);
            let right = make_dict(py, &[("b", 2.0)]);
            let py_right = Py::new(py, right.clone()).unwrap();
            let _ = left.divide(py_right.bind(py), 1.0).unwrap();
            assert_eq!(left.to_dict().get("a"), Some(&10.0));
            assert_eq!(left.to_dict().get("b"), Some(&6.0));
        });
    }
}
