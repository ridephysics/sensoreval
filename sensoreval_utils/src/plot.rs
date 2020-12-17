use crate::Error;
use plotly_types as plotly;
use std::io::Write;

pub const COLOR_A: &str = "#1f77b4";
pub const COLOR_M: &str = "#ff7f0e";
pub const COLOR_E: &str = "#2ca02c";

static ID2NAME: [&str; 3] = ["e", "n", "u"];

pub struct Plot<'a> {
    plot: plotly::Plot<'a, std::fs::File, &'static str>,
    nrows: usize,
    row_ids: std::collections::HashMap<String, usize>,
    trace_prefix: Option<String>,
}

impl<'a> Plot<'a> {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        let mut f = std::fs::File::create(path)?;

        f.write_all(b"<html><head><meta charset=\"utf-8\"/>")?;
        f.write_all(b"<style>.modebar-container {position: sticky !important;} .infolayer text[class^=\"y\"][class$=\"title\"] {transform: rotate(0deg); writing-mode: vertical-lr; text-orientation: upright; font-weight: bold !important;}</style>")?;
        write!(
            &mut f,
            "<script src=\"{}\" charset=\"utf-8\"></script>",
            plotly::URL_CDN
        )?;
        f.write_all(b"</head><body style=\"margin:0; padding:0; overflow-x:hidden\">")?;
        f.write_all(b"<script type=\"text/javascript\">")?;

        let mut plot = Self {
            plot: plotly::Plot::new(f, "plotly-div")?,
            nrows: 0,
            row_ids: std::collections::HashMap::new(),
            trace_prefix: None,
        };
        let config = plot.plot.config();

        config
            .responsive(true)
            .display_mode_bar(plotly::config::DisplayModeBar::True);
        config.scroll_zoom().set(plotly::config::ScrollZoom::True);

        plot.plot
            .layout()
            .legend()
            .traceorder()
            .flags()
            .grouped(true);

        Ok(plot)
    }

    pub fn set_trace_prefix<A: AsRef<str>>(&mut self, trace_prefix: Option<A>) {
        self.trace_prefix = trace_prefix.map(|v| v.as_ref().to_string());
    }

    pub fn add_row<A: AsRef<str>>(&mut self, name: Option<A>) -> Result<usize, Error> {
        if let Some(name) = name {
            if self.row_ids.get(name.as_ref()).is_some() {
                return Err(Error::RowAlreadyExists);
            }

            self.row_ids.insert(name.as_ref().to_string(), self.nrows);
            self.plot
                .layout()
                .yaxis(self.nrows)
                .title()
                .text(name.as_ref().to_string());
        }

        self.nrows += 1;

        Ok(self.nrows - 1)
    }

    pub fn rowid_by_name<A: AsRef<str>>(&self, name: A) -> Result<usize, Error> {
        self.row_ids
            .get(name.as_ref())
            .cloned()
            .ok_or(Error::RowNotFound)
    }

    pub fn ensure_row<A: AsRef<str>>(&mut self, name: A) -> Result<usize, Error> {
        match self.rowid_by_name(&name) {
            Ok(v) => Ok(v),
            Err(e) => match e {
                Error::RowNotFound => self.add_row(Some(name)),
                _ => Err(e),
            },
        }
    }

    pub fn add_trace(&mut self, trace: &mut plotly::traces::scatter::Scatter) -> Result<(), Error> {
        if self.nrows == 0 {
            return Err(Error::NoRow);
        }

        self.add_trace_to_rowid(trace, self.nrows - 1)
    }

    pub fn add_trace_to_rowname<A: AsRef<str>>(
        &mut self,
        trace: &mut plotly::traces::scatter::Scatter,
        name: A,
    ) -> Result<(), Error> {
        if let Some(id) = self.row_ids.get(name.as_ref()) {
            let id = *id;
            self.add_trace_to_rowid(trace, id)
        } else {
            Err(Error::RowNotFound)
        }
    }

    pub fn add_trace_to_rowid(
        &mut self,
        trace: &mut plotly::traces::scatter::Scatter,
        id: usize,
    ) -> Result<(), Error> {
        if id >= self.nrows {
            return Err(Error::RowNotFound);
        }

        trace.yaxis(format!("y{}", id + 1));

        if let Some(trace_prefix) = &self.trace_prefix {
            trace.name(format!(
                "{}{}",
                trace_prefix,
                trace.name.as_ref().map(|v| v.as_ref()).unwrap_or("")
            ));
        }

        self.plot.add_trace(trace)?;
        Ok(())
    }

    pub fn finish(mut self) -> Result<(), Error> {
        self.plot.layout().grid().rows(self.nrows as i64).columns(1);

        let mut f = self.plot.finish()?;

        let height = (100.0f64 / 3.0 * self.nrows as f64).max(100.0);
        f.write_all(b"</script>")?;
        write!(
            &mut f,
            "<div id=\"plotly-div\" style=\"width:100%;height:{}vh;\"></div>",
            height
        )?;
        f.write_all(b"<script type=\"text/javascript\">")?;
        f.write_all(include_bytes!("js/plot.js"))?;
        f.write_all(b"</script>")?;
        f.write_all(b"<script type=\"text/javascript\">makeplot();</script>")?;
        f.write_all(b"</body></html>")?;

        Ok(())
    }

    pub fn default_line<'b>() -> plotly::traces::scatter::Scatter<'b> {
        let mut t = plotly::traces::scatter::Scatter::default();
        t.xaxis("x");
        t.line().simplify(false);
        t
    }

    pub fn axisid_to_rowname(name: &str, id: usize) -> String {
        format!("{}-{}", name, ID2NAME[id])
    }
}
