use pyo3::prelude::*;

use crate::molecule::{Labels, Topology};

use crate::openmm::System;

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
    #[allow(unused)]
    inner: Py<PyAny>,
}

pub struct Interchange {
    inner: Py<PyAny>,
}

impl Interchange {
    pub fn to_openmm(&self) -> System {
        let inner = Python::with_gil(|py| {
            self.inner.call_method0(py, "to_openmm").unwrap()
        });
        System { inner }
    }
}

impl ForceField {
    pub fn new(name: &str) -> anyhow::Result<Self> {
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
        ParameterHandler { inner }
    }

    pub fn create_interchange(
        &self,
        topology: Topology,
    ) -> anyhow::Result<Interchange> {
        let inner = Python::with_gil(|py| {
            self.inner
                .call_method1(py, "create_interchange", (topology.inner,))
        })?;
        Ok(Interchange { inner })
    }

    py_method! { label_molecules, Labels, top => Topology, into }
}
