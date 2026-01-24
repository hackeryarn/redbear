use std::collections::HashMap;

use pyo3::{prelude::*, types::PyDict};

// I can have this store a reference to a data structure stored internally.
// Then when RedDict gets dropped, I can

#[pyclass]
#[derive(Clone)]
struct RedDict {
    data: HashMap<String, f64>,
}

#[pymethods]
impl RedDict {
    #[new]
    fn new(dict: &Bound<PyDict>) -> PyResult<Self> {
        let data = dict.extract()?;
        Ok(RedDict { data: data })
    }

    fn add_scalar(&mut self, value: f64) -> Self {
        self.data.iter_mut().for_each(|(_, val)| *val += value);
        self.clone()
    }

    fn subtract_scalar(&mut self, value: f64) -> Self {
        self.data.iter_mut().for_each(|(_, val)| *val -= value);
        self.clone()
    }

    fn add(&mut self, other: &Bound<Self>) -> PyResult<Self> {
        let other_data = other.extract::<Self>()?.data;
        self.data
            .iter_mut()
            .for_each(|(key, val)| *val += other_data.get(key).unwrap_or(&0.0));

        Ok(self.clone())
    }

    fn subtract(&mut self, other: &Bound<Self>) -> PyResult<Self> {
        let other_data = other.extract::<Self>()?.data;
        self.data
            .iter_mut()
            .for_each(|(key, val)| *val -= other_data.get(key).unwrap_or(&0.0));

        Ok(self.clone())
    }

    fn multiply(&mut self, other: Bound<Self>) -> PyResult<Self> {
        let other_data = other.extract::<Self>()?.data;
        self.data
            .iter_mut()
            .for_each(|(key, val)| *val *= other_data.get(key).unwrap_or(&0.0));

        Ok(self.clone())
    }

    #[getter]
    fn value(&self) -> HashMap<String, f64> {
        self.data.to_owned()
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn redbear(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<RedDict>()?;
    Ok(())
}
