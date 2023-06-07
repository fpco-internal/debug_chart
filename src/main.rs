use anyhow::Result;
use chrono::offset::Utc;
use chrono::DateTime;
use plotters::prelude::*;
use reqwest::blocking::get;
use serde_json::from_reader;
use std::collections::{HashMap, VecDeque};
use std::env::args;
use std::fs::File;
use std::path::Path;

fn main() -> Result<()> {
    let src = args().next().expect("One argument should be passed");
    let input: HashMap<String, GasRecords> = if !Path::new(&src).is_file() {
        get(&src)?.json()?
    } else {
        from_reader(File::open(&src)?)?
    };
    let input: HashMap<&String, Vec<i64>> = input
        .iter()
        .map(|(k, v)| {
            let e1 = v.entries.clone();
            let e2 = v.entries.clone().pop_front();
            let usage: Vec<_> = e1
                .iter()
                .zip(e2.iter())
                .map(|(e1, e2)| {
                    (e2.timestamp - e1.timestamp).num_seconds()
                        / ((e1.amount * 1000000.0).round() as i64)
                })
                .collect();
            (k, usage)
        })
        .collect();
    let (_, possible_filename) = src.rsplit_once('/').unwrap_or(("", &src));
    let (_, possible_filename) = possible_filename
        .rsplit_once('\\')
        .unwrap_or(("", possible_filename));
    let chart_name = format!("{possible_filename}.png");
    let x_range = 0_usize..1000;

    let root_area = BitMapBackend::new(&chart_name, (1024, 768)).into_drawing_area();
    root_area.fill(&WHITE)?;
    let mut cc = ChartBuilder::on(&root_area).build_cartesian_2d(x_range, 0..i64::MAX)?;

    input.into_iter().try_for_each(|(k, vs)| {
        cc.draw_series(LineSeries::new(vs.into_iter().enumerate(), &RED))
            .map(|x| {
                x.label(k);
            })
    })?;

    root_area.present()?;
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GasRecords {
    total: f64,
    entries: VecDeque<GasEntry>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct GasEntry {
    timestamp: DateTime<Utc>,
    amount: f64,
}
