mod filters;

use anyhow::Context as _;

/// a set of multiple kinds of data
///
/// Samples for data-kinds would be `accelerometer` or `pressure`.
///
/// Each data kind is an array with N number of elements representing a
/// time-series of samples.
/// Users must ensure that the N is the same for all kinds within this dataset
/// and that at a certain index the data of all kinds occured at the same time.
///
/// The times for each index of a sample are stored in a separate list internally.
///
/// Cloning this struct does not clone the kind-data since Arcs are used internally.
/// The list of kinds is being cloned though.
#[derive(Clone, Debug)]
pub struct Dataset {
    times: std::sync::Arc<ndarray::Array1<u64>>,
    kinds: std::collections::HashMap<String, std::sync::Arc<ndarray::ArrayD<f64>>>,
}

impl Dataset {
    pub fn new(times: ndarray::Array1<u64>) -> Self {
        Self {
            times: std::sync::Arc::new(times),
            kinds: std::collections::HashMap::new(),
        }
    }

    /// return the number of samples in this dataset
    ///
    /// Not to be confused with the number of kinds.
    /// The number of samples is the length of the array of each kind which should
    /// be the same for all of them.
    pub fn len(&self) -> usize {
        self.times.len()
    }

    pub fn is_empty(&self) -> bool {
        self.times.is_empty()
    }

    /// return time codes for this dataset
    pub fn times(&self) -> &ndarray::Array1<u64> {
        self.times.as_ref()
    }

    /// looks up a data kind by name and returns a reference to it's data
    pub fn get_kind(&self, name: &str) -> Option<&ndarray::ArrayD<f64>> {
        self.kinds.get(name).map(|kind| kind.as_ref())
    }

    /// looks up a data kind by name and returns a mutable reference to it's data
    ///
    /// This will clone the data if there are any other Datasets holding a
    /// reference to it.
    pub fn get_kind_mut(&mut self, name: &str) -> Option<&mut ndarray::ArrayD<f64>> {
        self.kinds
            .get_mut(name)
            .map(|kind| std::sync::Arc::make_mut(kind))
    }

    pub fn add_kind<N: Into<String>>(&mut self, name: N, value: ndarray::ArrayD<f64>) {
        self.kinds.insert(name.into(), std::sync::Arc::new(value));
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FilterConfig {
    /// ID that can be used to refer to the resulting data
    id: Option<String>,
    /// if non-empty, this is used to construct the input data for the filter
    #[serde(default)]
    data: std::collections::HashMap<String, ()>,
    /// the type of filter to use and it's arguments
    args: filters::Args,
}

impl FilterConfig {
    pub fn apply(&self, data: &mut Dataset) -> anyhow::Result<()> {
        self.args.apply(data)
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args: Vec<_> = std::env::args().collect();
    let filename = &args[1];
    let file = std::fs::File::open(filename)?;
    let value: std::collections::HashMap<String, Vec<FilterConfig>> =
        serde_yaml::from_reader(&file).context("can't read yaml")?;
    log::debug!("{:#?}", value);

    let mut dataset = Dataset::new(ndarray::array![]);
    for config in &value["main"] {
        config.apply(&mut dataset).context("can't apply filter")?;
    }

    //log::trace!("{:#?}", dataset);

    Ok(())
}
