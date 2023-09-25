use anyhow::Result;
use pyo3::{prelude::*, types::IntoPyDict};

#[derive(Debug, Clone)]
pub struct Molecule {
    #[allow(unused)]
    pub(crate) inner: Py<PyAny>,
}

#[derive(Debug, Clone)]
pub struct Topology {
    pub(crate) inner: Py<PyAny>,
}

impl Topology {
    #[allow(unused)]
    pub(crate) fn new(inner: Py<PyAny>) -> Self {
        Self { inner }
    }
}

impl IntoPy<Py<PyAny>> for Topology {
    fn into_py(self, _py: Python<'_>) -> Py<PyAny> {
        self.inner
    }
}

/// A "list" of molecule labels. as I usually call it in Python, there is only
/// one molecule, so there should only be one entry in the list. that's why
/// `get` first indexes list[0] and then the underlying dict
#[derive(Debug, Clone)]
pub struct Labels {
    #[allow(unused)]
    pub(crate) inner: Py<PyAny>,
}

impl Labels {
    pub(crate) fn new(inner: Py<PyAny>) -> Self {
        Self { inner }
    }

    pub fn get(&self, index: &'static str) -> Py<PyAny> {
        Python::with_gil(|py| {
            self.inner
                .call_method1(py, "__getitem__", (0,))
                .unwrap()
                .call_method1(py, "__getitem__", (index,))
                .unwrap()
        })
    }
}

impl Molecule {
    pub fn from_mapped_smiles(smiles: &str) -> Result<Self> {
        let inner = Python::with_gil(|py| {
            PyModule::from_code(
                py,
                r#"import logging
logging.getLogger("openff").setLevel(logging.ERROR)
    "#,
                "",
                "",
            )
            .unwrap();
            let openff_toolkit = PyModule::import(py, "openff.toolkit")?;
            let kwargs = [("allow_undefined_stereo", true)].into_py_dict(py);
            Ok::<_, anyhow::Error>(
                openff_toolkit
                    .getattr("Molecule")?
                    .call_method(
                        "from_mapped_smiles",
                        (String::from(smiles),),
                        Some(kwargs),
                    )?
                    .into(),
            )
        })?;

        Ok(Self { inner })
    }

    pub fn to_mapped_smiles(&self) -> String {
        Python::with_gil(|py| {
            let kwargs = [("mapped", true)].into_py_dict(py);
            self.inner
                .call_method(py, "to_smiles", (), Some(kwargs))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// return the Cartesian geometry of the `idx`th conformer of `self` as a
    /// flattened vector in units of Å
    pub fn get_conformer(&self, idx: usize) -> Vec<f64> {
        Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                "def get_conformer(mol, idx):
    return mol.conformers[idx].magnitude.flatten().tolist()
",
                "",
                "",
            )
            .unwrap()
            .getattr("get_conformer")
            .unwrap();
            fun.call1((&self.inner, idx)).unwrap().extract().unwrap()
        })
    }

    pub fn to_svg(&self) -> String {
        Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                "def draw_rdkit(mol):
    from rdkit.Chem.Draw import rdDepictor, rdMolDraw2D
    rdmol = mol.to_rdkit()
    rdDepictor.SetPreferCoordGen(True)
    rdDepictor.Compute2DCoords(rdmol)
    rdmol = rdMolDraw2D.PrepareMolForDrawing(rdmol)
    drawer = rdMolDraw2D.MolDraw2DSVG(300, 300)
    drawer.DrawMolecule(rdmol)
    drawer.FinishDrawing()
    return drawer.GetDrawingText()
",
                "",
                "",
            )
            .unwrap()
            .getattr("draw_rdkit")
            .unwrap();
            fun.call1((self.inner.clone(),)).unwrap().extract().unwrap()
        })
    }

    pub fn add_conformer(&mut self, coordinates: Vec<f64>) {
        Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                "def add_conformer(mol, coordinates):
    from openff.units import Quantity, unit
    import numpy as np
    a = np.array(coordinates)
    a = np.reshape(a, (-1, 3))
    c = Quantity(a, unit.angstrom)
    mol.add_conformer(c)
",
                "",
                "",
            )
            .unwrap()
            .getattr("add_conformer")
            .unwrap();
            fun.call1((&self.inner, coordinates)).unwrap();
        });
    }

    /// calls `self.to_inchi(fixed_hydrogens=True)`
    pub fn to_inchi(&self) -> String {
        Python::with_gil(|py| {
            let kwargs = [("fixed_hydrogens", true)].into_py_dict(py);
            self.inner
                .call_method(py, "to_inchi", (), Some(kwargs))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    py_method! {to_inchikey, String}

    py_method! {chemical_environment_matches, Vec<(usize, usize)>, query => &str}

    py_method! {to_topology, Topology, into}

    /// helper method for calling the Molecule.`method` constructors with kwargs
    /// of `allow_undefined_stereo = True`
    fn from_pattern(
        method: &str,
        pattern: &str,
    ) -> std::result::Result<Molecule, anyhow::Error> {
        let inner = Python::with_gil(|py| {
            PyModule::from_code(
                py,
                r#"import logging
logging.getLogger("openff").setLevel(logging.ERROR)
    "#,
                "",
                "",
            )
            .unwrap();
            let openff_toolkit = PyModule::import(py, "openff.toolkit")?;
            let kwargs = [("allow_undefined_stereo", true)].into_py_dict(py);
            Ok::<_, anyhow::Error>(
                openff_toolkit
                    .getattr("Molecule")?
                    .call_method(
                        method,
                        (String::from(pattern),),
                        Some(kwargs),
                    )?
                    .into(),
            )
        })?;

        Ok(Self { inner })
    }
}
