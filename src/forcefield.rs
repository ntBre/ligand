use pyo3::prelude::*;

use crate::molecule::{Labels, Topology};

#[derive(Debug)]
pub struct ForceField {
    inner: Py<PyAny>,
}

pub enum ParameterType {
    Bonds,
    Angles,
    Torsions,
}

impl ParameterType {
    fn as_str(&self) -> &'static str {
        match self {
            ParameterType::Bonds => "Bonds",
            ParameterType::Angles => "Angles",
            ParameterType::Torsions => "ProperTorsions",
        }
    }
}

pub struct ParameterHandler {
    _inner: Py<PyAny>,
}

// impl ParameterHandler {
//     pub(crate) fn parameters(&self) {
//     }
// }

impl ForceField {
    pub(crate) fn new(name: &str) -> anyhow::Result<Self> {
        let inner = Python::with_gil(|py| {
            let openff_toolkit = PyModule::import(py, "openff.toolkit")?;
            Ok::<_, anyhow::Error>(
                openff_toolkit
                    .getattr("ForceField")?
                    .call1((String::from(name),))?
                    .into(),
            )
        })?;
        Ok(Self { inner })
    }

    pub fn get_parameter_handler(
        &self,
        typ: ParameterType,
    ) -> ParameterHandler {
        let inner = Python::with_gil(|py| {
            self.inner
                .call_method1(py, "get_parameter_handler", (typ.as_str(),))
                .unwrap()
        });
        ParameterHandler { _inner: inner }
    }

    py_method! { label_molecules, Labels, top => Topology, into }
}
