use qr_model::{Bucket, Builtin, ChartSeries};
use rusqlite::Connection;

fn list_series_by_builtin(builtin: &Builtin) -> Vec<qr_model::ChartSeries> {
    match builtin {
        Builtin::BrowserTime => vec![ChartSeries::CalendarHeat, ChartSeries::Line],
    }
}

pub fn list_series_by_bucket(
    _conn: &Connection,
    bucket: &Bucket,
) -> Result<Vec<qr_model::ChartSeries>, String> {
    // TODO
    let Bucket { builtin, .. } = bucket;
    match builtin {
        Some(b) => Ok(list_series_by_builtin(b)),
        None => todo!(),
    }
}
