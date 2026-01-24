use std::collections::HashMap;
use std::sync::Arc;

use pyo3::{prelude::*, types::PyDict};

// I can have this store a reference to a data structure stored internally.
// Then when RedDict gets dropped, I can

#[pyclass]
#[derive(Clone)]
struct RedDict {
    data: Arc<HashMap<String, f64>>,
}

#[pymethods]
impl RedDict {
    #[new]
    fn new(dict: &Bound<PyDict>) -> PyResult<Self> {
        let data = Arc::new(dict.extract()?);
        Ok(RedDict { data: data })
    }

    fn add_scalar(&self, value: f64) -> Self {
        let mut new = self.clone();
        Arc::make_mut(&mut new.data)
            .iter_mut()
            .for_each(|(_, val)| *val += value);
        new
    }

    fn subtract_scalar(&self, value: f64) -> Self {
        let mut new = self.clone();
        Arc::make_mut(&mut new.data)
            .iter_mut()
            .for_each(|(_, val)| *val -= value);
        new
    }

    fn add(&self, other: &Bound<Self>) -> PyResult<Self> {
        let other_data = other.extract::<Self>()?.data;
        let mut new = self.clone();
        Arc::make_mut(&mut new.data)
            .iter_mut()
            .for_each(|(key, val)| *val += other_data.get(key).unwrap_or(&0.0));

        Ok(new)
    }

    fn subtract(&self, other: &Bound<Self>) -> PyResult<Self> {
        let other_data = other.extract::<Self>()?.data;
        let mut new = self.clone();
        Arc::make_mut(&mut new.data)
            .iter_mut()
            .for_each(|(key, val)| *val -= other_data.get(key).unwrap_or(&0.0));

        Ok(new)
    }

    fn multiply(&self, other: Bound<Self>) -> PyResult<Self> {
        let other_data = other.extract::<Self>()?.data;
        let mut new = self.clone();
        Arc::make_mut(&mut new.data)
            .iter_mut()
            .for_each(|(key, val)| *val *= other_data.get(key).unwrap_or(&0.0));

        Ok(new)
    }

    #[getter]
    fn value(&self) -> HashMap<String, f64> {
        Arc::unwrap_or_clone(self.data.clone())
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn redbear(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<RedDict>()?;
    Ok(())
}
