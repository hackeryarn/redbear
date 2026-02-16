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

use pyo3::prelude::*;

#[pyclass(from_py_object, mapping)]
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
    fn new(dict: HashMap<String, f64>) -> Self {
        let mut values = Vec::with_capacity(dict.len());
        let mut index = HashMap::with_capacity(dict.len());

        for (pos, (k, v)) in dict.into_iter().enumerate() {
            values.push(v);
            index.insert(k, pos);
        }

        Self {
            index: Arc::new(index),
            values: Arc::new(values),
        }
    }

    /// Allows element wise access to the dictionary items. Note that this access
    /// will be about twice as slow as normal python dictionary access.
    ///
    /// # Example
    ///
    /// ```python
    /// >>> import redbear as rb
    /// >>> d = rb.RedDict({"x": 1.0, "y": 2.0})
    /// >>> d["x"]
    /// 1.0
    /// ```
    fn __getitem__(&self, key: &str) -> Option<f64> {
        self.index
            .get(key)
            .and_then(|&i| self.values.get(i).copied())
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
    fn add_scalar(&self, value: f64) -> Self {
        self.perform_scalar(value, |a, b| a + b)
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
    fn add(&self, other: Self, fill: f64) -> Self {
        self.merge(&other, fill, |a, b| a + b)
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
    fn subtract_scalar(&self, value: f64) -> Self {
        self.perform_scalar(value, |a, b| a - b)
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
    fn subtract(&self, other: Self, fill: f64) -> Self {
        self.merge(&other, fill, |a, b| a - b)
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
    fn multiply_scalar(&self, value: f64) -> Self {
        self.perform_scalar(value, |a, b| a * b)
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
    fn multiply(&self, other: Self, fill: f64) -> Self {
        self.merge(&other, fill, |a, b| a * b)
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
    fn divide_scalar(&self, value: f64) -> Self {
        self.perform_scalar(value, |a, b| a / b)
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
    fn divide(&self, other: Self, fill: f64) -> Self {
        self.merge(&other, fill, |a, b| a / b)
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
    fn reset(&self, value: f64) -> Self {
        let mut new = self.clone();
        Arc::make_mut(&mut new.values).fill(value);
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

impl RedDict {
    /// Shared implementation for binary element-wise operations.
    ///
    /// `fill` is the value used when `other` is missing a key present in `self`.
    fn merge<F>(&self, other: &Self, fill: f64, f: F) -> Self
    where
        F: Fn(f64, f64) -> f64,
    {
        let mut new = self.clone();
        let new_vals = Arc::make_mut(&mut new.values);

        if new.index == other.index {
            for (lhs, rhs) in new_vals.iter_mut().zip(other.values.iter()) {
                *lhs = f(*lhs, *rhs);
            }
        } else {
            for (key, &i) in new.index.iter() {
                let rhs = other
                    .index
                    .get(key)
                    .map(|&j| other.values[j])
                    .unwrap_or(fill);
                new_vals[i] = f(new_vals[i], rhs);
            }
        }
        new
    }

    fn perform_scalar<F>(&self, value: f64, f: F) -> Self
    where
        F: Fn(f64, f64) -> f64,
    {
        let mut new = self.clone();
        Arc::make_mut(&mut new.values)
            .iter_mut()
            .for_each(|val| *val = f(*val, value));
        new
    }
}

/// A Python module implemented in Rust.
#[pymodule]
mod redbear {
    #[pymodule_export]
    use super::RedDict;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dict(entries: &[(&str, f64)]) -> RedDict {
        let hashmap: HashMap<String, f64> =
            entries.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        RedDict::new(hashmap)
    }

    #[test]
    fn test_new_from_empty_dict() {
        let rd = make_dict(&[]);
        assert_eq!(rd.to_dict().len(), 0);
    }

    #[test]
    fn test_new_from_single_entry() {
        let rd = make_dict(&[("x", 42.0)]);
        let map = rd.to_dict();
        assert_eq!(map.get("x"), Some(&42.0));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_new_from_multiple_entries() {
        let rd = make_dict(&[("a", 1.0), ("b", 2.0), ("c", 3.0)]);
        let map = rd.to_dict();
        assert_eq!(map.get("a"), Some(&1.0));
        assert_eq!(map.get("b"), Some(&2.0));
        assert_eq!(map.get("c"), Some(&3.0));
    }

    #[test]
    fn test_getitem_existing_key() {
        let rd = make_dict(&[("x", 5.0), ("y", 10.0)]);
        assert_eq!(rd.__getitem__("x"), Some(5.0));
        assert_eq!(rd.__getitem__("y"), Some(10.0));
    }

    #[test]
    fn test_getitem_missing_key() {
        let rd = make_dict(&[("a", 1.0)]);
        assert_eq!(rd.__getitem__("missing"), None);
    }

    #[test]
    fn test_getitem_empty_dict() {
        let rd = make_dict(&[]);
        assert_eq!(rd.__getitem__("anything"), None);
    }

    #[test]
    fn test_add_scalar() {
        let rd = make_dict(&[("a", 1.0), ("b", -2.0)]);
        let result = rd.add_scalar(3.0);
        assert_eq!(result.to_dict().get("a"), Some(&4.0));
        assert_eq!(result.to_dict().get("b"), Some(&1.0));
    }

    #[test]
    fn test_subtract_scalar() {
        let rd = make_dict(&[("a", 5.0), ("b", 3.0)]);
        let result = rd.subtract_scalar(2.0);
        assert_eq!(result.to_dict().get("a"), Some(&3.0));
        assert_eq!(result.to_dict().get("b"), Some(&1.0));
    }

    #[test]
    fn test_add_scalar_negative() {
        let rd = make_dict(&[("a", 10.0)]);
        let result = rd.add_scalar(-5.0);
        assert_eq!(result.to_dict().get("a"), Some(&5.0));
    }

    #[test]
    fn test_operations_return_new_instance() {
        let rd = make_dict(&[("a", 1.0)]);
        let added = rd.add_scalar(1.0);
        let subtracted = rd.subtract_scalar(1.0);
        assert_eq!(rd.to_dict().get("a"), Some(&1.0));
        assert_eq!(added.to_dict().get("a"), Some(&2.0));
        assert_eq!(subtracted.to_dict().get("a"), Some(&0.0));
    }

    #[test]
    fn test_add_uses_fill_for_missing_keys() {
        let left = make_dict(&[("a", 1.0), ("b", 2.0)]);
        let right = make_dict(&[("b", 10.0), ("c", 100.0)]);
        let result = left.add(right, 5.0);
        assert_eq!(result.to_dict().get("a"), Some(&6.0));
        assert_eq!(result.to_dict().get("b"), Some(&12.0));
        assert!(!result.to_dict().contains_key("c"));
    }

    #[test]
    fn test_subtract_uses_fill_for_missing_keys() {
        let left = make_dict(&[("a", 10.0), ("b", 5.0)]);
        let right = make_dict(&[("b", 2.0)]);
        let result = left.subtract(right, 3.0);
        assert_eq!(result.to_dict().get("a"), Some(&7.0));
        assert_eq!(result.to_dict().get("b"), Some(&3.0));
    }

    #[test]
    fn test_multiply_uses_fill_for_missing_keys() {
        let left = make_dict(&[("a", 2.0), ("b", 3.0)]);
        let right = make_dict(&[("b", 10.0)]);
        let result = left.multiply(right, 1.0);
        assert_eq!(result.to_dict().get("a"), Some(&2.0));
        assert_eq!(result.to_dict().get("b"), Some(&30.0));
    }

    #[test]
    fn test_add_default_fill_is_zero() {
        let left = make_dict(&[("a", 1.0)]);
        let right = make_dict(&[]);
        let result = left.add(right, 0.0);
        assert_eq!(result.to_dict().get("a"), Some(&1.0));
    }

    #[test]
    fn test_subtract_default_fill_is_zero() {
        let left = make_dict(&[("a", 5.0)]);
        let right = make_dict(&[]);
        let result = left.subtract(right, 0.0);
        assert_eq!(result.to_dict().get("a"), Some(&5.0));
    }

    #[test]
    fn test_multiply_default_fill_is_one() {
        let left = make_dict(&[("a", 7.0)]);
        let right = make_dict(&[]);
        let result = left.multiply(right, 1.0);
        assert_eq!(result.to_dict().get("a"), Some(&7.0));
    }

    #[test]
    fn test_add_fast_path_identical_keys() {
        let left = make_dict(&[("a", 1.0), ("b", 2.0)]);
        let right = make_dict(&[("a", 10.0), ("b", 20.0)]);
        let result = left.add(right, 0.0);
        assert_eq!(result.to_dict().get("a"), Some(&11.0));
        assert_eq!(result.to_dict().get("b"), Some(&22.0));
    }

    #[test]
    fn test_subtract_fast_path_identical_keys() {
        let left = make_dict(&[("a", 10.0), ("b", 20.0)]);
        let right = make_dict(&[("a", 3.0), ("b", 5.0)]);
        let result = left.subtract(right, 0.0);
        assert_eq!(result.to_dict().get("a"), Some(&7.0));
        assert_eq!(result.to_dict().get("b"), Some(&15.0));
    }

    #[test]
    fn test_multiply_fast_path_identical_keys() {
        let left = make_dict(&[("a", 2.0), ("b", 3.0)]);
        let right = make_dict(&[("a", 5.0), ("b", 4.0)]);
        let result = left.multiply(right, 1.0);
        assert_eq!(result.to_dict().get("a"), Some(&10.0));
        assert_eq!(result.to_dict().get("b"), Some(&12.0));
    }

    #[test]
    fn test_does_not_modify_operands() {
        let left = make_dict(&[("a", 1.0), ("b", 2.0)]);
        let right = make_dict(&[("b", 10.0)]);
        let _ = left.add(right.clone(), 5.0);
        let _ = left.subtract(right.clone(), 0.0);
        let _ = left.multiply(right.clone(), 1.0);
        assert_eq!(left.to_dict().get("a"), Some(&1.0));
        assert_eq!(left.to_dict().get("b"), Some(&2.0));
        assert_eq!(right.to_dict().get("b"), Some(&10.0));
    }

    #[test]
    fn test_chained_operations() {
        let rd = make_dict(&[("x", 1.0)]);
        let result = rd.add_scalar(2.0).subtract_scalar(1.0).add(rd.clone(), 0.0);
        assert_eq!(result.to_dict().get("x"), Some(&3.0));
    }

    #[test]
    fn test_multiply_scalar() {
        let rd = make_dict(&[("a", 2.0), ("b", 5.0)]);
        let result = rd.multiply_scalar(3.0);
        assert_eq!(result.to_dict().get("a"), Some(&6.0));
        assert_eq!(result.to_dict().get("b"), Some(&15.0));
    }

    #[test]
    fn test_multiply_scalar_by_zero() {
        let rd = make_dict(&[("x", 42.0)]);
        let result = rd.multiply_scalar(0.0);
        assert_eq!(result.to_dict().get("x"), Some(&0.0));
    }

    #[test]
    fn test_divide_scalar() {
        let rd = make_dict(&[("a", 10.0), ("b", 6.0)]);
        let result = rd.divide_scalar(2.0);
        assert_eq!(result.to_dict().get("a"), Some(&5.0));
        assert_eq!(result.to_dict().get("b"), Some(&3.0));
    }

    #[test]
    fn test_divide_scalar_by_fraction() {
        let rd = make_dict(&[("x", 1.0)]);
        let result = rd.divide_scalar(0.5);
        assert_eq!(result.to_dict().get("x"), Some(&2.0));
    }

    #[test]
    fn test_divide() {
        let left = make_dict(&[("a", 10.0), ("b", 6.0)]);
        let right = make_dict(&[("b", 2.0)]);
        let result = left.divide(right, 1.0);
        assert_eq!(result.to_dict().get("a"), Some(&10.0));
        assert_eq!(result.to_dict().get("b"), Some(&3.0));
    }

    #[test]
    fn test_divide_default_fill_is_one() {
        let left = make_dict(&[("a", 7.0)]);
        let right = make_dict(&[]);
        let result = left.divide(right, 1.0);
        assert_eq!(result.to_dict().get("a"), Some(&7.0));
    }

    #[test]
    fn test_sum() {
        let rd = make_dict(&[("a", 1.0), ("b", 2.0), ("c", 3.0)]);
        assert_eq!(rd.sum(), 6.0);
    }

    #[test]
    fn test_sum_empty() {
        let rd = make_dict(&[]);
        assert_eq!(rd.sum(), 0.0);
    }

    #[test]
    fn test_product() {
        let rd = make_dict(&[("a", 2.0), ("b", 3.0), ("c", 4.0)]);
        assert_eq!(rd.product(), 24.0);
    }

    #[test]
    fn test_product_single_element() {
        let rd = make_dict(&[("x", 5.0)]);
        assert_eq!(rd.product(), 5.0);
    }

    #[test]
    fn test_reset() {
        let rd = make_dict(&[("a", 1.0), ("b", 2.0)]);
        let result = rd.reset(99.0);
        assert_eq!(result.to_dict().get("a"), Some(&99.0));
        assert_eq!(result.to_dict().get("b"), Some(&99.0));
    }

    #[test]
    fn test_reset_to_zero() {
        let rd = make_dict(&[("x", 42.0)]);
        let result = rd.reset(0.0);
        assert_eq!(result.to_dict().get("x"), Some(&0.0));
    }

    #[test]
    fn test_original_unchanged_after_reset() {
        let rd = make_dict(&[("a", 1.0)]);
        let _ = rd.reset(100.0);
        assert_eq!(rd.to_dict().get("a"), Some(&1.0));
    }

    #[test]
    fn test_multiply_does_not_modify_operands() {
        let left = make_dict(&[("a", 1.0), ("b", 2.0)]);
        let right = make_dict(&[("b", 10.0)]);
        let _ = left.multiply(right.clone(), 1.0);
        assert_eq!(left.to_dict().get("a"), Some(&1.0));
        assert_eq!(left.to_dict().get("b"), Some(&2.0));
        assert_eq!(right.to_dict().get("b"), Some(&10.0));
    }

    #[test]
    fn test_divide_does_not_modify_operands() {
        let left = make_dict(&[("a", 10.0), ("b", 6.0)]);
        let right = make_dict(&[("b", 2.0)]);
        let _ = left.divide(right.clone(), 1.0);
        assert_eq!(left.to_dict().get("a"), Some(&10.0));
        assert_eq!(left.to_dict().get("b"), Some(&6.0));
    }
}
