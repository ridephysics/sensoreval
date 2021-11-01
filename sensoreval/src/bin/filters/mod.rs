pub mod apply_calibration;
pub mod axismap;
pub mod load_data;
pub mod pressure_coeff;

macro_rules! filters {
    ( $( $x:ident ),* ) => {
        #[derive(Debug, serde::Deserialize)]
        #[serde(tag = "type")]
        #[serde(deny_unknown_fields)]
        #[allow(non_camel_case_types)]
        pub enum Args {
            $( $x($x::Config), )*
        }

        impl Args {
            pub fn apply(&self, data: &mut crate::Dataset) -> anyhow::Result<()> {
                match self {
                    $( Self::$x(c) => $x::apply(c, data), )*
                }
            }
        }
    };
}

filters!(apply_calibration, axismap, load_data, pressure_coeff);
