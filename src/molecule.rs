use anyhow::Result;
use pyo3::prelude::*;

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
            let openff_toolkit = PyModule::import(py, "openff.toolkit")?;
            Ok::<_, anyhow::Error>(
                openff_toolkit
                    .getattr("Molecule")?
                    .call_method1(
                        "from_mapped_smiles",
                        (String::from(smiles),),
                    )?
                    .into(),
            )
        })?;

        Ok(Self { inner })
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

    py_method! {to_inchi, String}

    py_method! {chemical_environment_matches, Vec<(usize, usize)>, query => &str}

    py_method! {to_topology, Topology, into}
}
