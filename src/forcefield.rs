use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

use crate::molecule::{Labels, Topology};

use crate::openmm::{self, System};

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

    pub fn virtual_sites(&self) -> Vec<()> {
        todo!();
    }

    pub fn to_openmm_topology(&self) -> openmm::Topology {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct ForceField {
    inner: Py<PyAny>,
}

impl ForceField {
    pub fn new(name: &str) -> anyhow::Result<Self> {
        let inner = Python::with_gil(|py| {
            let openff_toolkit = PyModule::import(py, "openff.toolkit")?;
            let kwargs = [("allow_cosmetic_attributes", true)].into_py_dict(py);
            Ok::<_, anyhow::Error>(
                openff_toolkit
                    .getattr("ForceField")?
                    .call((String::from(name),), Some(kwargs))?
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

    pub fn bonds(&self) -> Vec<Bond> {
        todo!();
    }

    pub fn angles(&self) -> Vec<Angle> {
        todo!();
    }

    pub fn proper_torsions(&self) -> Vec<ProperTorsion> {
        todo!();
    }

    pub fn to_xml(&self) -> String {
        todo!();
    }
}

#[derive(Clone, Debug)]
pub struct Unit;

pub struct Bond {
    pub parameterize: Option<String>,
    pub value: f64,
    pub unit: Unit,
}

pub struct Angle {
    pub parameterize: Option<String>,
    pub value: f64,
    pub unit: Unit,
}

pub struct ProperTorsion {
    pub parameterize: Option<String>,
    pub value: f64,
    pub unit: Unit,
}
